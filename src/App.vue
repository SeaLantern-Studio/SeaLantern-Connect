<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import SplashScreen from "./components/SplashScreen.vue";
import AppHeader from "./components/layout/AppHeader.vue";
import AppSidebar from "./components/layout/AppSidebar.vue";
import HostView from "./components/rooms/HostView.vue";
import JoinView from "./components/rooms/JoinView.vue";
import PersonalizationView from "./components/settings/PersonalizationView.vue";
import SettingsView from "./components/settings/SettingsView.vue";
import { emptyConnectStatus, type ConnectStatus } from "./connect";
import { setLocale, t } from "./i18n";
import type {
  ConnectionSettingsUpdate,
  PersonalizationUpdate,
  Preferences,
  ThemePreference,
} from "./preferences";

type SectionId = "create" | "join" | "personalize" | "settings";

const status = ref<ConnectStatus>(emptyConnectStatus);
const showSplash = ref(true);
const isInitializing = ref(true);
const dark = ref(window.matchMedia("(prefers-color-scheme: dark)").matches);
const themePreference = ref<ThemePreference>("system");
const preferences = ref<Preferences>({
  theme: "system",
  locale: "zh-CN",
  rememberWindowState: true,
  closeAction: "hide_to_tray",
  joinUri: "",
  joinPort: 25565,
  reconnectTimeoutSecs: null,
  relayCustom: false,
  relayUrl: "",
});
const activeSection = ref<SectionId>("join");
const sidebarCollapsed = ref(false);
const systemTheme = window.matchMedia("(prefers-color-scheme: dark)");
let unlisten: UnlistenFn | null = null;

const busy = computed(() => status.value.phase === "starting" || status.value.phase === "stopping");
const connected = computed(() => status.value.phase === "active");
const pageTitle = computed(
  () =>
    ({
      create: t("navigation.createRoom"),
      join: t("navigation.joinRoom"),
      personalize: t("navigation.personalization"),
      settings: t("navigation.settings"),
    })[activeSection.value],
);
const connectionState = computed(() => {
  if (connected.value) return "active";
  if (busy.value) return "busy";
  return "idle";
});

function applyTheme() {
  dark.value =
    themePreference.value === "system" ? systemTheme.matches : themePreference.value === "dark";
  document.documentElement.dataset.theme = dark.value ? "dark" : "light";
}

async function toggleTheme() {
  themePreference.value = dark.value ? "light" : "dark";
  applyTheme();
  try {
    await invoke("set_theme", { theme: themePreference.value });
    preferences.value.theme = themePreference.value;
  } catch (error) {
    console.error("Failed to save theme preference", error);
  }
}

function applyPersonalization(update: PersonalizationUpdate) {
  Object.assign(preferences.value, update);
  themePreference.value = update.theme;
  setLocale(update.locale);
  applyTheme();
}

function applyConnectionSettings(update: ConnectionSettingsUpdate) {
  Object.assign(preferences.value, update);
}

function handleSystemThemeChange() {
  if (themePreference.value === "system") applyTheme();
}

function navigate(section: string) {
  if (["create", "join", "personalize", "settings"].includes(section)) {
    activeSection.value = section as SectionId;
  }
}

onMounted(async () => {
  try {
    preferences.value = await invoke<Preferences>("get_preferences");
    themePreference.value = preferences.value.theme;
    setLocale(preferences.value.locale);
  } catch (error) {
    console.error("Failed to load preferences", error);
  }
  applyTheme();
  systemTheme.addEventListener("change", handleSystemThemeChange);

  try {
    status.value = await invoke<ConnectStatus>("get_status");
    unlisten = await listen<ConnectStatus>("connect-status", (event) => {
      status.value = event.payload;
    });
  } finally {
    isInitializing.value = false;
  }
});

onUnmounted(() => {
  unlisten?.();
  systemTheme.removeEventListener("change", handleSystemThemeChange);
});
</script>

<template>
  <Transition name="splash-fade">
    <SplashScreen v-if="showSplash" :loading="isInitializing" @ready="showSplash = false" />
  </Transition>

  <div v-if="!showSplash" class="app-shell" :class="{ 'sidebar-collapsed': sidebarCollapsed }">
    <AppSidebar
      :active="activeSection"
      :connection-state="connectionState"
      :tunnel-mode="status.mode"
      :collapsed="sidebarCollapsed"
      @navigate="navigate"
      @toggle-collapse="sidebarCollapsed = !sidebarCollapsed"
    />
    <AppHeader :title="pageTitle" :dark="dark" @toggle-theme="toggleTheme" />

    <main class="app-content">
      <Transition name="page" mode="out-in">
        <div :key="activeSection" class="page-transition-frame">
          <HostView v-if="activeSection === 'create'" :status="status" />
          <JoinView
            v-else-if="activeSection === 'join'"
            :status="status"
            :saved-invite="preferences.joinUri"
            :saved-port="preferences.joinPort"
          />
          <PersonalizationView
            v-else-if="activeSection === 'personalize'"
            :preferences="preferences"
            @saved="applyPersonalization"
          />
          <SettingsView v-else :preferences="preferences" @saved="applyConnectionSettings" />
        </div>
      </Transition>
    </main>
  </div>
</template>
