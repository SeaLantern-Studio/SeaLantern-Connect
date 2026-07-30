export type ThemePreference = "system" | "light" | "dark";
export type Locale = "zh-CN" | "en";
export type CloseAction = "exit" | "hide_to_tray";

export interface Preferences {
  theme: ThemePreference;
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
  locale: Locale;
  rememberWindowState: boolean;
  closeAction: CloseAction;
}

export interface ConnectionSettingsUpdate {
  relayCustom: boolean;
  relayUrl: string;
  reconnectTimeoutSecs: number | null;
}
