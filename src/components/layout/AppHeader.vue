<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { Copy, Minus, Monitor, Moon, Square, Sun, X } from "@lucide/vue";
import {
  closeWindow,
  isWindowMaximized,
  minimizeWindow,
  onWindowResized,
  toggleMaximize,
} from "@api";
import { t } from "../../i18n";
import type { Locale, ThemePreference } from "../../models/preferences";

const props = defineProps<{
  title: string;
  theme: ThemePreference;
  locale: Locale;
}>();

defineEmits<{
  changeLocale: [locale: Locale];
  changeTheme: [theme: ThemePreference];
}>();

const isMacOS = /Macintosh|Mac OS X/i.test(navigator.userAgent);
const isMaximized = ref(false);
let unlistenResize: (() => void) | null = null;

const languageIndicator = computed(() => (props.locale === "zh-CN" ? "中" : "EN"));
const languageSwitchTitle = computed(() =>
  props.locale === "zh-CN" ? t("personalization.english") : t("personalization.simplifiedChinese"),
);

const themeIndicatorOffset = computed(() => {
  const index = ["system", "light", "dark"].indexOf(props.theme);
  return Math.max(index, 0) * 26;
});

onMounted(async () => {
  isMaximized.value = await isWindowMaximized();
  unlistenResize = await onWindowResized(async () => {
    isMaximized.value = await isWindowMaximized();
  });
});

onUnmounted(() => {
  unlistenResize?.();
});
</script>

<template>
  <header class="titlebar" :class="{ 'macos-overlay': isMacOS }" data-tauri-drag-region>
    <h1 class="page-title" data-tauri-drag-region>{{ title }}</h1>
    <div class="titlebar-actions">
      <button
        class="header-language-button"
        type="button"
        :title="languageSwitchTitle"
        :aria-label="languageSwitchTitle"
        @click="$emit('changeLocale', locale === 'zh-CN' ? 'en' : 'zh-CN')"
      >
        <span aria-hidden="true">{{ languageIndicator }}</span>
      </button>
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
