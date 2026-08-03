<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { Cmz_Input, Cmz_Select, Cmz_Toggle, type SelectOption } from "cmzya-modern-ui";
import { disableAutostart, enableAutostart, getAutostartEnabled } from "@api";
import { t } from "@i18n";
import type {
  ApplicationSettingsUpdate,
  ConnectionSettingsUpdate,
  LightweightSettingsUpdate,
  Preferences,
  SplashDurationMs,
} from "@models/preferences";

const props = defineProps<{ preferences: Preferences }>();
const emit = defineEmits<{
  change: [update: ConnectionSettingsUpdate];
  applicationChange: [update: ApplicationSettingsUpdate];
  lightweightChange: [update: LightweightSettingsUpdate];
}>();

const form = ref<ConnectionSettingsUpdate>(pickSettings(props.preferences));
const applicationForm = ref<ApplicationSettingsUpdate>(pickApplicationSettings(props.preferences));
const autostartEnabled = ref(false);
const autostartLoading = ref(true);
const autostartUpdating = ref(false);
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
const splashDurationOptions = computed<SelectOption[]>(() => [
  { label: t("connectionSettings.disabled"), value: 0 },
  ...[500, 1000, 1500, 2000].map((durationMs) => ({
    label: t("connectionSettings.seconds", { value: durationMs / 1000 }),
    value: durationMs,
  })),
]);

function pickApplicationSettings(preferences: Preferences): ApplicationSettingsUpdate {
  return {
    splashDurationMs: preferences.splashDurationMs,
    silentStart: preferences.silentStart,
    rememberWindowState: preferences.rememberWindowState,
  };
}

function pickSettings(preferences: Preferences): ConnectionSettingsUpdate {
  return {
    relayCustom: preferences.relayCustom,
    relayUrl: preferences.relayUrl,
    reconnectTimeoutSecs: preferences.reconnectTimeoutSecs ?? 30,
  };
}

watch(
  () =>
    [
      props.preferences.splashDurationMs,
      props.preferences.silentStart,
      props.preferences.rememberWindowState,
    ] as const,
  ([splashDurationMs, silentStart, rememberWindowState]) => {
    applicationForm.value = { splashDurationMs, silentStart, rememberWindowState };
  },
);

onMounted(() => void loadAutostart());

async function loadAutostart() {
  autostartLoading.value = true;
  try {
    autostartEnabled.value = await getAutostartEnabled();
  } catch (error) {
    console.error("Failed to load autostart state", error);
  } finally {
    autostartLoading.value = false;
  }
}

async function updateAutostart(value: boolean) {
  if (autostartLoading.value || autostartUpdating.value) return;
  const fallback = autostartEnabled.value;
  autostartEnabled.value = value;
  autostartUpdating.value = true;
  try {
    if (value) await enableAutostart();
    else await disableAutostart();
  } catch (error) {
    autostartEnabled.value = fallback;
    console.error("Failed to update autostart state", error);
  } finally {
    autostartUpdating.value = false;
  }
}

function updateSilentStart(value: boolean) {
  applicationForm.value.silentStart = value;
  persistApplicationSettings();
}

function updateRememberWindowState(value: boolean) {
  applicationForm.value.rememberWindowState = value;
  persistApplicationSettings();
}

function setSplashDuration(value: string | number) {
  if (!splashDurationOptions.value.some((option) => option.value === value)) return;
  applicationForm.value.splashDurationMs = value as SplashDurationMs;
  persistApplicationSettings();
}

function persistApplicationSettings() {
  emit("applicationChange", { ...applicationForm.value });
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
          <h2>{{ t("connectionSettings.startup") }}</h2>
        </div>
      </div>
      <div class="preference-row switch-row">
        <span>{{ t("connectionSettings.autostart") }}</span>
        <Cmz_Toggle
          :model-value="autostartEnabled"
          variant="switch"
          size="sm"
          :disabled="autostartLoading || autostartUpdating"
          @update:model-value="updateAutostart"
        />
      </div>
      <div v-if="autostartEnabled" class="preference-row switch-row">
        <span>{{ t("connectionSettings.silentStart") }}</span>
        <Cmz_Toggle
          :model-value="applicationForm.silentStart"
          variant="switch"
          size="sm"
          @update:model-value="updateSilentStart"
        />
      </div>
      <div class="preference-row">
        <span>{{ t("connectionSettings.splashDuration") }}</span>
        <Cmz_Select
          class="settings-select"
          :model-value="applicationForm.splashDurationMs"
          :options="splashDurationOptions"
          @update:model-value="setSplashDuration"
        />
      </div>
    </section>

    <section class="settings-section">
      <div class="settings-section-heading">
        <div>
          <h2>{{ t("connectionSettings.windowBehavior") }}</h2>
        </div>
      </div>
      <div class="preference-row switch-row">
        <span>{{ t("connectionSettings.rememberWindowState") }}</span>
        <Cmz_Toggle
          :model-value="applicationForm.rememberWindowState"
          variant="switch"
          size="sm"
          @update:model-value="updateRememberWindowState"
        />
      </div>
    </section>

    <section class="settings-section">
      <div class="settings-section-heading">
        <div>
          <h2>{{ t("connectionSettings.lightweightSection") }}</h2>
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
