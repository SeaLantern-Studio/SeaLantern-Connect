import defaultTheme from "./default";
import mauveTheme from "./mauve";
import neutralTheme from "./neutral";
import sageTheme from "./sage";
import warmTheme from "./warm";
import type { ColorPlan, ColorThemeId, ThemeColors, ThemeDefinition } from "./types";

const themes: Record<ColorThemeId, ThemeDefinition> = {
  default: defaultTheme,
  neutral: neutralTheme,
  warm: warmTheme,
  sage: sageTheme,
  mauve: mauveTheme,
};

export function getThemeOptions(): Array<{ label: string; value: ColorThemeId }> {
  return Object.values(themes).map((theme) => ({ label: theme.name, value: theme.id }));
}

export function getThemeColors(themeId: ColorThemeId, plan: ColorPlan): ThemeColors {
  return (themes[themeId] ?? themes.default)[plan];
}

export type { ColorPlan, ColorThemeId, ThemeColors, ThemeDefinition } from "./types";
