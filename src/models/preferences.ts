export type ThemePreference = "system" | "light" | "dark";
export type ColorThemeId = "celadon" | "inkstone" | "vellum" | "moss" | "gloaming";
export type SplashDurationMs = 0 | 500 | 1000 | 1500 | 2000;
export type Locale = "zh-CN" | "en";
export type CloseAction = "ask" | "exit" | "hide_to_tray";
export type HostUriLifetime = "always" | "never" | "1h" | "3h" | "6h" | "12h" | "24h";

export interface Preferences {
  theme: ThemePreference;
  colorTheme: ColorThemeId;
  fontSize: number;
  fontFamily: string;
  splashDurationMs: SplashDurationMs;
  locale: Locale;
  rememberWindowState: boolean;
  closeAction: CloseAction;
  autoLightweightMinutes: number | null;
  hostUriLifetime: HostUriLifetime;
  joinUri: string;
  joinPort: number;
  reconnectTimeoutSecs: number | null;
  relayCustom: boolean;
  relayUrl: string;
}

export interface PersonalizationUpdate {
  theme: ThemePreference;
  colorTheme: ColorThemeId;
  fontSize: number;
  fontFamily: string;
  splashDurationMs: SplashDurationMs;
  locale: Locale;
  rememberWindowState: boolean;
  closeAction: CloseAction;
}

export interface ConnectionSettingsUpdate {
  relayCustom: boolean;
  relayUrl: string;
  reconnectTimeoutSecs: number | null;
}

export interface LightweightSettingsUpdate {
  autoLightweightMinutes: number | null;
}
