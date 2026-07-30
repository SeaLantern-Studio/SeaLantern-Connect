<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Moon, Square, Sun, X } from "lucide-vue-next";

defineProps<{
  title: string;
  dark: boolean;
}>();

defineEmits<{
  toggleTheme: [];
}>();

const appWindow = getCurrentWindow();
const isMacOS = /Macintosh|Mac OS X/i.test(navigator.userAgent);
const isMaximized = ref(false);
let unlistenResize: (() => void) | null = null;

async function minimizeWindow() {
  await appWindow.minimize();
}

async function toggleMaximize() {
  await appWindow.toggleMaximize();
}

async function closeWindow() {
  await appWindow.close();
}

onMounted(async () => {
  isMaximized.value = await appWindow.isMaximized();
  unlistenResize = await appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized();
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
      <button class="icon-button" type="button" title="切换明暗主题" @click="$emit('toggleTheme')">
        <Sun v-if="dark" :size="16" />
        <Moon v-else :size="16" />
      </button>
      <div v-if="!isMacOS" class="window-controls">
        <button class="window-button" type="button" title="最小化" @click="minimizeWindow">
          <Minus :size="12" />
        </button>
        <button
          class="window-button"
          type="button"
          :title="isMaximized ? '还原' : '最大化'"
          @click="toggleMaximize"
        >
          <Copy v-if="isMaximized" :size="12" />
          <Square v-else :size="12" />
        </button>
        <button
          class="window-button window-button-close"
          type="button"
          title="关闭"
          @click="closeWindow"
        >
          <X :size="12" />
        </button>
      </div>
    </div>
  </header>
</template>
