import type { ThemeDefinition } from "./types";

const warmTheme: ThemeDefinition = {
  id: "warm",
  name: "Vellum",
  description: "Fiber-warm paper surfaces with a soft umber accent",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#fbf8f2",
    bgSecondary: "#f5eee2",
    bgTertiary: "#e8dcc9",
    primary: "#80664d",
    secondary: "#9b7d5e",
    textPrimary: "#282522",
    textSecondary: "#6f6961",
    border: "#dfd2c0",
  },
  dark: {
    bg: "#161412",
    bgSecondary: "#1f1c19",
    bgTertiary: "#2c2823",
    primary: "#ceb99f",
    secondary: "#decab2",
    textPrimary: "#f4f1ec",
    textSecondary: "#aca49a",
    border: "rgba(206, 185, 159, 0.14)",
  },
  lightAcrylic: {
    bg: "rgba(251, 248, 242, 0.65)",
    bgSecondary: "rgba(245, 238, 226, 0.55)",
    bgTertiary: "rgba(232, 220, 201, 0.45)",
    primary: "#80664d",
    secondary: "#9b7d5e",
    textPrimary: "#282522",
    textSecondary: "#6f6961",
    border: "rgba(223, 210, 192, 0.65)",
  },
  darkAcrylic: {
    bg: "rgba(22, 20, 18, 0.65)",
    bgSecondary: "rgba(31, 28, 25, 0.55)",
    bgTertiary: "rgba(44, 40, 35, 0.45)",
    primary: "#ceb99f",
    secondary: "#decab2",
    textPrimary: "#f4f1ec",
    textSecondary: "#aca49a",
    border: "rgba(206, 185, 159, 0.11)",
  },
};

export default warmTheme;
