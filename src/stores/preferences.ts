import { defineStore } from "pinia";
import { ref } from "vue";
import * as preferencesApi from "@api";
import { setLocale } from "../i18n";
import type {
  ApplicationSettingsUpdate,
  ConnectionSettingsUpdate,
  HostUriLifetime,
  LightweightSettingsUpdate,
  Locale,
  PersonalizationUpdate,
  Preferences,
  ThemePreference,
} from "../models/preferences";
import { applyColorTheme } from "../themes/apply";
import { applyTypography, DEFAULT_FONT_SIZE } from "../ui/typography";

const defaultCustomTheme = {
  light: {
    bg: "#f7f7f6",
    bgSecondary: "#f0f0ef",
    bgTertiary: "#dedfe0",
    primary: "#45505d",
    primarySolid: "#45505d",
    primarySolidHover: "#36414d",
    secondary: "#69727c",
    textPrimary: "#202326",
    textSecondary: "#666b70",
    border: "#d6d8da",
  },
  dark: {
    bg: "#111214",
    bgSecondary: "#191a1d",
    bgTertiary: "#25272b",
    primary: "#aab4c0",
    primarySolid: "#455666",
    primarySolidHover: "#536778",
    secondary: "#c1c7cf",
    textPrimary: "#f1f1f2",
    textSecondary: "#a6a8ae",
    border: "#30343a",
  },
};

const defaults: Preferences = {
  theme: "system",
  colorTheme: "inkstone",
  customTheme: defaultCustomTheme,
  fontSize: DEFAULT_FONT_SIZE,
  fontFamily: "",
  splashDurationMs: 1000,
  silentStart: false,
  locale: "zh-CN",
  rememberWindowState: true,
  windowMaterial: "solid",
  autoLightweightMinutes: null,
  hostUriLifetime: "always",
  joinUri: "",
  joinPort: 25565,
  reconnectTimeoutSecs: null,
  relayCustom: false,
  relayUrl: "",
};

async function restartForMaterialChange(): Promise<boolean> {
  try {
    await preferencesApi.restartApplication();
    return true;
  } catch (error) {
    console.error("Failed to restart application", error);
    return false;
  }
}

export const usePreferencesStore = defineStore("preferences", () => {
  const preferences = ref<Preferences>({ ...defaults });
  const systemTheme = window.matchMedia("(prefers-color-scheme: dark)");
  let themeSaveQueue = Promise.resolve();
  let localeSaveQueue = Promise.resolve();
  let personalizationSaveQueue = Promise.resolve();
  let personalizationRevision = 0;
  let applicationSaveQueue = Promise.resolve();
  let connectionSaveQueue = Promise.resolve();
  let lightweightSaveQueue = Promise.resolve();

  function applyTheme(): void {
    applyColorTheme(
      preferences.value.theme,
      preferences.value.colorTheme,
      systemTheme.matches,
      preferences.value.windowMaterial,
      preferences.value.customTheme,
    );
  }

  function applyPersonalizationStyles(): void {
    applyTheme();
    applyTypography(preferences.value.fontSize, preferences.value.fontFamily);
  }

  async function load(): Promise<void> {
    try {
      preferences.value = await preferencesApi.getPreferences();
      setLocale(preferences.value.locale);
    } catch (error) {
      console.error("Failed to load preferences", error);
    }
    applyPersonalizationStyles();
  }

  async function saveTheme(theme: ThemePreference, fallback: ThemePreference): Promise<void> {
    try {
      await preferencesApi.saveTheme(theme);
    } catch (error) {
      if (preferences.value.theme === theme) {
        preferences.value.theme = fallback;
        applyTheme();
      }
      console.error("Failed to save theme preference", error);
    }
  }

  function setTheme(theme: ThemePreference): void {
    if (preferences.value.theme === theme) return;
    const fallback = preferences.value.theme;
    preferences.value.theme = theme;
    applyTheme();
    themeSaveQueue = themeSaveQueue.then(() => saveTheme(theme, fallback));
  }

  async function saveLocale(locale: Locale, fallback: Locale): Promise<void> {
    try {
      await preferencesApi.saveLocale(locale);
    } catch (error) {
      if (preferences.value.locale === locale) {
        preferences.value.locale = fallback;
        setLocale(fallback);
      }
      console.error("Failed to save locale preference", error);
    }
  }

  function changeLocale(locale: Locale): void {
    if (preferences.value.locale === locale) return;
    const fallback = preferences.value.locale;
    preferences.value.locale = locale;
    setLocale(locale);
    localeSaveQueue = localeSaveQueue.then(() => saveLocale(locale, fallback));
  }

  function updatePersonalization(update: PersonalizationUpdate): void {
    const snapshot = { ...update };
    snapshot.customTheme = structuredClone(update.customTheme);
    Object.assign(preferences.value, snapshot);
    applyPersonalizationStyles();
    const revision = ++personalizationRevision;
    personalizationSaveQueue = personalizationSaveQueue
      .then(() => {
        if (revision !== personalizationRevision) return;
        return preferencesApi.savePersonalization(snapshot);
      })
      .catch((error) => console.error("Failed to save personalization", error));
  }

  function updateApplicationSettings(update: ApplicationSettingsUpdate): void {
    const snapshot = { ...update };
    Object.assign(preferences.value, snapshot);
    applicationSaveQueue = applicationSaveQueue
      .then(() => preferencesApi.saveApplicationSettings(snapshot))
      .catch((error) => console.error("Failed to save application settings", error));
  }

  function updateConnectionSettings(update: ConnectionSettingsUpdate): void {
    const snapshot = { ...update };
    Object.assign(preferences.value, snapshot);
    connectionSaveQueue = connectionSaveQueue
      .then(() => preferencesApi.saveConnectionSettings(snapshot))
      .catch((error) => console.error("Failed to save connection settings", error));
  }

  function updateLightweightSettings(update: LightweightSettingsUpdate): void {
    const snapshot = { ...update };
    Object.assign(preferences.value, snapshot);
    lightweightSaveQueue = lightweightSaveQueue
      .then(() => preferencesApi.saveLightweightSettings(snapshot))
      .catch((error) => console.error("Failed to save lightweight settings", error));
  }

  async function setHostUriLifetime(lifetime: HostUriLifetime): Promise<boolean> {
    if (preferences.value.hostUriLifetime === lifetime) return true;
    const fallback = preferences.value.hostUriLifetime;
    preferences.value.hostUriLifetime = lifetime;
    try {
      await preferencesApi.saveInviteLifetime(lifetime);
      return true;
    } catch (error) {
      if (preferences.value.hostUriLifetime === lifetime) {
        preferences.value.hostUriLifetime = fallback;
      }
      console.error("Failed to save host URI lifetime", error);
      return false;
    }
  }

  function handleSystemThemeChange(): void {
    if (preferences.value.theme === "system") applyTheme();
  }

  function startSystemThemeListener(): void {
    systemTheme.addEventListener("change", handleSystemThemeChange);
  }

  function stopSystemThemeListener(): void {
    systemTheme.removeEventListener("change", handleSystemThemeChange);
  }

  return {
    preferences,
    load,
    setTheme,
    changeLocale,
    updatePersonalization,
    updateApplicationSettings,
    restartForMaterialChange,
    updateConnectionSettings,
    updateLightweightSettings,
    setHostUriLifetime,
    startSystemThemeListener,
    stopSystemThemeListener,
  };
});
