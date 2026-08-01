<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { storeToRefs } from "pinia";
import { LoaderCircle, Save } from "lucide-vue-next";
import AppToast from "./components/AppToast.vue";
import SplashScreen from "./components/SplashScreen.vue";
import AppHeader from "./components/layout/AppHeader.vue";
import AppSidebar from "./components/layout/AppSidebar.vue";
import HostView from "./components/rooms/HostView.vue";
import JoinView from "./components/rooms/JoinView.vue";
import PersonalizationView from "./components/settings/PersonalizationView.vue";
import SettingsView from "./components/settings/SettingsView.vue";
import { t } from "./i18n";
import type { CloseAction } from "./preferences";
import { useConnectionStore } from "./stores/connection";
import { usePreferencesStore } from "./stores/preferences";
import { useUiStore } from "./stores/ui";
import { enableAutoHidingScrollbars } from "./scrollbars";
import { inviteFromDeepLinkUrls } from "./connect";

const connectionStore = useConnectionStore();
const preferencesStore = usePreferencesStore();
const uiStore = useUiStore();
const { status, state: connectionState } = storeToRefs(connectionStore);
const { preferences } = storeToRefs(preferencesStore);
const { activeSection, sidebarCollapsed } = storeToRefs(uiStore);
const showSplash = ref(true);
const isInitializing = ref(true);
const choosingCloseAction = ref(false);
let unlistenCloseAction: UnlistenFn | null = null;
let unlistenDeepLinks: UnlistenFn | null = null;
let disableAutoHidingScrollbars: (() => void) | null = null;

const pageTitle = computed(
  () =>
    ({
      create: t("navigation.createRoom"),
      join: t("navigation.joinRoom"),
      personalize: t("navigation.personalization"),
      settings: t("navigation.settings"),
    })[activeSection.value],
);

async function chooseCloseAction(closeAction: Exclude<CloseAction, "ask">): Promise<void> {
  if (choosingCloseAction.value) return;
  choosingCloseAction.value = true;
  try {
    const saved = await preferencesStore.setCloseAction(closeAction);
    if (!saved) return;
    uiStore.closeClosePrompt();
    await getCurrentWindow().close();
  } finally {
    choosingCloseAction.value = false;
  }
}

function importDeepLink(urls: string[]): void {
  const invite = inviteFromDeepLinkUrls(urls);
  if (invite) uiStore.importInvite(invite);
}

async function setupDeepLinks(): Promise<void> {
  try {
    unlistenDeepLinks = await onOpenUrl(importDeepLink);
    importDeepLink((await getCurrent()) ?? []);
  } catch (error) {
    console.error("Failed to initialize deep links", error);
  }
}

onMounted(async () => {
  disableAutoHidingScrollbars = enableAutoHidingScrollbars();
  unlistenCloseAction = await listen("close-action-requested", uiStore.openClosePrompt);
  await setupDeepLinks();
  await preferencesStore.load();
  preferencesStore.startSystemThemeListener();
  try {
    await connectionStore.initialize();
  } finally {
    isInitializing.value = false;
  }
});

onUnmounted(() => {
  disableAutoHidingScrollbars?.();
  unlistenCloseAction?.();
  unlistenDeepLinks?.();
  connectionStore.dispose();
  preferencesStore.stopSystemThemeListener();
});
</script>

<template>
  <AppToast />

  <Transition name="splash-fade">
    <SplashScreen
      v-if="showSplash"
      :loading="isInitializing"
      :duration-ms="preferences.splashDurationMs"
      @ready="showSplash = false"
    />
  </Transition>

  <div v-if="!showSplash" class="app-shell" :class="{ 'sidebar-collapsed': sidebarCollapsed }">
    <AppSidebar
      :active="activeSection"
      :connection-state="connectionState"
      :tunnel-mode="status.mode"
      :collapsed="sidebarCollapsed"
      @navigate="uiStore.navigate"
      @toggle-collapse="uiStore.toggleSidebar"
    />
    <AppHeader
      :title="pageTitle"
      :theme="preferences.theme"
      :locale="preferences.locale"
      @change-locale="preferencesStore.changeLocale"
      @change-theme="preferencesStore.setTheme"
    />

    <main class="app-content">
      <Transition name="page" mode="out-in">
        <div :key="activeSection" class="page-transition-frame">
          <HostView
            v-if="activeSection === 'create'"
            :status="status"
            :uri-lifetime="preferences.hostUriLifetime"
            @change-uri-lifetime="preferencesStore.setHostUriLifetime"
          />
          <JoinView
            v-else-if="activeSection === 'join'"
            :status="status"
            :saved-invite="preferences.joinUri"
            :saved-port="preferences.joinPort"
            :incoming-invite="uiStore.incomingInvite"
            @consume-incoming-invite="uiStore.consumeIncomingInvite"
          />
          <PersonalizationView
            v-else-if="activeSection === 'personalize'"
            :preferences="preferences"
            @change-color-theme="preferencesStore.setColorTheme"
            @change-theme="preferencesStore.setTheme"
            @saved="preferencesStore.applyPersonalization"
          />
          <SettingsView
            v-else
            :preferences="preferences"
            @saved="preferencesStore.applyConnectionSettings"
          />
        </div>
      </Transition>
    </main>

    <button
      v-if="uiStore.showsSaveButton"
      class="floating-save-button"
      type="button"
      :title="t('connectionSettings.save')"
      :aria-label="t('connectionSettings.save')"
      :disabled="!uiStore.activeSaveState.enabled"
      @click="uiStore.saveActiveSection"
    >
      <LoaderCircle v-if="uiStore.activeSaveState.saving" class="spin" :size="17" />
      <Save v-else :size="17" />
    </button>
  </div>

  <div v-if="uiStore.closePromptOpen" class="modal-backdrop close-action-backdrop">
    <section
      class="confirm-dialog close-action-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="close-action-title"
      aria-describedby="close-action-hint"
    >
      <h2 id="close-action-title">{{ t("window.closePromptTitle") }}</h2>
      <p id="close-action-hint">{{ t("window.closePromptHint") }}</p>
      <div class="dialog-actions">
        <button
          class="danger-button"
          type="button"
          :disabled="choosingCloseAction"
          @click="chooseCloseAction('exit')"
        >
          {{ t("personalization.exitApplication") }}
        </button>
        <button
          class="primary-button"
          type="button"
          :disabled="choosingCloseAction"
          @click="chooseCloseAction('hide_to_tray')"
        >
          {{ t("personalization.hideToTray") }}
        </button>
      </div>
    </section>
  </div>
</template>
