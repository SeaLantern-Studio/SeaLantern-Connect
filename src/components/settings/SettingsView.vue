<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, watchEffect } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Cmz_Input, Cmz_Select, type SelectOption } from "cmzya-modern-ui";
import { t } from "../../i18n";
import type { ConnectionSettingsUpdate, Preferences } from "../../preferences";
import { useUiStore } from "../../stores/ui";
import { toast } from "../../toast";

const props = defineProps<{ preferences: Preferences }>();
const emit = defineEmits<{ saved: [update: ConnectionSettingsUpdate] }>();
const uiStore = useUiStore();

const form = ref<ConnectionSettingsUpdate>(pickSettings(props.preferences));
const reconnectUnlimited = ref(props.preferences.reconnectTimeoutSecs == null);
const saving = ref(false);
const relayOptions = computed<SelectOption[]>(() => [
  { label: t("connectionSettings.defaultRelay"), value: "default" },
  { label: t("connectionSettings.customRelay"), value: "custom" },
]);
const reconnectOptions = computed<SelectOption[]>(() => [
  { label: t("connectionSettings.unlimited"), value: "unlimited" },
  { label: t("connectionSettings.limited"), value: "limited" },
]);
const timeoutOptions = computed<SelectOption[]>(() =>
  [10, 15, 20, 30, 60].map((seconds) => ({
    label: t("connectionSettings.seconds", { value: seconds }),
    value: seconds,
  })),
);

function pickSettings(preferences: Preferences): ConnectionSettingsUpdate {
  return {
    relayCustom: preferences.relayCustom,
    relayUrl: preferences.relayUrl,
    reconnectTimeoutSecs: preferences.reconnectTimeoutSecs ?? 30,
  };
}

function setReconnectTimeout(value: string | number) {
  form.value.reconnectTimeoutSecs = Number(value);
}

function setRelayMode(value: string | number) {
  if (value === "default" || value === "custom") form.value.relayCustom = value === "custom";
}

function setReconnectMode(value: string | number) {
  if (value === "unlimited" || value === "limited") {
    reconnectUnlimited.value = value === "unlimited";
  }
}

watch(
  () => props.preferences,
  (preferences) => {
    form.value = pickSettings(preferences);
    reconnectUnlimited.value = preferences.reconnectTimeoutSecs == null;
  },
  { deep: true },
);

const validRelay = computed(() => {
  if (!form.value.relayCustom) return true;
  try {
    const url = new URL(form.value.relayUrl.trim());
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
});
const pendingUpdate = computed<ConnectionSettingsUpdate>(() => ({
  relayCustom: form.value.relayCustom,
  relayUrl: form.value.relayUrl.trim(),
  reconnectTimeoutSecs: reconnectUnlimited.value ? null : form.value.reconnectTimeoutSecs,
}));
const hasChanges = computed(() => {
  const current = props.preferences;
  const update = pendingUpdate.value;
  return (
    update.relayCustom !== current.relayCustom ||
    update.relayUrl !== current.relayUrl ||
    update.reconnectTimeoutSecs !== current.reconnectTimeoutSecs
  );
});

async function save() {
  if (!validRelay.value || !hasChanges.value || saving.value) return;
  saving.value = true;
  const update = pendingUpdate.value;
  try {
    await invoke("set_connection_settings", { update });
    emit("saved", update);
    toast.success(t("connectionSettings.saved"));
  } catch (error) {
    console.error("Failed to save connection settings", error);
    toast.error(t("common.saveFailed"));
  } finally {
    saving.value = false;
  }
}

const saveAction = () => save();

watchEffect(() => {
  uiStore.setSaveState(
    "settings",
    !saving.value && validRelay.value && hasChanges.value,
    saving.value,
  );
});

onMounted(() => uiStore.registerSaveAction("settings", saveAction));
onUnmounted(() => uiStore.unregisterSaveAction("settings", saveAction));
</script>

<template>
  <div class="workspace settings-workspace">
    <section class="settings-section">
      <div class="settings-section-heading">
        <div>
          <h2>{{ t("connectionSettings.relaySection") }}</h2>
          <p>{{ t("connectionSettings.relayHint") }}</p>
        </div>
      </div>
      <div class="preference-row">
        <span>{{ t("connectionSettings.relayNode") }}</span>
        <Cmz_Select
          class="settings-select"
          :model-value="form.relayCustom ? 'custom' : 'default'"
          :options="relayOptions"
          @update:model-value="setRelayMode"
        />
      </div>
      <label v-if="form.relayCustom" class="preference-row settings-input-row">
        <span>{{ t("connectionSettings.customRelayUrl") }}</span>
        <Cmz_Input
          class="settings-input"
          v-model="form.relayUrl"
          type="url"
          :placeholder="t('connectionSettings.relayPlaceholder')"
        />
      </label>
      <p v-if="form.relayCustom && !validRelay" class="field-error relay-error">
        {{ t("connectionSettings.invalidRelay") }}
      </p>
    </section>

    <section class="settings-section">
      <div class="settings-section-heading">
        <div>
          <h2>{{ t("connectionSettings.reconnectSection") }}</h2>
          <p>{{ t("connectionSettings.reconnectHint") }}</p>
        </div>
      </div>
      <div class="preference-row">
        <span>{{ t("connectionSettings.reconnectPolicy") }}</span>
        <Cmz_Select
          class="settings-select"
          :model-value="reconnectUnlimited ? 'unlimited' : 'limited'"
          :options="reconnectOptions"
          @update:model-value="setReconnectMode"
        />
      </div>
      <label v-if="!reconnectUnlimited" class="preference-row settings-input-row">
        <span>{{ t("connectionSettings.timeout") }}</span>
        <Cmz_Select
          class="settings-select"
          :model-value="form.reconnectTimeoutSecs ?? 30"
          :options="timeoutOptions"
          dropdown-width="100%"
          @update:model-value="setReconnectTimeout"
        />
      </label>
    </section>
  </div>
</template>
