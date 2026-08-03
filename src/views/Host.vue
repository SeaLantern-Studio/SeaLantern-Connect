<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { Cmz_Button, Cmz_Input, Cmz_Select, type SelectOption } from "cmzya-modern-ui";
import {
  getLanScan,
  probeHostPort,
  startHost,
  startLanScan,
  stopLanScan,
  stopTunnel,
  type LanScanSnapshot,
} from "@api";
import type { HostUriLifetime } from "@models/preferences";
import type { P2pPeer, P2pStatus } from "@models/p2p";
import { backendMessage, t } from "@i18n";
import {
  Check,
  CircleAlert,
  CircleCheck,
  Copy,
  HousePlus,
  LoaderCircle,
  Square,
} from "@lucide/vue";

const props = defineProps<{
  status: P2pStatus;
  uriLifetime: HostUriLifetime;
}>();
const emit = defineEmits<{
  changeUriLifetime: [value: HostUriLifetime];
}>();

const portMode = ref<"auto" | "manual">("auto");
const manualPort = ref("25565");
const maxPlayers = ref("");
const scan = ref<LanScanSnapshot>({ scanning: false, port: null });
const scanError = ref("");
const commandError = ref("");
const pending = ref(false);
const copied = ref(false);
let scanTimer: number | null = null;
let monitorTimer: number | null = null;
const portModeOptions = computed<SelectOption[]>(() => [
  { label: t("create.automaticDiscovery"), value: "auto" },
  { label: t("create.manual"), value: "manual" },
]);
const uriLifetimeOptions = computed<SelectOption[]>(() => [
  { label: t("create.lifetimeAlways"), value: "always" },
  { label: t("create.lifetimeNever"), value: "never" },
  { label: t("create.lifetime1h"), value: "1h" },
  { label: t("create.lifetime3h"), value: "3h" },
  { label: t("create.lifetime6h"), value: "6h" },
  { label: t("create.lifetime12h"), value: "12h" },
  { label: t("create.lifetime24h"), value: "24h" },
]);

const hosting = computed(() => props.status.mode === "host" && props.status.phase !== "idle");
const occupied = computed(() => props.status.phase !== "idle" && props.status.mode !== "host");
const selectedPort = computed(() =>
  portMode.value === "auto" ? scan.value.port : Number(manualPort.value),
);
const validPort = computed(() => {
  const port = selectedPort.value;
  return Number.isInteger(port) && port != null && port >= 1 && port <= 65535;
});
const validMaxPlayers = computed(() => {
  if (maxPlayers.value.trim() === "") return true;
  const count = Number(maxPlayers.value);
  return Number.isInteger(count) && count >= 1 && count <= 1000;
});
const canCreate = computed(
  () => !pending.value && !occupied.value && validPort.value && validMaxPlayers.value,
);
const localizedStatusMessage = computed(() =>
  props.status.message ? backendMessage(props.status.message) : null,
);
const hostPeers = computed(() => props.status.hostPeers);

function setPortMode(value: string | number) {
  if (value !== "auto" && value !== "manual") return;
  portMode.value = value;
  if (value === "auto") void beginScan(true);
  else void stopScan();
}

function setUriLifetime(value: string | number) {
  if (!uriLifetimeOptions.value.some((option) => option.value === value)) return;
  emit("changeUriLifetime", value as HostUriLifetime);
}

async function beginScan(restart = false) {
  stopMonitoring();
  scanError.value = "";
  try {
    scan.value = await startLanScan(restart);
    startPolling();
  } catch (error) {
    scanError.value = backendMessage(error);
  }
}

function startPolling() {
  if (scanTimer != null) return;
  scanTimer = window.setInterval(async () => {
    try {
      scan.value = await getLanScan();
      if (scan.value.port != null) {
        stopPolling();
        startMonitoring();
      } else if (!scan.value.scanning) {
        stopPolling();
      }
    } catch (error) {
      scanError.value = backendMessage(error);
      stopPolling();
    }
  }, 800);
}

function startMonitoring() {
  if (monitorTimer != null) return;
  monitorTimer = window.setInterval(async () => {
    const port = scan.value.port;
    if (port == null) return;
    try {
      const available = await probeHostPort(port);
      if (!available) void beginScan(true);
    } catch (error) {
      scanError.value = backendMessage(error);
    }
  }, 5000);
}

function stopMonitoring() {
  if (monitorTimer != null) {
    window.clearInterval(monitorTimer);
    monitorTimer = null;
  }
}

function stopPolling() {
  if (scanTimer != null) {
    window.clearInterval(scanTimer);
    scanTimer = null;
  }
}

async function stopScan() {
  stopPolling();
  stopMonitoring();
  scan.value = { scanning: false, port: null };
  try {
    await stopLanScan();
  } catch (error) {
    scanError.value = backendMessage(error);
  }
}

async function createRoom() {
  if (!canCreate.value || selectedPort.value == null) return;
  pending.value = true;
  commandError.value = "";
  try {
    await startHost(
      selectedPort.value,
      maxPlayers.value.trim() === "" ? null : Number(maxPlayers.value),
      props.uriLifetime,
    );
  } catch (error) {
    commandError.value = backendMessage(error);
  } finally {
    pending.value = false;
  }
}

async function stopRoom() {
  pending.value = true;
  commandError.value = "";
  try {
    await stopTunnel();
  } catch (error) {
    commandError.value = backendMessage(error);
  } finally {
    pending.value = false;
  }
}

async function copyInvite() {
  if (!props.status.shareUri) return;
  await navigator.clipboard.writeText(props.status.shareUri);
  copied.value = true;
  window.setTimeout(() => (copied.value = false), 1600);
}

function compactPeerId(id: string) {
  return id.length > 18 ? `${id.slice(0, 10)}...${id.slice(-6)}` : id;
}

function peerRoute(peer: P2pPeer) {
  if (peer.route === "direct") return t("join.direct");
  if (peer.route === "relay") return t("join.relay");
  return t("join.detecting");
}

onMounted(() => {
  if (props.status.phase === "idle") void beginScan();
});

watch(
  () => props.status.phase,
  (phase, previous) => {
    if (phase === "idle" && previous !== "idle" && portMode.value === "auto") {
      void beginScan(true);
    } else if (phase !== "idle") {
      void stopScan();
    }
  },
);

onUnmounted(() => {
  stopPolling();
  stopMonitoring();
  void stopLanScan().catch((error) => {
    console.error("Failed to stop LAN scan", error);
  });
});
</script>

<template>
  <div class="workspace create-workspace">
    <section class="intro">
      <div>
        <h1>{{ hosting ? t("create.running") : t("create.title") }}</h1>
        <p v-if="hosting">{{ t("create.runningHint") }}</p>
        <p v-else>{{ t("create.idleHint") }}</p>
      </div>
      <span class="phase-pill" :class="status.phase">
        {{
          hosting
            ? status.phase === "active"
              ? t("create.created")
              : t("create.processing")
            : t("create.notCreated")
        }}
      </span>
    </section>

    <section v-if="hosting" class="connection-panel host-panel">
      <div class="share-block">
        <span>{{ t("create.invite") }}</span>
        <strong>{{ status.shareUri ?? t("create.generatingInvite") }}</strong>
        <Cmz_Button
          class="copy-button"
          variant="outline"
          type="button"
          :disabled="!status.shareUri"
          @click="copyInvite"
        >
          <Check v-if="copied" :size="16" />
          <Copy v-else :size="16" />
          {{ copied ? t("create.copied") : t("create.copyInvite") }}
        </Cmz_Button>
      </div>
      <div class="host-summary">
        <div>
          <span>{{ t("create.players") }}</span>
          <strong>{{ status.playerCount }}</strong>
        </div>
        <div>
          <span>{{ t("create.targetPort") }}</span>
          <strong>{{ status.hostPort ?? "--" }}</strong>
        </div>
      </div>
      <section v-if="hostPeers.length > 0" class="host-peer-section">
        <div class="host-peer-heading">
          <span>{{ t("create.playerConnections") }}</span>
          <strong>{{ hostPeers.length }}</strong>
        </div>
        <div class="host-peer-list">
          <div v-for="peer in hostPeers" :key="peer.id" class="host-peer-row">
            <code :title="peer.id">{{ compactPeerId(peer.id) }}</code>
            <span>{{ peerRoute(peer) }}</span>
            <strong>{{ peer.rttMs == null ? "--" : `${peer.rttMs} ms` }}</strong>
          </div>
        </div>
      </section>
      <div class="connection-footer">
        <p>{{ localizedStatusMessage ?? t("create.ready") }}</p>
        <Cmz_Button
          class="danger-button"
          variant="outline"
          type="button"
          :disabled="pending"
          :loading="pending"
          @click="stopRoom"
        >
          <Square :size="15" />{{ t("create.stop") }}
        </Cmz_Button>
      </div>
    </section>

    <section v-else class="create-panel">
      <div class="form-field">
        <span class="field-label">{{ t("create.minecraftPort") }}</span>
        <Cmz_Select
          class="settings-select"
          :model-value="portMode"
          :options="portModeOptions"
          @update:model-value="setPortMode"
        />
      </div>

      <div v-if="portMode === 'auto'" class="discovery-row">
        <div class="discovery-state" :class="{ detected: scan.port != null, failed: scanError }">
          <div>
            <strong v-if="scan.port != null">
              {{ t("create.portFoundLabel") }}
              <span class="detected-port">{{ scan.port }}</span>
            </strong>
            <strong v-else>{{
              scanError ? t("create.discoveryFailed") : t("create.discovering")
            }}</strong>
            <span>{{
              scan.port != null ? t("create.worldReady") : t("create.waitingBroadcast")
            }}</span>
          </div>
        </div>
        <span
          class="discovery-status-icon"
          :class="{ detected: scan.port != null, failed: scanError }"
          :title="
            scan.port != null
              ? t('create.portFoundLabel')
              : scanError
                ? t('create.discoveryFailed')
                : t('create.discovering')
          "
          aria-hidden="true"
        >
          <CircleCheck v-if="scan.port != null" :size="20" />
          <CircleAlert v-else-if="scanError" :size="19" />
          <LoaderCircle v-else class="spin" :size="19" />
        </span>
      </div>

      <div v-else class="form-field manual-port-field">
        <label for="host-port" class="field-label">{{ t("create.port") }}</label>
        <Cmz_Input
          id="host-port"
          class="room-input"
          v-model="manualPort"
          type="number"
          :min="1"
          :max="65535"
          :hide-number-controls="true"
        />
      </div>

      <div class="form-field settings-field">
        <label for="max-players" class="field-label">{{ t("create.maxPlayers") }}</label>
        <Cmz_Input
          id="max-players"
          class="room-input"
          v-model="maxPlayers"
          type="number"
          :min="1"
          :max="1000"
          :hide-number-controls="true"
          :placeholder="t('create.unlimited')"
        />
      </div>

      <div class="form-field">
        <span class="field-label">{{ t("create.uriLifetime") }}</span>
        <Cmz_Select
          class="settings-select"
          :model-value="uriLifetime"
          :options="uriLifetimeOptions"
          @update:model-value="setUriLifetime"
        />
      </div>

      <div class="create-actions">
        <p v-if="commandError || occupied || status.message" class="field-error">
          {{ occupied ? t("create.occupied") : commandError || localizedStatusMessage }}
        </p>
        <Cmz_Button
          class="primary-button"
          type="button"
          :disabled="!canCreate"
          :loading="pending"
          @click="createRoom"
        >
          <HousePlus :size="17" />
          {{ t("create.create") }}
        </Cmz_Button>
      </div>
    </section>
  </div>
</template>
