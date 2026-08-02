<script setup lang="ts">
import { computed, ref } from "vue";
import { Cmz_Input, Cmz_Select, Cmz_Toggle, type SelectOption } from "cmzya-modern-ui";
import { t } from "../../i18n";
import type {
  ConnectionSettingsUpdate,
  LightweightSettingsUpdate,
  Preferences,
} from "../../models/preferences";

const props = defineProps<{ preferences: Preferences }>();
const emit = defineEmits<{
  change: [update: ConnectionSettingsUpdate];
  lightweightChange: [update: LightweightSettingsUpdate];
}>();

const form = ref<ConnectionSettingsUpdate>(pickSettings(props.preferences));
const reconnectUnlimited = ref(props.preferences.reconnectTimeoutSecs == null);
const autoLightweightEnabled = ref(props.preferences.autoLightweightMinutes != null);
const autoLightweightMinutes = ref(String(props.preferences.autoLightweightMinutes ?? 3));
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

function setAutoLightweightEnabled(value: boolean) {
  autoLightweightEnabled.value = value;
  if (!value) {
    emit("lightweightChange", { autoLightweightMinutes: null });
    return;
  }
  if (!validAutoLightweightMinutes.value) autoLightweightMinutes.value = "3";
  persistLightweight();
}

function setAutoLightweightMinutes(value: string) {
  autoLightweightMinutes.value = value;
  persistLightweight();
}

const validAutoLightweightMinutes = computed(() => {
  const minutes = Number(autoLightweightMinutes.value);
  return Number.isInteger(minutes) && minutes >= 1 && minutes <= 1440;
});

function persistLightweight() {
  if (!autoLightweightEnabled.value || !validAutoLightweightMinutes.value) return;
  emit("lightweightChange", { autoLightweightMinutes: Number(autoLightweightMinutes.value) });
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
          <h2>{{ t("connectionSettings.lightweightSection") }}</h2>
          <p>{{ t("connectionSettings.lightweightHint") }}</p>
        </div>
      </div>
      <div class="preference-row switch-row">
        <span>{{ t("connectionSettings.autoLightweight") }}</span>
        <Cmz_Toggle
          :model-value="autoLightweightEnabled"
          @update:model-value="setAutoLightweightEnabled"
        />
      </div>
      <label v-if="autoLightweightEnabled" class="preference-row settings-input-row">
        <span>{{ t("connectionSettings.lightweightDelay") }}</span>
        <Cmz_Input
          class="settings-input"
          :model-value="autoLightweightMinutes"
          type="number"
          :min="1"
          :max="1440"
          :hide-number-controls="true"
          @update:model-value="setAutoLightweightMinutes"
        >
          <template #suffix>{{ t("connectionSettings.minutes") }}</template>
        </Cmz_Input>
      </label>
      <p
        v-if="autoLightweightEnabled && !validAutoLightweightMinutes"
        class="field-error relay-error"
      >
        {{ t("connectionSettings.invalidLightweightDelay") }}
      </p>
    </section>

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
