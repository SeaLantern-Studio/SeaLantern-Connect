import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { ref } from "vue";
import { setLocale } from "../i18n";
import type {
  ConnectionSettingsUpdate,
  CloseAction,
  HostUriLifetime,
  Locale,
  PersonalizationUpdate,
  Preferences,
  ThemePreference,
} from "../preferences";
import type { ColorThemeId } from "../themes";
import { applyColorTheme } from "../themes/apply";
import { applyTypography, DEFAULT_FONT_SIZE } from "../typography";

const defaults: Preferences = {
  theme: "system",
  colorTheme: "default",
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
  let colorThemeSaveQueue = Promise.resolve();
  let localeSaveQueue = Promise.resolve();

  function applyTheme(): void {
    applyColorTheme(preferences.value.theme, preferences.value.colorTheme, systemTheme.matches);
  }

  function applyPersonalizationStyles(): void {
    applyTheme();
    applyTypography(preferences.value.fontSize, preferences.value.fontFamily);
  }

  async function load(): Promise<void> {
    try {
      preferences.value = await invoke<Preferences>("get_preferences");
      setLocale(preferences.value.locale);
    } catch (error) {
      console.error("Failed to load preferences", error);
    }
    applyPersonalizationStyles();
  }

  async function saveTheme(theme: ThemePreference, fallback: ThemePreference): Promise<void> {
    try {
      await invoke("set_theme", { theme });
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

  async function saveColorTheme(colorTheme: ColorThemeId, fallback: ColorThemeId): Promise<void> {
    try {
      await invoke("set_color_theme", { colorTheme });
    } catch (error) {
      if (preferences.value.colorTheme === colorTheme) {
        preferences.value.colorTheme = fallback;
        applyTheme();
      }
      console.error("Failed to save color theme", error);
    }
  }

  function setColorTheme(colorTheme: ColorThemeId): void {
    if (preferences.value.colorTheme === colorTheme) return;
    const fallback = preferences.value.colorTheme;
    preferences.value.colorTheme = colorTheme;
    applyTheme();
    colorThemeSaveQueue = colorThemeSaveQueue.then(() => saveColorTheme(colorTheme, fallback));
  }

  async function saveLocale(locale: Locale, fallback: Locale): Promise<void> {
    try {
      await invoke("set_locale", { locale });
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

  function applyPersonalization(update: PersonalizationUpdate): void {
    Object.assign(preferences.value, update);
    setLocale(update.locale);
    applyPersonalizationStyles();
  }

  function applyConnectionSettings(update: ConnectionSettingsUpdate): void {
    Object.assign(preferences.value, update);
  }

  async function setCloseAction(closeAction: CloseAction): Promise<boolean> {
    try {
      await invoke("set_close_action", { closeAction });
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
      await invoke("set_host_uri_lifetime", { lifetime });
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
    setColorTheme,
    changeLocale,
    applyPersonalization,
    applyConnectionSettings,
    setCloseAction,
    setHostUriLifetime,
    startSystemThemeListener,
    stopSystemThemeListener,
  };
});
