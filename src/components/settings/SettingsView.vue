<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Cmz_Button,
  Cmz_Input,
  Cmz_Select,
  Cmz_TabBar,
  type SelectOption,
  type TabBarItem,
} from "cmzya-modern-ui";
import { Check, CircleAlert, Network, RefreshCw, Save } from "lucide-vue-next";
import { t } from "../../i18n";
import type { ConnectionSettingsUpdate, Preferences } from "../../preferences";

const props = defineProps<{ preferences: Preferences }>();
const emit = defineEmits<{ saved: [update: ConnectionSettingsUpdate] }>();

const form = ref<ConnectionSettingsUpdate>(pickSettings(props.preferences));
const reconnectUnlimited = ref(props.preferences.reconnectTimeoutSecs == null);
const saving = ref(false);
const result = ref<"saved" | "error" | null>(null);
const relayTabs = computed<TabBarItem[]>(() => [
  { key: "default", label: t("connectionSettings.defaultRelay") },
  { key: "custom", label: t("connectionSettings.customRelay") },
]);
const reconnectTabs = computed<TabBarItem[]>(() => [
  { key: "unlimited", label: t("connectionSettings.unlimited") },
  { key: "limited", label: t("connectionSettings.limited") },
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

function setRelayMode(value: string | null) {
  if (value === "default" || value === "custom") form.value.relayCustom = value === "custom";
}

function setReconnectMode(value: string | null) {
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

async function save() {
  if (!validRelay.value || saving.value) return;
  saving.value = true;
  result.value = null;
  const update: ConnectionSettingsUpdate = {
    relayCustom: form.value.relayCustom,
    relayUrl: form.value.relayUrl.trim(),
    reconnectTimeoutSecs: reconnectUnlimited.value ? null : form.value.reconnectTimeoutSecs,
  };
  try {
    await invoke("set_connection_settings", { update });
    emit("saved", update);
    result.value = "saved";
  } catch (error) {
    console.error("Failed to save connection settings", error);
    result.value = "error";
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="workspace settings-workspace">
    <section class="settings-section">
      <div class="settings-section-heading">
        <Network :size="20" />
        <div>
          <h2>{{ t("connectionSettings.relayNode") }}</h2>
          <p>{{ t("connectionSettings.hint") }}</p>
        </div>
      </div>
      <div class="preference-row">
        <span>{{ t("connectionSettings.relayNode") }}</span>
        <Cmz_TabBar
          class="mode-tabs settings-segment"
          :model-value="form.relayCustom ? 'custom' : 'default'"
          :tabs="relayTabs"
          :level="2"
          @update:model-value="setRelayMode"
        />
      </div>
      <div class="settings-detail-slot">
        <label v-if="form.relayCustom" class="preference-row settings-input-row">
          <span>{{ t("connectionSettings.customRelayUrl") }}</span>
          <Cmz_Input
            v-model="form.relayUrl"
            type="url"
            :placeholder="t('connectionSettings.relayPlaceholder')"
          />
        </label>
      </div>
      <p v-if="form.relayCustom && !validRelay" class="field-error relay-error">
        <CircleAlert :size="14" />{{ t("connectionSettings.invalidRelay") }}
      </p>
    </section>

    <section class="settings-section">
      <div class="settings-section-heading">
        <RefreshCw :size="20" />
        <div>
          <h2>{{ t("connectionSettings.reconnectPolicy") }}</h2>
          <p>{{ t("connectionSettings.hint") }}</p>
        </div>
      </div>
      <div class="preference-row">
        <span>{{ t("connectionSettings.reconnectPolicy") }}</span>
        <Cmz_TabBar
          class="mode-tabs settings-segment"
          :model-value="reconnectUnlimited ? 'unlimited' : 'limited'"
          :tabs="reconnectTabs"
          :level="2"
          @update:model-value="setReconnectMode"
        />
      </div>
      <div class="settings-detail-slot">
        <label v-if="!reconnectUnlimited" class="preference-row settings-input-row">
          <span>{{ t("connectionSettings.timeout") }}</span>
          <Cmz_Select
            :model-value="form.reconnectTimeoutSecs ?? 30"
            :options="timeoutOptions"
            dropdown-width="100%"
            @update:model-value="setReconnectTimeout"
          />
        </label>
      </div>
    </section>

    <div class="settings-actions">
      <p v-if="result" class="settings-result" :class="result">
        <Check v-if="result === 'saved'" :size="15" />
        <CircleAlert v-else :size="15" />
        {{ result === "saved" ? t("connectionSettings.saved") : t("common.saveFailed") }}
      </p>
      <Cmz_Button size="sm" :loading="saving" :disabled="!validRelay" @click="save">
        <Save :size="15" />{{ t("connectionSettings.save") }}
      </Cmz_Button>
    </div>
  </div>
</template>
