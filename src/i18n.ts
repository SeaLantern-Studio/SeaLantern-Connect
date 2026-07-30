import { ref } from "vue";
import en from "./language/en.json";
import zhCN from "./language/zh-CN.json";
import type { Locale } from "./preferences";

type TranslationNode = string | { [key: string]: TranslationNode };

const translations: Record<Locale, TranslationNode> = {
  "zh-CN": zhCN,
  en,
};

export const locale = ref<Locale>("zh-CN");

export function setLocale(value: Locale) {
  locale.value = value;
  document.documentElement.lang = value;
}

function resolve(node: TranslationNode, key: string): string | undefined {
  let current: TranslationNode | undefined = node;
  for (const segment of key.split(".")) {
    if (typeof current === "string") return undefined;
    current = current[segment];
    if (current === undefined) return undefined;
  }
  return typeof current === "string" ? current : undefined;
}

export function t(key: string, params: Record<string, string | number> = {}) {
  let value = resolve(translations[locale.value], key) ?? resolve(translations.en, key) ?? key;
  for (const [name, replacement] of Object.entries(params)) {
    value = value.split(`{${name}}`).join(String(replacement));
  }
  return value;
}
