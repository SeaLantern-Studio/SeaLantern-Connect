<script setup lang="ts">
import { computed, ref } from "vue";
import { Cmz_Input, Cmz_Select, type SelectOption } from "cmzya-modern-ui";
import { t } from "../../i18n";
import type { ConnectionSettingsUpdate, Preferences } from "../../models/preferences";

const props = defineProps<{ preferences: Preferences }>();
const emit = defineEmits<{ change: [update: ConnectionSettingsUpdate] }>();

const form = ref<ConnectionSettingsUpdate>(pickSettings(props.preferences));
const reconnectUnlimited = ref(props.preferences.reconnectTimeoutSecs == null);
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
  persist();
}

function setRelayMode(value: string | number) {
  if (value !== "default" && value !== "custom") return;
  form.value.relayCustom = value === "custom";
  persist();
}

function setReconnectMode(value: string | number) {
  if (value !== "unlimited" && value !== "limited") return;
  reconnectUnlimited.value = value === "unlimited";
  persist();
}

function setRelayUrl(value: string | number) {
  form.value.relayUrl = String(value);
  persist();
}

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
function persist() {
  if (validRelay.value) emit("change", { ...pendingUpdate.value });
}
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
          :model-value="form.relayUrl"
          type="url"
          :placeholder="t('connectionSettings.relayPlaceholder')"
          @update:model-value="setRelayUrl"
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
