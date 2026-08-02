<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { storeToRefs } from "pinia";
import { Cmz_Button, Cmz_Modal } from "cmzya-modern-ui";
import { closeWindow, getInitialDeepLinks, onCloseActionRequested, onDeepLinks } from "@api";
import AppToast from "./components/AppToast.vue";
import SplashScreen from "./components/SplashScreen.vue";
import AppHeader from "./components/layout/AppHeader.vue";
import AppSidebar from "./components/layout/AppSidebar.vue";
import HostView from "./components/rooms/HostView.vue";
import JoinView from "./components/rooms/JoinView.vue";
import PersonalizationView from "./components/settings/PersonalizationView.vue";
import SettingsView from "./components/settings/SettingsView.vue";
import { t } from "./i18n";
import { inviteFromDeepLinkUrls } from "./invitations";
import type { CloseAction } from "./models/preferences";
import { usePreferencesStore } from "./stores/preferences";
import { useSessionStore } from "./stores/session";
import { useUiStore } from "./stores/ui";
import { enableAutoHidingScrollbars } from "./ui/scrollbars";

const sessionStore = useSessionStore();
const preferencesStore = usePreferencesStore();
const uiStore = useUiStore();
const { status, state: connectionState } = storeToRefs(sessionStore);
const { preferences } = storeToRefs(preferencesStore);
const { activeSection, sidebarCollapsed } = storeToRefs(uiStore);
const showSplash = ref(true);
const isInitializing = ref(true);
const choosingCloseAction = ref(false);
let unlistenCloseAction: (() => void) | null = null;
let unlistenDeepLinks: (() => void) | null = null;
let disableAutoHidingScrollbars: (() => void) | null = null;
let lastDeepLink: { uri: string; receivedAt: number } | null = null;

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
    await closeWindow();
  } finally {
    choosingCloseAction.value = false;
  }
}

function importDeepLink(urls: string[]): void {
  const invite = inviteFromDeepLinkUrls(urls);
  if (!invite) return;
  const now = Date.now();
  if (lastDeepLink?.uri === invite && now - lastDeepLink.receivedAt < 1000) return;
  lastDeepLink = { uri: invite, receivedAt: now };
  uiStore.importInvite(invite);
}

async function setupDeepLinks(): Promise<void> {
  try {
    unlistenDeepLinks = await onDeepLinks(importDeepLink);
    importDeepLink(await getInitialDeepLinks());
  } catch (error) {
    console.error("Failed to initialize deep links", error);
  }
}

onMounted(async () => {
  disableAutoHidingScrollbars = enableAutoHidingScrollbars();
  unlistenCloseAction = await onCloseActionRequested(uiStore.openClosePrompt);
  await setupDeepLinks();
  await preferencesStore.load();
  preferencesStore.startSystemThemeListener();
  try {
    await sessionStore.initialize();
  } finally {
    isInitializing.value = false;
  }
});

onUnmounted(() => {
  disableAutoHidingScrollbars?.();
  unlistenCloseAction?.();
  unlistenDeepLinks?.();
  sessionStore.dispose();
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
            @change="preferencesStore.updatePersonalization"
          />
          <SettingsView
            v-else
            :preferences="preferences"
            @change="preferencesStore.updateConnectionSettings"
          />
        </div>
      </Transition>
    </main>
  </div>

  <Cmz_Modal
    :visible="uiStore.closePromptOpen"
    :title="t('window.closePromptTitle')"
    width="380px"
    :close-on-overlay="false"
    @close="uiStore.closeClosePrompt"
  >
    <p class="modal-copy">{{ t("window.closePromptHint") }}</p>
    <template #footer>
      <Cmz_Button
        class="danger-button"
        variant="outline"
        type="button"
        :disabled="choosingCloseAction"
        @click="chooseCloseAction('exit')"
      >
        {{ t("personalization.exitApplication") }}
      </Cmz_Button>
      <Cmz_Button
        class="primary-button"
        type="button"
        :disabled="choosingCloseAction"
        @click="chooseCloseAction('hide_to_tray')"
      >
        {{ t("personalization.hideToTray") }}
      </Cmz_Button>
    </template>
  </Cmz_Modal>
</template>
