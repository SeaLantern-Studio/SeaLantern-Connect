import { getThemeColors, type ColorThemeId, type ThemeColors } from ".";
import type { ThemePreference, WindowMaterial } from "../models/preferences";

const nativeMaterialColors: Record<"light" | "dark", ThemeColors> = {
  light: {
    bg: "#f3f3f3",
    bgSecondary: "#f9f9f9",
    bgTertiary: "#ffffff",
    primary: "#0067c0",
    primarySolid: "#0078d4",
    primarySolidHover: "#005fb8",
    secondary: "#5f6368",
    textPrimary: "#1b1b1b",
    textSecondary: "#5f5f5f",
    border: "#d6d6d6",
  },
  dark: {
    bg: "#202020",
    bgSecondary: "#292929",
    bgTertiary: "#333333",
    primary: "#60cdff",
    primarySolid: "#0078d4",
    primarySolidHover: "#1a86d9",
    secondary: "#c5c5c5",
    textPrimary: "#f5f5f5",
    textSecondary: "#c4c4c4",
    border: "#454545",
  },
};

function rgba(hex: string, alpha: number): string {
  const value = Number.parseInt(hex.slice(1), 16);
  return `rgba(${value >> 16}, ${(value >> 8) & 0xff}, ${value & 0xff}, ${alpha})`;
}

function materialOpacity(
  material: WindowMaterial,
  dark: boolean,
): {
  surface: number;
  soft: number;
  strong: number;
} {
  switch (material) {
    case "mica":
      return { surface: 0.3, soft: 0.46, strong: 0.38 };
    case "acrylic":
      return dark
        ? { surface: 0.26, soft: 0.42, strong: 0.34 }
        : { surface: 0.18, soft: 0.26, strong: 0.22 };
    case "vibrancy":
      return { surface: 0.14, soft: 0.08, strong: 0.18 };
    default:
      return { surface: 1, soft: 1, strong: 1 };
  }
}

export function applyColorTheme(
  preference: ThemePreference,
  colorTheme: ColorThemeId,
  systemDark: boolean,
  windowMaterial: WindowMaterial,
): void {
  const usesMaterialPalette = windowMaterial !== "solid";
  const dark = preference === "system" ? systemDark : preference === "dark";
  const colors = usesMaterialPalette
    ? nativeMaterialColors[dark ? "dark" : "light"]
    : getThemeColors(colorTheme, dark ? "dark" : "light");
  const opacity = materialOpacity(windowMaterial, dark);
  const root = document.documentElement;
  const nativeMaterial =
    windowMaterial !== "solid" && /Windows|Macintosh|Mac OS X/i.test(navigator.userAgent);

  root.dataset.theme = dark ? "dark" : "light";
  root.dataset.windowMaterial = windowMaterial;
  root.toggleAttribute("data-native-material", nativeMaterial);
  root.style.setProperty("--background", nativeMaterial ? "transparent" : colors.bg);
  root.style.setProperty(
    "--surface",
    nativeMaterial ? rgba(colors.bgSecondary, opacity.surface) : colors.bgSecondary,
  );
  root.style.setProperty(
    "--surface-soft",
    nativeMaterial ? rgba(colors.bg, opacity.soft) : colors.bgSecondary,
  );
  root.style.setProperty(
    "--surface-strong",
    nativeMaterial ? rgba(colors.bgTertiary, opacity.strong) : colors.bgTertiary,
  );
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
    nativeMaterial
      ? rgba(colors.bgSecondary, Math.max(opacity.surface - 0.04, 0.14))
      : dark
        ? "rgba(15, 17, 23, 0.65)"
        : "rgba(255, 255, 255, 0.65)",
  );
  root.style.setProperty(
    "--cmz-acrylic-bg",
    nativeMaterial
      ? rgba(colors.bgTertiary, opacity.strong)
      : dark
        ? "rgba(15, 17, 23, 0.75)"
        : "rgba(255, 255, 255, 0.75)",
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
