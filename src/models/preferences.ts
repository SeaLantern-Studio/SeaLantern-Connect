export type ThemePreference = "system" | "light" | "dark";
export type ColorThemeId = "celadon" | "inkstone" | "vellum" | "moss" | "gloaming" | "custom";
export type SplashDurationMs = 0 | 500 | 1000 | 1500 | 2000;
export type Locale = "zh-CN" | "en";
export type WindowMaterial = "solid" | "mica" | "acrylic" | "vibrancy" | "liquid_glass";
export type HostUriLifetime = "always" | "never" | "1h" | "3h" | "6h" | "12h" | "24h";

export interface ThemeColors {
  bg: string;
  bgSecondary: string;
  bgTertiary: string;
  primary: string;
  primarySolid: string;
  primarySolidHover: string;
  secondary: string;
  textPrimary: string;
  textSecondary: string;
  border: string;
}

export interface CustomTheme {
  light: ThemeColors;
  dark: ThemeColors;
}

export interface Preferences {
  theme: ThemePreference;
  colorTheme: ColorThemeId;
  customTheme: CustomTheme;
  fontSize: number;
  fontFamily: string;
  splashDurationMs: SplashDurationMs;
  silentStart: boolean;
  locale: Locale;
  rememberWindowState: boolean;
  windowMaterial: WindowMaterial;
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
  customTheme: CustomTheme;
  fontSize: number;
  fontFamily: string;
  windowMaterial: WindowMaterial;
}

export interface ApplicationSettingsUpdate {
  splashDurationMs: SplashDurationMs;
  silentStart: boolean;
  rememberWindowState: boolean;
}

export interface ConnectionSettingsUpdate {
  relayCustom: boolean;
  relayUrl: string;
  reconnectTimeoutSecs: number | null;
}

export interface LightweightSettingsUpdate {
  autoLightweightMinutes: number | null;
}
