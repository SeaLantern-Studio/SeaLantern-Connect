<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Cmz_TabBar, type TabBarItem } from "cmzya-modern-ui";
import { ArrowRight, Check, Copy, Link, Radio, RotateCcw, Unplug } from "lucide-vue-next";
import {
  saveJoinPort as persistJoinPort,
  startJoin,
  stopJoin,
  stopTunnel,
  validateInvite,
} from "@api";
import { normalizeInvite, type IncomingInvite } from "../../invitations";
import type { ConnectStatus } from "../../models/tunnel";
import { t } from "../../i18n";

const props = defineProps<{
  status: ConnectStatus;
  savedInvite: string;
  savedPort: number;
  incomingInvite: IncomingInvite | null;
}>();
const emit = defineEmits<{
  consumeIncomingInvite: [id: number];
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
const replacingConnection = computed(() => props.status.phase !== "idle");
const canJoin = computed(() => invite.value.trim().length > 0 && props.status.phase === "idle");
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

watch(
  () => props.incomingInvite,
  (request) => {
    if (!request) return;
    emit("consumeIncomingInvite", request.id);
    void importIncomingInvite(request.uri);
  },
  { immediate: true },
);

function setPortMode(value: string | null) {
  if (value === "auto" || value === "manual") portMode.value = value;
}

function resetInvite() {
  invite.value = "";
  validationError.value = "";
  commandError.value = "";
}

async function submitInvite() {
  validationError.value = "";
  commandError.value = "";
  try {
    invite.value = normalizeInvite(invite.value);
    await validateInvite(invite.value);
    await join();
  } catch {
    validationError.value = t("join.invalidInvite");
  }
}

async function importIncomingInvite(uri: string) {
  const normalized = normalizeInvite(uri);
  validationError.value = "";
  commandError.value = "";
  invite.value = normalized;
  if (joining.value && props.status.shareUri === normalized) return;
  try {
    await validateInvite(normalized);
    confirming.value = true;
  } catch {
    if (props.status.phase === "idle") validationError.value = t("join.invalidInvite");
    else commandError.value = t("join.invalidInvite");
  }
}

function waitForIdle(timeoutMs = 20_000): Promise<void> {
  if (props.status.phase === "idle") return Promise.resolve();
  return new Promise((resolve, reject) => {
    const stopWatching = watch(
      () => props.status.phase,
      (phase) => {
        if (phase === "idle") finish();
      },
    );
    const timeout = window.setTimeout(
      () => finish(new Error("timed out while stopping the current connection")),
      timeoutMs,
    );
    function finish(error?: Error) {
      window.clearTimeout(timeout);
      stopWatching();
      if (error) reject(error);
      else resolve();
    }
  });
}

async function join() {
  if (portMode.value === "manual" && !validManualPort.value) return;
  confirming.value = false;
  commandError.value = "";
  try {
    if (props.status.phase !== "idle") {
      await stopTunnel();
      await waitForIdle();
    }
    await startJoin(invite.value, portMode.value === "auto" ? null : Number(manualPort.value));
  } catch (error) {
    commandError.value = String(error);
  }
}

async function saveJoinPort() {
  if (!validManualPort.value) return;
  try {
    await persistJoinPort(Number(manualPort.value));
  } catch (error) {
    console.error("Failed to save join port", error);
  }
}

async function stop() {
  commandError.value = "";
  try {
    await stopJoin();
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
          placeholder="https://ideaflash.cn/#/join/v1/..."
          @keydown.enter="canJoin && submitInvite()"
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
        {{ validationError }}
      </p>
      <div class="join-actions">
        <div class="privacy-note">{{ t("join.privacy") }}</div>
        <button class="primary-button" type="button" :disabled="!canJoin" @click="submitInvite">
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
      {{ t("join.connectionFailed") }}
      {{ occupied ? t("join.occupied") : commandError || status.message || t("join.retryHint") }}
    </p>
  </div>

  <div v-if="confirming" class="modal-backdrop" @click.self="confirming = false">
    <section class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="confirm-title">
      <h2 id="confirm-title">{{ t("join.confirmTitle") }}</h2>
      <p>{{ replacingConnection ? t("join.replaceHint") : t("join.confirmHint") }}</p>
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
          <Radio :size="17" />{{
            replacingConnection ? t("join.confirmReplace") : t("join.confirm")
          }}
        </button>
      </div>
    </section>
  </div>
</template>
