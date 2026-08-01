import { defineStore } from "pinia";
import { ref } from "vue";
import * as preferencesApi from "@api";
import { setLocale } from "../i18n";
import type {
  ConnectionSettingsUpdate,
  CloseAction,
  HostUriLifetime,
  Locale,
  PersonalizationUpdate,
  Preferences,
  ThemePreference,
} from "../models/preferences";
import { applyColorTheme } from "../themes/apply";
import { applyTypography, DEFAULT_FONT_SIZE } from "../ui/typography";

const defaults: Preferences = {
  theme: "system",
  colorTheme: "celadon",
  fontSize: DEFAULT_FONT_SIZE,
  fontFamily: "",
  splashDurationMs: 1000,
  locale: "zh-CN",
  rememberWindowState: true,
  closeAction: "ask",
  hostUriLifetime: "always",
  joinUri: "",
  joinPort: 25565,
  reconnectTimeoutSecs: null,
  relayCustom: false,
  relayUrl: "",
};

export const usePreferencesStore = defineStore("preferences", () => {
  const preferences = ref<Preferences>({ ...defaults });
  const systemTheme = window.matchMedia("(prefers-color-scheme: dark)");
  let themeSaveQueue = Promise.resolve();
  let localeSaveQueue = Promise.resolve();
  let personalizationSaveQueue = Promise.resolve();
  let connectionSaveQueue = Promise.resolve();

  function applyTheme(): void {
    applyColorTheme(preferences.value.theme, preferences.value.colorTheme, systemTheme.matches);
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
    Object.assign(preferences.value, snapshot);
    setLocale(snapshot.locale);
    applyPersonalizationStyles();
    personalizationSaveQueue = personalizationSaveQueue
      .then(() => preferencesApi.savePersonalization(snapshot))
      .catch((error) => console.error("Failed to save personalization", error));
  }

  function updateConnectionSettings(update: ConnectionSettingsUpdate): void {
    const snapshot = { ...update };
    Object.assign(preferences.value, snapshot);
    connectionSaveQueue = connectionSaveQueue
      .then(() => preferencesApi.saveConnectionSettings(snapshot))
      .catch((error) => console.error("Failed to save connection settings", error));
  }

  async function setCloseAction(closeAction: CloseAction): Promise<boolean> {
    try {
      await preferencesApi.saveCloseAction(closeAction);
      preferences.value.closeAction = closeAction;
      return true;
    } catch (error) {
      console.error("Failed to save close action", error);
      return false;
    }
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
    updateConnectionSettings,
    setCloseAction,
    setHostUriLifetime,
    startSystemThemeListener,
    stopSystemThemeListener,
  };
});
