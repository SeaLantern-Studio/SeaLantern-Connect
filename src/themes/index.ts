import celadonTheme from "./celadon";
import gloamingTheme from "./gloaming";
import inkstoneTheme from "./inkstone";
import mossTheme from "./moss";
import vellumTheme from "./vellum";
import type { ColorPlan, ColorThemeId, ThemeColors, ThemeDefinition } from "./types";

type PresetColorThemeId = Exclude<ColorThemeId, "custom">;

const themes: Record<PresetColorThemeId, ThemeDefinition> = {
  inkstone: inkstoneTheme,
  celadon: celadonTheme,
  vellum: vellumTheme,
  moss: mossTheme,
  gloaming: gloamingTheme,
};

export function getThemeOptions(): Array<{ label: string; value: ColorThemeId }> {
  return [
    ...Object.values(themes).map((theme) => ({ label: theme.name, value: theme.id })),
    { label: "Custom", value: "custom" },
  ];
}

export function getThemeColors(themeId: ColorThemeId, plan: ColorPlan): ThemeColors {
  const preset = themeId === "custom" ? themes.inkstone : themes[themeId];
  return (preset ?? themes.inkstone)[plan];
}

export type { ColorPlan, ColorThemeId, ThemeColors, ThemeDefinition } from "./types";
