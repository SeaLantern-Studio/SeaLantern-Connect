<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Cmz_Select, type SelectOption } from "cmzya-modern-ui";
import type { ConnectStatus } from "../../connect";
import type { HostUriLifetime } from "../../preferences";
import { t } from "../../i18n";
import {
  Check,
  CircleAlert,
  CircleCheck,
  Copy,
  HousePlus,
  LoaderCircle,
  Square,
} from "lucide-vue-next";

interface LanScanSnapshot {
  scanning: boolean;
  port: number | null;
}

const props = defineProps<{
  status: ConnectStatus;
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
    scan.value = await invoke<LanScanSnapshot>(restart ? "restart_lan_scan" : "start_lan_scan");
    startPolling();
  } catch (error) {
    scanError.value = String(error);
  }
}

function startPolling() {
  if (scanTimer != null) return;
  scanTimer = window.setInterval(async () => {
    try {
      scan.value = await invoke<LanScanSnapshot>("get_lan_scan");
      if (scan.value.port != null) {
        stopPolling();
        startMonitoring();
      } else if (!scan.value.scanning) {
        stopPolling();
      }
    } catch (error) {
      scanError.value = String(error);
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
      const available = await invoke<boolean>("probe_host_port", { port });
      if (!available) void beginScan(true);
    } catch (error) {
      scanError.value = String(error);
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
    await invoke("stop_lan_scan");
  } catch (error) {
    scanError.value = String(error);
  }
}

async function createRoom() {
  if (!canCreate.value || selectedPort.value == null) return;
  pending.value = true;
  commandError.value = "";
  try {
    await invoke("start_host", {
      port: selectedPort.value,
      maxPlayers: maxPlayers.value.trim() === "" ? null : Number(maxPlayers.value),
      uriLifetime: props.uriLifetime,
    });
  } catch (error) {
    commandError.value = String(error);
  } finally {
    pending.value = false;
  }
}

async function stopRoom() {
  pending.value = true;
  commandError.value = "";
  try {
    await invoke("stop_tunnel");
  } catch (error) {
    commandError.value = String(error);
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
  void invoke("stop_lan_scan").catch((error) => {
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
        <button class="copy-button" type="button" :disabled="!status.shareUri" @click="copyInvite">
          <Check v-if="copied" :size="16" />
          <Copy v-else :size="16" />
          {{ copied ? t("create.copied") : t("create.copyInvite") }}
        </button>
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
      <div class="connection-footer">
        <p>{{ status.message ?? t("create.ready") }}</p>
        <button class="danger-button" type="button" :disabled="pending" @click="stopRoom">
          <Square :size="15" />{{ t("create.stop") }}
        </button>
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
        <input
          id="host-port"
          v-model="manualPort"
          type="number"
          min="1"
          max="65535"
          inputmode="numeric"
        />
      </div>

      <div class="form-field settings-field">
        <label for="max-players" class="field-label">{{ t("create.maxPlayers") }}</label>
        <input
          id="max-players"
          v-model="maxPlayers"
          type="number"
          min="1"
          max="1000"
          inputmode="numeric"
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
          {{ occupied ? t("create.occupied") : commandError || status.message }}
        </p>
        <button class="primary-button" type="button" :disabled="!canCreate" @click="createRoom">
          <LoaderCircle v-if="pending" class="spin" :size="17" />
          <HousePlus v-else :size="17" />
          {{ t("create.create") }}
        </button>
      </div>
    </section>
  </div>
</template>
