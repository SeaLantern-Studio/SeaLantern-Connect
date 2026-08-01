import celadonTheme from "./celadon";
import gloamingTheme from "./gloaming";
import inkstoneTheme from "./inkstone";
import mossTheme from "./moss";
import vellumTheme from "./vellum";
import type { ColorPlan, ColorThemeId, ThemeColors, ThemeDefinition } from "./types";

const themes: Record<ColorThemeId, ThemeDefinition> = {
  celadon: celadonTheme,
  inkstone: inkstoneTheme,
  vellum: vellumTheme,
  moss: mossTheme,
  gloaming: gloamingTheme,
};

export function getThemeOptions(): Array<{ label: string; value: ColorThemeId }> {
  return Object.values(themes).map((theme) => ({ label: theme.name, value: theme.id }));
}

export function getThemeColors(themeId: ColorThemeId, plan: ColorPlan): ThemeColors {
  return (themes[themeId] ?? themes.celadon)[plan];
}

export type { ColorPlan, ColorThemeId, ThemeColors, ThemeDefinition } from "./types";
