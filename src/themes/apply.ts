import { getThemeColors, type ColorThemeId } from ".";
import type { ThemePreference } from "../models/preferences";

function rgba(hex: string, alpha: number): string {
  const value = Number.parseInt(hex.slice(1), 16);
  return `rgba(${value >> 16}, ${(value >> 8) & 0xff}, ${value & 0xff}, ${alpha})`;
}

export function applyColorTheme(
  preference: ThemePreference,
  colorTheme: ColorThemeId,
  systemDark: boolean,
): void {
  const dark = preference === "system" ? systemDark : preference === "dark";
  const colors = getThemeColors(colorTheme, dark ? "dark" : "light");
  const root = document.documentElement;

  root.dataset.theme = dark ? "dark" : "light";
  root.style.setProperty("--background", colors.bg);
  root.style.setProperty("--surface", colors.bgSecondary);
  root.style.setProperty("--surface-soft", colors.bgSecondary);
  root.style.setProperty("--surface-strong", colors.bgTertiary);
  root.style.setProperty("--primary", colors.primary);
  root.style.setProperty(
    "--primary-hover",
    `color-mix(in srgb, ${colors.primary} 82%, ${colors.textPrimary})`,
  );
  root.style.setProperty("--primary-solid", colors.primarySolid);
  root.style.setProperty("--primary-solid-hover", colors.primarySolidHover);
  root.style.setProperty("--accent", colors.secondary);
  root.style.setProperty("--text", colors.textPrimary);
  root.style.setProperty("--muted", colors.textSecondary);
  root.style.setProperty("--border", colors.border);
  root.style.setProperty("--cmz-primary-bg", rgba(colors.primary, dark ? 0.12 : 0.08));
  root.style.setProperty(
    "--cmz-glass-bg",
    dark ? "rgba(15, 17, 23, 0.65)" : "rgba(255, 255, 255, 0.65)",
  );
  root.style.setProperty(
    "--cmz-acrylic-bg",
    dark ? "rgba(15, 17, 23, 0.75)" : "rgba(255, 255, 255, 0.75)",
  );
  root.style.setProperty(
    "--shadow",
    dark ? "0 16px 48px rgba(0, 0, 0, 0.6)" : "0 16px 48px rgba(0, 0, 0, 0.1)",
  );
  root.style.setProperty(
    "--card-shadow",
    dark
      ? "0 1px 4px rgba(0, 0, 0, 0.3), 0 4px 12px rgba(0, 0, 0, 0.4)"
      : "0 1px 4px rgba(0, 0, 0, 0.04), 0 4px 12px rgba(0, 0, 0, 0.06)",
  );
}
