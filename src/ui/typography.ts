export const DEFAULT_FONT_SIZE = 14;
export const MIN_FONT_SIZE = 12;
export const MAX_FONT_SIZE = 20;

const DEFAULT_FONT_FAMILY = 'Inter, "PingFang SC", "Microsoft YaHei", sans-serif';

export function applyTypography(fontSize: number, fontFamily: string): void {
  const size = Math.min(MAX_FONT_SIZE, Math.max(MIN_FONT_SIZE, fontSize || DEFAULT_FONT_SIZE));
  const family = fontFamily.trim();
  const escapedFamily = family.replace(/\\/g, "\\\\").replace(/"/g, '\\"');

  document.documentElement.style.fontSize = `${size}px`;
  document.documentElement.style.setProperty(
    "--app-font-family",
    family ? `"${escapedFamily}", ${DEFAULT_FONT_FAMILY}` : DEFAULT_FONT_FAMILY,
  );
}
