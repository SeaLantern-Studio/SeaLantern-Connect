import type { ThemeDefinition } from "./types";

const mauveTheme: ThemeDefinition = {
  id: "mauve",
  name: "Gloaming",
  description: "Dusk-gray surfaces with a fading violet accent",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#f9f7fa",
    bgSecondary: "#f1ecf3",
    bgTertiary: "#e2d9e5",
    primary: "#775f80",
    secondary: "#987f9e",
    textPrimary: "#2b272d",
    textSecondary: "#706973",
    border: "#d9d0dc",
  },
  dark: {
    bg: "#141116",
    bgSecondary: "#1d1920",
    bgTertiary: "#2a2430",
    primary: "#bea0c7",
    secondary: "#d0b5d5",
    textPrimary: "#f4f1f5",
    textSecondary: "#aaa2ac",
    border: "rgba(190, 160, 199, 0.14)",
  },
  lightAcrylic: {
    bg: "rgba(249, 247, 250, 0.65)",
    bgSecondary: "rgba(241, 236, 243, 0.55)",
    bgTertiary: "rgba(226, 217, 229, 0.45)",
    primary: "#775f80",
    secondary: "#987f9e",
    textPrimary: "#2b272d",
    textSecondary: "#706973",
    border: "rgba(217, 208, 220, 0.65)",
  },
  darkAcrylic: {
    bg: "rgba(20, 17, 22, 0.65)",
    bgSecondary: "rgba(29, 25, 32, 0.55)",
    bgTertiary: "rgba(42, 36, 48, 0.45)",
    primary: "#bea0c7",
    secondary: "#d0b5d5",
    textPrimary: "#f4f1f5",
    textSecondary: "#aaa2ac",
    border: "rgba(190, 160, 199, 0.11)",
  },
};

export default mauveTheme;
