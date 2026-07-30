import type { ColorThemeId } from "./themes";

export type ThemePreference = "system" | "light" | "dark";
export type SplashDurationMs = 0 | 500 | 1000 | 1500 | 2000;
export type Locale = "zh-CN" | "en";
export type CloseAction = "ask" | "exit" | "hide_to_tray";

export interface Preferences {
  theme: ThemePreference;
  colorTheme: ColorThemeId;
  splashDurationMs: SplashDurationMs;
  locale: Locale;
  rememberWindowState: boolean;
  closeAction: CloseAction;
  joinUri: string;
  joinPort: number;
  reconnectTimeoutSecs: number | null;
  relayCustom: boolean;
  relayUrl: string;
}

export interface PersonalizationUpdate {
  theme: ThemePreference;
  colorTheme: ColorThemeId;
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
