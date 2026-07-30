export type ColorPlan = "light" | "dark" | "lightAcrylic" | "darkAcrylic";

export type ColorThemeId = "default" | "midnight" | "ocean" | "rose" | "sunset";

export interface ThemeColors {
  bg: string;
  bgSecondary: string;
  bgTertiary: string;
  primary: string;
  secondary: string;
  textPrimary: string;
  textSecondary: string;
  border: string;
}

export interface ThemeDefinition {
  id: ColorThemeId;
  name: string;
  description: string;
  author: string;
  version: string;
  light: ThemeColors;
  dark: ThemeColors;
  lightAcrylic: ThemeColors;
  darkAcrylic: ThemeColors;
}
