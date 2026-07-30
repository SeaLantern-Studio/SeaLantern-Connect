import defaultTheme from "./default";
import midnightTheme from "./midnight";
import oceanTheme from "./ocean";
import roseTheme from "./rose";
import sunsetTheme from "./sunset";
import type { ColorPlan, ColorThemeId, ThemeColors, ThemeDefinition } from "./types";

const themes: Record<ColorThemeId, ThemeDefinition> = {
  default: defaultTheme,
  midnight: midnightTheme,
  ocean: oceanTheme,
  rose: roseTheme,
  sunset: sunsetTheme,
};

export function getThemeOptions(): Array<{ label: string; value: ColorThemeId }> {
  return Object.values(themes).map((theme) => ({ label: theme.name, value: theme.id }));
}

export function getThemeColors(themeId: ColorThemeId, plan: ColorPlan): ThemeColors {
  return themes[themeId][plan];
}

export type { ColorPlan, ColorThemeId, ThemeColors, ThemeDefinition } from "./types";
