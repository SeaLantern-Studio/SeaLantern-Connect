import { getThemeColors, type ColorThemeId } from ".";
import type { ThemePreference } from "../preferences";

function adjustBrightness(hex: string, percent: number): string {
  const value = Number.parseInt(hex.slice(1), 16);
  const offset = Math.round(2.55 * percent);
  const red = Math.min(255, Math.max(0, (value >> 16) + offset));
  const green = Math.min(255, Math.max(0, ((value >> 8) & 0xff) + offset));
  const blue = Math.min(255, Math.max(0, (value & 0xff) + offset));
  return `#${((1 << 24) | (red << 16) | (green << 8) | blue).toString(16).slice(1)}`;
}

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
  root.style.setProperty("--surface", dark ? colors.bgSecondary : "#ffffff");
  root.style.setProperty("--surface-soft", colors.bgSecondary);
  root.style.setProperty("--surface-strong", colors.bgTertiary);
  root.style.setProperty("--primary", colors.primary);
  root.style.setProperty("--primary-hover", adjustBrightness(colors.primary, dark ? -20 : -30));
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
