<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Check, Copy, Languages, Minus, Monitor, Moon, Square, Sun, X } from "lucide-vue-next";
import { t } from "../../i18n";
import type { Locale, ThemePreference } from "../../preferences";

const props = defineProps<{
  title: string;
  theme: ThemePreference;
  locale: Locale;
}>();

defineEmits<{
  changeLocale: [locale: Locale];
  changeTheme: [theme: ThemePreference];
}>();

const appWindow = getCurrentWindow();
const isMacOS = /Macintosh|Mac OS X/i.test(navigator.userAgent);
const isMaximized = ref(false);
const languageMenuOpen = ref(false);
const languageSelector = ref<HTMLElement | null>(null);
let unlistenResize: (() => void) | null = null;

const languageOptions = computed<{ key: Locale; label: string }[]>(() => [
  { key: "zh-CN", label: t("personalization.simplifiedChinese") },
  { key: "en", label: t("personalization.english") },
]);

const themeIndicatorOffset = computed(() => {
  const index = ["system", "light", "dark"].indexOf(props.theme);
  return Math.max(index, 0) * 26;
});

async function minimizeWindow() {
  await appWindow.minimize();
}

async function toggleMaximize() {
  await appWindow.toggleMaximize();
}

async function closeWindow() {
  await appWindow.close();
}

function closeLanguageMenu() {
  languageMenuOpen.value = false;
}

function handleDocumentPointerDown(event: PointerEvent) {
  if (!languageSelector.value?.contains(event.target as Node)) closeLanguageMenu();
}

onMounted(async () => {
  document.addEventListener("pointerdown", handleDocumentPointerDown);
  isMaximized.value = await appWindow.isMaximized();
  unlistenResize = await appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized();
  });
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", handleDocumentPointerDown);
  unlistenResize?.();
});
</script>

<template>
  <header class="titlebar" :class="{ 'macos-overlay': isMacOS }" data-tauri-drag-region>
    <h1 class="page-title" data-tauri-drag-region>{{ title }}</h1>
    <div class="titlebar-actions">
      <div ref="languageSelector" class="header-language-selector">
        <button
          class="header-language-button"
          type="button"
          :title="t('personalization.language')"
          aria-haspopup="menu"
          :aria-expanded="languageMenuOpen"
          @click="languageMenuOpen = !languageMenuOpen"
        >
          <Languages :size="16" />
        </button>
        <div
          v-if="languageMenuOpen"
          class="header-language-menu"
          role="menu"
          @keydown.esc.stop="closeLanguageMenu"
        >
          <button
            v-for="option in languageOptions"
            :key="option.key"
            class="header-language-item"
            :class="{ active: locale === option.key }"
            type="button"
            role="menuitemradio"
            :aria-checked="locale === option.key"
            @click="$emit('changeLocale', option.key); closeLanguageMenu()"
          >
            <span class="header-language-label">{{ option.label }}</span>
            <Check v-if="locale === option.key" :size="16" aria-hidden="true" />
          </button>
        </div>
      </div>
      <div class="theme-switcher" role="group" :aria-label="t('personalization.theme')">
        <div
          class="theme-indicator"
          :style="{ transform: `translateX(${themeIndicatorOffset}px)` }"
        />
        <button
          class="theme-button"
          :class="{ active: theme === 'system' }"
          type="button"
          :title="t('personalization.followSystem')"
          :aria-pressed="theme === 'system'"
          @click="$emit('changeTheme', 'system')"
        >
          <Monitor :size="16" />
        </button>
        <button
          class="theme-button"
          :class="{ active: theme === 'light' }"
          type="button"
          :title="t('personalization.light')"
          :aria-pressed="theme === 'light'"
          @click="$emit('changeTheme', 'light')"
        >
          <Sun :size="16" />
        </button>
        <button
          class="theme-button"
          :class="{ active: theme === 'dark' }"
          type="button"
          :title="t('personalization.dark')"
          :aria-pressed="theme === 'dark'"
          @click="$emit('changeTheme', 'dark')"
        >
          <Moon :size="16" />
        </button>
      </div>
      <div v-if="!isMacOS" class="window-controls">
        <button
          class="window-button"
          type="button"
          :title="t('window.minimize')"
          @click="minimizeWindow"
        >
          <Minus :size="12" />
        </button>
        <button
          class="window-button"
          type="button"
          :title="isMaximized ? t('window.restore') : t('window.maximize')"
          @click="toggleMaximize"
        >
          <Copy v-if="isMaximized" :size="12" />
          <Square v-else :size="12" />
        </button>
        <button
          class="window-button window-button-close"
          type="button"
          :title="t('window.close')"
          @click="closeWindow"
        >
          <X :size="12" />
        </button>
      </div>
    </div>
  </header>
</template>
