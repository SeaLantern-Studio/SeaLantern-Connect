import type { ThemeDefinition } from "./types";

const sageTheme: ThemeDefinition = {
  id: "sage",
  name: "Mountain Haze",
  description: "Mist-washed blue-green surfaces with a distant pine accent",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#f5f8f7",
    bgSecondary: "#eaf2f0",
    bgTertiary: "#d5e3df",
    primary: "#52786f",
    secondary: "#728f88",
    textPrimary: "#26302e",
    textSecondary: "#687572",
    border: "#ceddd9",
  },
  dark: {
    bg: "#0f1514",
    bgSecondary: "#17201e",
    bgTertiary: "#22302d",
    primary: "#8ebab0",
    secondary: "#a9c8c0",
    textPrimary: "#f0f4f1",
    textSecondary: "#a2ada5",
    border: "rgba(142, 186, 176, 0.14)",
  },
  lightAcrylic: {
    bg: "rgba(245, 248, 247, 0.65)",
    bgSecondary: "rgba(234, 242, 240, 0.55)",
    bgTertiary: "rgba(213, 227, 223, 0.45)",
    primary: "#52786f",
    secondary: "#728f88",
    textPrimary: "#26302e",
    textSecondary: "#687572",
    border: "rgba(206, 221, 217, 0.65)",
  },
  darkAcrylic: {
    bg: "rgba(15, 21, 20, 0.65)",
    bgSecondary: "rgba(23, 32, 30, 0.55)",
    bgTertiary: "rgba(34, 48, 45, 0.45)",
    primary: "#8ebab0",
    secondary: "#a9c8c0",
    textPrimary: "#f0f4f1",
    textSecondary: "#a2ada5",
    border: "rgba(142, 186, 176, 0.11)",
  },
};

export default sageTheme;
