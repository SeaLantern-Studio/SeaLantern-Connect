<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Cmz_Button, Cmz_TabBar, Cmz_Toggle, type TabBarItem } from "cmzya-modern-ui";
import { AppWindow, Palette, Save } from "lucide-vue-next";
import { t } from "../../i18n";
import type { PersonalizationUpdate, Preferences } from "../../preferences";
import { toast } from "../../toast";

const props = defineProps<{ preferences: Preferences }>();
const emit = defineEmits<{
  changeTheme: [theme: PersonalizationUpdate["theme"]];
  saved: [update: PersonalizationUpdate];
}>();

const form = ref<PersonalizationUpdate>(pickPreferences(props.preferences));
const saving = ref(false);
const themeTabs = computed<TabBarItem[]>(() => [
  { key: "system", label: t("personalization.followSystem") },
  { key: "light", label: t("personalization.light") },
  { key: "dark", label: t("personalization.dark") },
]);
const closeActionTabs = computed<TabBarItem[]>(() => [
  { key: "exit", label: t("personalization.exitApplication") },
  { key: "hide_to_tray", label: t("personalization.hideToTray") },
]);

function pickPreferences(preferences: Preferences): PersonalizationUpdate {
  return {
    theme: preferences.theme,
    locale: preferences.locale,
    rememberWindowState: preferences.rememberWindowState,
    closeAction: preferences.closeAction,
  };
}

watch(
  () => props.preferences.theme,
  (theme) => (form.value.theme = theme),
);
watch(
  () => props.preferences.locale,
  (locale) => (form.value.locale = locale),
);
watch(
  () => props.preferences.rememberWindowState,
  (rememberWindowState) => (form.value.rememberWindowState = rememberWindowState),
);
watch(
  () => props.preferences.closeAction,
  (closeAction) => (form.value.closeAction = closeAction),
);

async function save() {
  saving.value = true;
  const update = { ...form.value };
  try {
    await invoke("set_personalization", { update });
    emit("saved", update);
    toast.success(t("personalization.saved"));
  } catch (error) {
    console.error("Failed to save personalization", error);
    toast.error(t("common.saveFailed"));
  } finally {
    saving.value = false;
  }
}

function updateRememberWindowState(value: boolean) {
  form.value.rememberWindowState = value;
}

function setTheme(value: string | null) {
  if (value !== "system" && value !== "light" && value !== "dark") return;
  form.value.theme = value;
  emit("changeTheme", value);
}

function setCloseAction(value: string | null) {
  if (value !== "exit" && value !== "hide_to_tray") return;
  form.value.closeAction = value;
}
</script>

<template>
  <div class="workspace settings-workspace">
    <section class="settings-section">
      <div class="settings-section-heading">
        <Palette :size="20" />
        <div>
          <h2>{{ t("personalization.appearance") }}</h2>
          <p>{{ t("personalization.appearanceHint") }}</p>
        </div>
      </div>

      <div class="preference-row">
        <span>{{ t("personalization.theme") }}</span>
        <Cmz_TabBar
          class="mode-tabs settings-segment three"
          :model-value="form.theme"
          :tabs="themeTabs"
          :level="2"
          @update:model-value="setTheme"
        />
      </div>
    </section>

    <section class="settings-section">
      <div class="settings-section-heading">
        <AppWindow :size="20" />
        <div>
          <h2>{{ t("personalization.windowBehavior") }}</h2>
          <p>{{ t("personalization.windowBehaviorHint") }}</p>
        </div>
      </div>

      <div class="preference-row">
        <span>{{ t("personalization.closeAction") }}</span>
        <Cmz_TabBar
          class="mode-tabs settings-segment"
          :model-value="form.closeAction"
          :tabs="closeActionTabs"
          :level="2"
          @update:model-value="setCloseAction"
        />
      </div>

      <div class="preference-row switch-row">
        <span>{{ t("personalization.rememberWindowState") }}</span>
        <Cmz_Toggle
          :model-value="form.rememberWindowState"
          variant="switch"
          size="sm"
          :disabled="saving"
          @update:model-value="updateRememberWindowState"
        />
      </div>
    </section>

    <div class="settings-actions">
      <Cmz_Button size="sm" :loading="saving" @click="save">
        <Save :size="15" />{{ t("connectionSettings.save") }}
      </Cmz_Button>
    </div>
  </div>
</template>
