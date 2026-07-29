<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import SplashScreen from "./components/SplashScreen.vue";
import {
  ArrowRight,
  Check,
  CircleAlert,
  Copy,
  Link,
  LoaderCircle,
  Minus,
  Moon,
  Network,
  Radio,
  Square,
  Sun,
  Unplug,
  X,
} from "lucide-vue-next";

type Phase = "idle" | "starting" | "active" | "stopping";

interface ConnectStatus {
  phase: Phase;
  localAddress: string | null;
  route: "direct" | "relay" | null;
  rttMs: number | null;
  txBytes: number;
  rxBytes: number;
  error: string | null;
  message: string | null;
}

const emptyStatus: ConnectStatus = {
  phase: "idle",
  localAddress: null,
  route: null,
  rttMs: null,
  txBytes: 0,
  rxBytes: 0,
  error: null,
  message: null,
};

const invite = ref("");
const status = ref<ConnectStatus>(emptyStatus);
const validationError = ref("");
const commandError = ref("");
const confirming = ref(false);
const copied = ref(false);
const showSplash = ref(true);
const isInitializing = ref(true);
const dark = ref(window.matchMedia("(prefers-color-scheme: dark)").matches);
const isMacOS = /Macintosh|Mac OS X/i.test(navigator.userAgent);
const appWindow = getCurrentWindow();
const isMaximized = ref(false);
let unlisten: UnlistenFn | null = null;
let unlistenResize: UnlistenFn | null = null;

const busy = computed(() => status.value.phase === "starting" || status.value.phase === "stopping");
const connected = computed(() => status.value.phase === "active");
const canPreview = computed(() => invite.value.trim().length > 0 && status.value.phase === "idle");
const phaseLabel = computed(() => {
  if (status.value.phase === "starting") return "正在建立安全连接";
  if (status.value.phase === "active") return "联机通道已就绪";
  if (status.value.phase === "stopping") return "正在断开连接";
  return "等待联机邀请";
});

function applyTheme() {
  document.documentElement.dataset.theme = dark.value ? "dark" : "light";
}

function toggleTheme() {
  dark.value = !dark.value;
  applyTheme();
}

async function minimizeWindow() {
  await appWindow.minimize();
}

async function toggleMaximize() {
  await appWindow.toggleMaximize();
}

async function closeWindow() {
  await appWindow.close();
}

function hideSplash() {
  showSplash.value = false;
}

function normalizeInvite(value: string) {
  const trimmed = value.trim();
  const fragment = trimmed.match(/^https?:\/\/[^#]+\/#\/join\/v1\/([^/?#\s]+)$/i);
  return fragment ? `sculk://join/v1/${fragment[1]}` : trimmed;
}

async function previewInvite() {
  validationError.value = "";
  commandError.value = "";
  try {
    invite.value = normalizeInvite(invite.value);
    await invoke("validate_invite", { uri: invite.value });
    confirming.value = true;
  } catch {
    validationError.value = "这不是有效的 SeaLantern 联机邀请，请检查后重试。";
  }
}

async function join() {
  confirming.value = false;
  commandError.value = "";
  try {
    await invoke("start_join", { uri: invite.value });
    status.value = await invoke<ConnectStatus>("get_status");
  } catch (error) {
    commandError.value = String(error);
  }
}

async function stop() {
  commandError.value = "";
  try {
    await invoke("stop_join");
    status.value = await invoke<ConnectStatus>("get_status");
  } catch (error) {
    commandError.value = String(error);
  }
}

async function copyAddress() {
  if (!status.value.localAddress) return;
  await navigator.clipboard.writeText(status.value.localAddress);
  copied.value = true;
  window.setTimeout(() => (copied.value = false), 1600);
}

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`;
  if (value < 1024 ** 2) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 ** 2).toFixed(1)} MB`;
}

onMounted(async () => {
  applyTheme();
  try {
    isMaximized.value = await appWindow.isMaximized();
    unlistenResize = await appWindow.onResized(async () => {
      isMaximized.value = await appWindow.isMaximized();
    });
    status.value = await invoke<ConnectStatus>("get_status");
    unlisten = await listen<ConnectStatus>("connect-status", (event) => {
      status.value = event.payload;
    });
  } finally {
    isInitializing.value = false;
  }
});

onUnmounted(() => {
  unlisten?.();
  unlistenResize?.();
});
</script>

<template>
  <Transition name="splash-fade">
    <SplashScreen v-if="showSplash" :loading="isInitializing" @ready="hideSplash" />
  </Transition>

  <div v-if="!showSplash" class="app-shell">
    <header class="titlebar" :class="{ 'macos-overlay': isMacOS }" data-tauri-drag-region>
      <div class="brand" data-tauri-drag-region>
        <img src="/logo.svg" alt="" />
        <span>SeaLantern Connect</span>
      </div>
      <div class="titlebar-actions">
        <button class="icon-button" type="button" title="切换明暗主题" @click="toggleTheme">
          <Sun v-if="dark" :size="17" />
          <Moon v-else :size="17" />
        </button>
        <div v-if="!isMacOS" class="window-controls">
          <button class="window-button" type="button" title="最小化" @click="minimizeWindow">
            <Minus :size="12" />
          </button>
          <button
            class="window-button"
            type="button"
            :title="isMaximized ? '还原' : '最大化'"
            @click="toggleMaximize"
          >
            <Copy v-if="isMaximized" :size="12" />
            <Square v-else :size="12" />
          </button>
          <button class="window-button window-button-close" type="button" title="关闭" @click="closeWindow">
            <X :size="12" />
          </button>
        </div>
      </div>
    </header>

    <main class="workspace">
      <section class="intro">
        <div class="status-mark" :class="status.phase">
          <Radio v-if="connected" :size="25" />
          <LoaderCircle v-else-if="busy" class="spin" :size="25" />
          <Link v-else :size="25" />
        </div>
        <div>
          <h1>{{ phaseLabel }}</h1>
          <p v-if="connected">打开 Minecraft Java 版多人游戏，房间会自动出现在局域网列表中。</p>
          <p v-else>粘贴好友发送的分享链接，确认后即可加入 Minecraft 世界。</p>
        </div>
        <span class="phase-pill" :class="status.phase">
          {{ connected ? "已连接" : busy ? "处理中" : "未连接" }}
        </span>
      </section>

      <section v-if="!connected && status.phase === 'idle'" class="join-panel">
        <label for="invite">联机邀请</label>
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
          <button class="primary-button" type="button" :disabled="!canPreview" @click="previewInvite">
            继续
            <ArrowRight :size="17" />
          </button>
        </div>
        <p v-if="validationError" class="field-error"><CircleAlert :size="14" />{{ validationError }}</p>
        <div class="privacy-note">
          <Network :size="16" />
          邀请凭证只用于连接房主，不会上传到 SeaLantern 服务。
        </div>
      </section>

      <section v-else class="connection-panel">
        <div class="address-block">
          <span>Minecraft 地址</span>
          <strong>{{ status.localAddress ?? "正在分配本地端口..." }}</strong>
          <button class="copy-button" type="button" :disabled="!status.localAddress" @click="copyAddress">
            <Check v-if="copied" :size="16" />
            <Copy v-else :size="16" />
            {{ copied ? "已复制" : "复制地址" }}
          </button>
        </div>
        <div class="metrics">
          <div><span>连接路径</span><strong>{{ status.route === "direct" ? "P2P 直连" : status.route === "relay" ? "中继" : "检测中" }}</strong></div>
          <div><span>网络延迟</span><strong>{{ status.rttMs == null ? "--" : `${status.rttMs} ms` }}</strong></div>
          <div><span>发送</span><strong>{{ formatBytes(status.txBytes) }}</strong></div>
          <div><span>接收</span><strong>{{ formatBytes(status.rxBytes) }}</strong></div>
        </div>
        <div class="connection-footer">
          <p>{{ status.message ?? "正在同步连接状态..." }}</p>
          <button class="danger-button" type="button" :disabled="busy" @click="stop">
            <Unplug :size="16" />断开
          </button>
        </div>
      </section>

      <p v-if="commandError || status.error" class="error-banner">
        <CircleAlert :size="16" />连接未完成。{{ commandError || status.message || "请检查邀请和网络后重试。" }}
      </p>
    </main>

    <div v-if="confirming" class="modal-backdrop" @click.self="confirming = false">
      <section class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="confirm-title">
        <div class="dialog-icon"><Link :size="23" /></div>
        <h2 id="confirm-title">加入这个 Minecraft 世界？</h2>
        <p>连接后，SeaLantern Connect 会在本机创建一个临时地址，并在多人游戏列表中广播房间。</p>
        <div class="invite-summary"><span>邀请协议</span><strong>sculk / v1</strong></div>
        <div class="dialog-actions">
          <button class="secondary-button" type="button" @click="confirming = false">取消</button>
          <button class="primary-button" type="button" @click="join"><Radio :size="17" />确认加入</button>
        </div>
      </section>
    </div>
  </div>
</template>
