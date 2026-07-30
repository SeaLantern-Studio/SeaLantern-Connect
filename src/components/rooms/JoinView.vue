<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Cmz_TabBar, type TabBarItem } from "cmzya-modern-ui";
import {
  ArrowRight,
  Check,
  CircleAlert,
  Copy,
  Link,
  LoaderCircle,
  Network,
  Radio,
  RotateCcw,
  Unplug,
} from "lucide-vue-next";
import type { ConnectStatus } from "../../connect";
import { t } from "../../i18n";

const props = defineProps<{
  status: ConnectStatus;
  savedInvite: string;
  savedPort: number;
}>();

const invite = ref(props.savedInvite);
const validationError = ref("");
const commandError = ref("");
const confirming = ref(false);
const portMode = ref<"auto" | "manual">("auto");
const manualPort = ref(String(props.savedPort));
const copied = ref(false);
const portModeTabs = computed<TabBarItem[]>(() => [
  { key: "auto", label: t("join.automatic") },
  { key: "manual", label: t("join.manual") },
]);

const joining = computed(() => props.status.mode === "join" && props.status.phase !== "idle");
const occupied = computed(() => props.status.phase !== "idle" && props.status.mode !== "join");
const busy = computed(
  () => joining.value && (props.status.phase === "starting" || props.status.phase === "stopping"),
);
const connected = computed(() => joining.value && props.status.phase === "active");
const canPreview = computed(() => invite.value.trim().length > 0 && props.status.phase === "idle");
const validManualPort = computed(() => {
  const port = Number(manualPort.value);
  return Number.isInteger(port) && port >= 1 && port <= 65535;
});
const phaseLabel = computed(() => {
  if (props.status.phase === "starting") return t("join.starting");
  if (props.status.phase === "active") return t("join.active");
  if (props.status.phase === "stopping") return t("join.stopping");
  return t("join.idle");
});

watch(
  () => props.savedInvite,
  (value) => {
    if (!invite.value) invite.value = value;
  },
);

watch(
  () => props.savedPort,
  (value) => (manualPort.value = String(value)),
);

function normalizeInvite(value: string) {
  const trimmed = value.trim();
  const fragment = trimmed.match(/^https?:\/\/[^#]+\/#\/join\/v1\/([^/?#\s]+)$/i);
  return fragment ? `sculk://join/v1/${fragment[1]}` : trimmed;
}

function setPortMode(value: string | null) {
  if (value === "auto" || value === "manual") portMode.value = value;
}

function resetInvite() {
  invite.value = "";
  validationError.value = "";
  commandError.value = "";
}

async function previewInvite() {
  validationError.value = "";
  commandError.value = "";
  try {
    invite.value = normalizeInvite(invite.value);
    await invoke("validate_invite", { uri: invite.value });
    confirming.value = true;
  } catch {
    validationError.value = t("join.invalidInvite");
  }
}

async function join() {
  if (portMode.value === "manual" && !validManualPort.value) return;
  confirming.value = false;
  commandError.value = "";
  try {
    await invoke("start_join", {
      uri: invite.value,
      localPort: portMode.value === "auto" ? null : Number(manualPort.value),
    });
  } catch (error) {
    commandError.value = String(error);
  }
}

async function saveJoinPort() {
  if (!validManualPort.value) return;
  try {
    await invoke("set_join_port", { port: Number(manualPort.value) });
  } catch (error) {
    console.error("Failed to save join port", error);
  }
}

async function stop() {
  commandError.value = "";
  try {
    await invoke("stop_join");
  } catch (error) {
    commandError.value = String(error);
  }
}

async function copyAddress() {
  if (!props.status.localAddress) return;
  await navigator.clipboard.writeText(props.status.localAddress);
  copied.value = true;
  window.setTimeout(() => (copied.value = false), 1600);
}

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`;
  if (value < 1024 ** 2) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 ** 2).toFixed(1)} MB`;
}
</script>

<template>
  <div class="workspace">
    <section class="intro">
      <div class="status-mark" :class="status.phase">
        <Radio v-if="connected" :size="25" />
        <LoaderCircle v-else-if="busy" class="spin" :size="25" />
        <Link v-else :size="25" />
      </div>
      <div>
        <h1>{{ phaseLabel }}</h1>
        <p v-if="connected">{{ t("join.activeHint") }}</p>
        <p v-else>{{ t("join.idleHint") }}</p>
      </div>
      <span class="phase-pill" :class="status.phase">
        {{ connected ? t("join.connected") : busy ? t("join.processing") : t("join.disconnected") }}
      </span>
    </section>

    <section v-if="!joining" class="join-panel">
      <label for="invite">{{ t("join.invite") }}</label>
      <div class="invite-row" :class="{ invalid: validationError }">
        <Link :size="18" />
        <input
          id="invite"
          v-model="invite"
          type="text"
          spellcheck="false"
          autocomplete="off"
          placeholder="sculk://join/v1/..."
          @keydown.enter="canPreview && previewInvite()"
        />
        <button
          class="reset-invite-button"
          type="button"
          :title="t('join.clearInput')"
          :disabled="invite.length === 0"
          @click="resetInvite"
        >
          <RotateCcw :size="16" />
        </button>
      </div>
      <p v-if="validationError" class="field-error">
        <CircleAlert :size="14" />{{ validationError }}
      </p>
      <div class="join-actions">
        <div class="privacy-note"><Network :size="16" />{{ t("join.privacy") }}</div>
        <button class="primary-button" type="button" :disabled="!canPreview" @click="previewInvite">
          {{ t("join.continue") }}<ArrowRight :size="17" />
        </button>
      </div>
    </section>

    <section v-else class="connection-panel">
      <div class="address-block">
        <span>{{ t("join.minecraftAddress") }}</span>
        <strong>{{ status.localAddress ?? t("join.allocatingPort") }}</strong>
        <button
          class="copy-button"
          type="button"
          :disabled="!status.localAddress"
          @click="copyAddress"
        >
          <Check v-if="copied" :size="16" />
          <Copy v-else :size="16" />
          {{ copied ? t("join.copied") : t("join.copyAddress") }}
        </button>
      </div>
      <div class="metrics">
        <div>
          <span>{{ t("join.route") }}</span>
          <strong>{{
            status.route === "direct"
              ? t("join.direct")
              : status.route === "relay"
                ? t("join.relay")
                : t("join.detecting")
          }}</strong>
        </div>
        <div>
          <span>{{ t("join.latency") }}</span
          ><strong>{{ status.rttMs == null ? "--" : `${status.rttMs} ms` }}</strong>
        </div>
        <div>
          <span>{{ t("join.sent") }}</span
          ><strong>{{ formatBytes(status.txBytes) }}</strong>
        </div>
        <div>
          <span>{{ t("join.received") }}</span
          ><strong>{{ formatBytes(status.rxBytes) }}</strong>
        </div>
      </div>
      <div class="connection-footer">
        <p>{{ status.message ?? t("join.syncing") }}</p>
        <button class="danger-button" type="button" :disabled="busy" @click="stop">
          <Unplug :size="16" />{{ t("join.disconnect") }}
        </button>
      </div>
    </section>

    <p v-if="commandError || status.error || occupied" class="error-banner">
      <CircleAlert :size="16" />{{ t("join.connectionFailed")
      }}{{ occupied ? t("join.occupied") : commandError || status.message || t("join.retryHint") }}
    </p>
  </div>

  <div v-if="confirming" class="modal-backdrop" @click.self="confirming = false">
    <section class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="confirm-title">
      <div class="dialog-icon"><Link :size="23" /></div>
      <h2 id="confirm-title">{{ t("join.confirmTitle") }}</h2>
      <p>{{ t("join.confirmHint") }}</p>
      <div class="invite-summary">
        <span>{{ t("join.inviteProtocol") }}</span
        ><strong>sculk / v1</strong>
      </div>
      <div class="join-port-setting">
        <div class="join-port-heading">
          <span>{{ t("join.localPort") }}</span>
          <Cmz_TabBar
            class="mode-tabs compact"
            :model-value="portMode"
            :tabs="portModeTabs"
            :level="2"
            @update:model-value="setPortMode"
          />
        </div>
        <div class="join-port-detail">
          <span v-if="portMode === 'auto'">{{ t("join.automaticPort") }}</span>
          <input
            v-else
            v-model="manualPort"
            :class="{ invalid: !validManualPort }"
            type="number"
            min="1"
            max="65535"
            inputmode="numeric"
            :aria-label="t('join.localPortNumber')"
            @change="saveJoinPort"
          />
        </div>
      </div>
      <div class="dialog-actions">
        <button class="secondary-button" type="button" @click="confirming = false">
          {{ t("join.cancel") }}
        </button>
        <button
          class="primary-button"
          type="button"
          :disabled="portMode === 'manual' && !validManualPort"
          @click="join"
        >
          <Radio :size="17" />{{ t("join.confirm") }}
        </button>
      </div>
    </section>
  </div>
</template>
