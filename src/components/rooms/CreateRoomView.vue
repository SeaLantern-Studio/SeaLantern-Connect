<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Check,
  CircleAlert,
  Copy,
  HousePlus,
  LoaderCircle,
  Radio,
  RefreshCw,
  Square,
} from "lucide-vue-next";

interface ConnectStatus {
  phase: "idle" | "starting" | "active" | "stopping";
  mode: "host" | "join" | null;
  shareUri: string | null;
  playerCount: number;
  hostPort: number | null;
  message: string | null;
  error: string | null;
}

interface LanScanSnapshot {
  scanning: boolean;
  port: number | null;
}

const props = defineProps<{
  status: ConnectStatus;
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

async function beginScan(restart = false) {
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
      if (scan.value.port != null || !scan.value.scanning) stopPolling();
    } catch (error) {
      scanError.value = String(error);
      stopPolling();
    }
  }, 800);
}

function stopPolling() {
  if (scanTimer != null) {
    window.clearInterval(scanTimer);
    scanTimer = null;
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
    }
  },
);

onUnmounted(stopPolling);
</script>

<template>
  <div class="workspace create-workspace">
    <section class="intro">
      <div class="status-mark" :class="status.phase">
        <Radio v-if="hosting && status.phase === 'active'" :size="25" />
        <LoaderCircle v-else-if="hosting" class="spin" :size="25" />
        <HousePlus v-else :size="25" />
      </div>
      <div>
        <h1>{{ hosting ? "房间正在运行" : "共享 Minecraft 世界" }}</h1>
        <p v-if="hosting">复制邀请并发送给好友，对方即可通过 SeaLantern Connect 加入。</p>
        <p v-else>打开单人世界的局域网联机，SeaLantern Connect 会自动寻找开放端口。</p>
      </div>
      <span class="phase-pill" :class="status.phase">
        {{ hosting ? (status.phase === "active" ? "已创建" : "处理中") : "未创建" }}
      </span>
    </section>

    <section v-if="hosting" class="connection-panel host-panel">
      <div class="share-block">
        <span>房间邀请</span>
        <strong>{{ status.shareUri ?? "正在生成邀请..." }}</strong>
        <button class="copy-button" type="button" :disabled="!status.shareUri" @click="copyInvite">
          <Check v-if="copied" :size="16" />
          <Copy v-else :size="16" />
          {{ copied ? "已复制" : "复制邀请" }}
        </button>
      </div>
      <div class="host-summary">
        <div>
          <span>当前玩家</span>
          <strong>{{ status.playerCount }}</strong>
        </div>
        <div>
          <span>目标端口</span>
          <strong>{{ status.hostPort ?? "--" }}</strong>
        </div>
      </div>
      <div class="connection-footer">
        <p>{{ status.message ?? "房间已准备好，等待玩家加入。" }}</p>
        <button class="danger-button" type="button" :disabled="pending" @click="stopRoom">
          <Square :size="15" />停止房间
        </button>
      </div>
    </section>

    <section v-else class="create-panel">
      <div class="form-field">
        <span class="field-label">Minecraft 端口</span>
        <div class="segmented-control" aria-label="端口选择方式">
          <button
            type="button"
            :class="{ active: portMode === 'auto' }"
            @click="
              portMode = 'auto';
              beginScan();
            "
          >
            自动发现
          </button>
          <button
            type="button"
            :class="{ active: portMode === 'manual' }"
            @click="portMode = 'manual'"
          >
            手动填写
          </button>
        </div>
      </div>

      <div v-if="portMode === 'auto'" class="discovery-row">
        <div class="discovery-state" :class="{ detected: scan.port != null, failed: scanError }">
          <Check v-if="scan.port != null" :size="18" />
          <CircleAlert v-else-if="scanError" :size="18" />
          <LoaderCircle v-else class="spin" :size="18" />
          <div>
            <strong>{{
              scan.port != null
                ? `已发现端口 ${scan.port}`
                : scanError
                  ? "无法自动发现"
                  : "正在寻找局域网世界"
            }}</strong>
            <span>{{
              scan.port != null ? "Minecraft 世界可以创建房间" : "等待 Minecraft 广播局域网端口"
            }}</span>
          </div>
        </div>
        <button class="icon-button" type="button" title="重新扫描" @click="beginScan(true)">
          <RefreshCw :size="16" />
        </button>
      </div>

      <div v-else class="form-field manual-port-field">
        <label for="host-port" class="field-label">端口号</label>
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
        <label for="max-players" class="field-label">最大玩家数</label>
        <input
          id="max-players"
          v-model="maxPlayers"
          type="number"
          min="1"
          max="1000"
          inputmode="numeric"
          placeholder="不限制"
        />
      </div>

      <div class="create-actions">
        <p v-if="commandError || occupied || status.message" class="field-error">
          <CircleAlert :size="14" />{{
            occupied ? "请先停止当前连接。" : commandError || status.message
          }}
        </p>
        <button class="primary-button" type="button" :disabled="!canCreate" @click="createRoom">
          <LoaderCircle v-if="pending" class="spin" :size="17" />
          <HousePlus v-else :size="17" />
          创建房间
        </button>
      </div>
    </section>
  </div>
</template>
