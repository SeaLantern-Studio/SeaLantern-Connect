<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import {
  Cmz_Button,
  Cmz_Input,
  Cmz_Modal,
  Cmz_Progress,
  Cmz_Select,
  type SelectOption,
} from "cmzya-modern-ui";
import {
  Check,
  ChevronDown,
  Copy,
  Download,
  ExternalLink,
  LoaderCircle,
  LockKeyhole,
  LogIn,
  LogOut,
  Play,
  Plus,
  RefreshCw,
  Shuffle,
  Square,
  Terminal,
  Trash2,
} from "@lucide/vue";
import {
  downloadFrpClient,
  createFrpTunnel,
  deleteFrpTunnel,
  getFrpClientStatus,
  getFrpSessionStatus,
  listFrpTunnels,
  listFrpNodes,
  loginSakuraFrp,
  loginOpenFrp,
  logoutFrp,
  onFrpDownloadProgress,
  openPremium,
  openSakuraKeys,
  openSakuraPurchase,
  startFrpTunnel,
  stopFrpTunnel,
} from "@api";
import { t } from "@i18n";
import { useToastStore } from "../stores/toast";
import type {
  FrpClientStatus,
  FrpNode,
  FrpProvider,
  FrpSessionStatus,
  FrpTunnel,
} from "@models/frp";

const props = defineProps<{
  provider: FrpProvider;
}>();
const toastStore = useToastStore();

const status = ref<FrpClientStatus | null>(null);
const loading = ref(true);
const downloading = ref(false);
const session = ref<FrpSessionStatus | null>(null);
const credential = ref("");
const tunnels = ref<FrpTunnel[]>([]);
const accountBusy = ref(false);
const tunnelsLoading = ref(false);
const selectedTunnelId = ref("");
const creating = ref(false);
const deleteOpen = ref(false);
const deleteBusy = ref(false);
const nodes = ref<FrpNode[]>([]);
const nodesLoading = ref(false);
const selectedNodeId = ref("");
const tunnelName = ref("");
const localPort = ref("25565");
const remotePort = ref("");
const downloadProgress = ref(0);
const copied = ref(false);
const outputLog = ref<HTMLElement | null>(null);
const nodeSelect = ref<HTMLElement | null>(null);
const nodeDropdownWidth = ref("200px");
let unlistenProgress: (() => void) | null = null;
let nodeObserver: ResizeObserver | null = null;
let sessionTimer: number | null = null;
let pollErrorShown = false;

const providerName = computed(() => (props.provider === "open_frp" ? "OpenFRP" : "SakuraFRP"));
const providerDescription = computed(() =>
  t(props.provider === "open_frp" ? "frp.openFrpDescription" : "frp.sakuraFrpDescription"),
);
const nodeOptions = computed<SelectOption[]>(() =>
  nodes.value.map((node) => ({
    label: node.vip ? `${node.name} · VIP` : node.name,
    value: node.id,
  })),
);
const selectedTunnel = computed(() =>
  tunnels.value.find((tunnel) => tunnel.id === selectedTunnelId.value),
);
const selectedNode = computed(() => nodes.value.find((node) => node.id === selectedNodeId.value));
const remotePortRange = computed<[number, number]>(() => {
  const match = selectedNode.value?.allowPort?.match(/^\(\s*(\d+)\s*,\s*(\d+)\s*\)$/);
  if (!match) return [1, 65535];
  const min = Math.max(1, Number(match[1]));
  const max = Math.min(65535, Number(match[2]));
  return min <= max ? [min, max] : [1, 65535];
});
const remotePortHint = computed(() =>
  props.provider === "open_frp" ? (selectedNode.value?.allowPort ?? "1-65535") : t("frp.automatic"),
);
const validRemotePort = computed(() => {
  const remote = remotePort.value.trim();
  const port = Number(remote);
  return (
    /^\d+$/.test(remote) &&
    Number.isInteger(port) &&
    port >= remotePortRange.value[0] &&
    port <= remotePortRange.value[1]
  );
});
const validTunnelName = computed(() =>
  /^[A-Za-z][A-Za-z0-9_-]{1,31}$/.test(tunnelName.value.trim()),
);
const activeTunnel = computed(() =>
  tunnels.value.find((tunnel) => tunnel.id === session.value?.tunnelId),
);
const activeEndpoint = computed(
  () => activeTunnel.value?.remoteEndpoint ?? selectedTunnel.value?.remoteEndpoint ?? null,
);
const validCreateForm = computed(() => {
  const port = Number(localPort.value);
  const remote = remotePort.value.trim();
  return (
    selectedNodeId.value.length > 0 &&
    validTunnelName.value &&
    Number.isInteger(port) &&
    port >= 1 &&
    port <= 65535 &&
    (props.provider === "open_frp"
      ? validRemotePort.value
      : remote.length === 0 || validRemotePort.value)
  );
});

function reportError(reason: unknown): void {
  const message = reason instanceof Error ? reason.message : String(reason);
  toastStore.error(message.replace(/^Error:\s*/, ""));
}

function randomizeRemotePort(): void {
  const [min, max] = remotePortRange.value;
  remotePort.value = String(Math.floor(Math.random() * (max - min + 1)) + min);
}

async function refresh(showError = true): Promise<void> {
  loading.value = true;
  try {
    status.value = await getFrpClientStatus(props.provider);
    session.value = await getFrpSessionStatus(props.provider);
    if (session.value.authenticated) await loadTunnels();
  } catch (reason) {
    if (showError) reportError(reason);
  } finally {
    loading.value = false;
  }
}

async function loadTunnels(): Promise<void> {
  tunnelsLoading.value = true;
  try {
    tunnels.value = await listFrpTunnels(props.provider);
    if (!tunnels.value.some((tunnel) => tunnel.id === selectedTunnelId.value)) {
      selectedTunnelId.value = tunnels.value[0]?.id ?? "";
    }
  } catch (reason) {
    reportError(reason);
  } finally {
    tunnelsLoading.value = false;
  }
}

async function login(): Promise<void> {
  if (!credential.value.trim() || accountBusy.value) return;
  accountBusy.value = true;
  try {
    session.value = await loginSakuraFrp(credential.value);
    credential.value = "";
    await loadTunnels();
  } catch (reason) {
    reportError(reason);
  } finally {
    accountBusy.value = false;
  }
}

async function loginBrowser(): Promise<void> {
  if (accountBusy.value) return;
  accountBusy.value = true;
  try {
    session.value = await loginOpenFrp();
    await loadTunnels();
  } catch (reason) {
    reportError(reason);
  } finally {
    accountBusy.value = false;
  }
}

async function logout(): Promise<void> {
  accountBusy.value = true;
  try {
    session.value = await logoutFrp(props.provider);
    tunnels.value = [];
    selectedTunnelId.value = "";
  } catch (reason) {
    reportError(reason);
  } finally {
    accountBusy.value = false;
  }
}

async function toggleTunnel(): Promise<void> {
  if (!session.value || accountBusy.value) return;
  accountBusy.value = true;
  try {
    session.value = session.value.running
      ? await stopFrpTunnel(props.provider)
      : await startFrpTunnel(props.provider, selectedTunnelId.value);
    if (session.value.running) await loadTunnels();
  } catch (reason) {
    reportError(reason);
  } finally {
    accountBusy.value = false;
  }
}

async function copyEndpoint(): Promise<void> {
  if (!activeEndpoint.value) return;
  await navigator.clipboard.writeText(activeEndpoint.value);
  copied.value = true;
  window.setTimeout(() => (copied.value = false), 1600);
}

async function toggleCreate(): Promise<void> {
  creating.value = !creating.value;
  if (!creating.value || nodes.value.length > 0) return;
  nodesLoading.value = true;
  try {
    nodes.value = await listFrpNodes(props.provider);
    selectedNodeId.value = nodes.value[0]?.id ?? "";
    tunnelName.value = `SeaLantern_${Math.random().toString(36).slice(2, 8)}`;
  } catch (reason) {
    reportError(reason);
  } finally {
    nodesLoading.value = false;
  }
}

async function createTunnel(): Promise<void> {
  if (!validCreateForm.value || accountBusy.value) return;
  accountBusy.value = true;
  try {
    tunnels.value = await createFrpTunnel(props.provider, {
      nodeId: selectedNodeId.value,
      name: tunnelName.value.trim(),
      localPort: Number(localPort.value),
      remotePort: remotePort.value.trim(),
    });
    selectedTunnelId.value = tunnels.value[tunnels.value.length - 1]?.id ?? "";
    creating.value = false;
  } catch (reason) {
    reportError(reason);
  } finally {
    accountBusy.value = false;
  }
}

function askDelete(): void {
  if (!selectedTunnel.value || session.value?.running) return;
  deleteOpen.value = true;
}

async function removeTunnel(): Promise<void> {
  if (!selectedTunnel.value || deleteBusy.value) return;
  deleteBusy.value = true;
  try {
    tunnels.value = await deleteFrpTunnel(props.provider, selectedTunnel.value.id);
    selectedTunnelId.value = tunnels.value[0]?.id ?? "";
    deleteOpen.value = false;
  } catch (reason) {
    reportError(reason);
    deleteOpen.value = false;
  } finally {
    deleteBusy.value = false;
  }
}

async function download(): Promise<void> {
  if (downloading.value) return;
  downloading.value = true;
  downloadProgress.value = 0;
  try {
    status.value = await downloadFrpClient(props.provider);
    downloadProgress.value = 100;
  } catch (reason) {
    reportError(reason);
    await refresh(false);
  } finally {
    downloading.value = false;
  }
}

function syncNodeWidth(): void {
  const width = nodeSelect.value?.getBoundingClientRect().width ?? 0;
  if (width > 0) nodeDropdownWidth.value = `${Math.round(width)}px`;
}

async function scrollOutput(): Promise<void> {
  await nextTick();
  if (outputLog.value) outputLog.value.scrollTop = outputLog.value.scrollHeight;
}

function toggleOutput(event: Event): void {
  const details = event.currentTarget as HTMLDetailsElement;
  if (details.open) void scrollOutput();
}

watch(
  () => props.provider,
  () => refresh(),
);
watch(creating, async (open) => {
  nodeObserver?.disconnect();
  if (!open) return;
  await nextTick();
  syncNodeWidth();
  if (nodeSelect.value) {
    nodeObserver ??= new ResizeObserver(syncNodeWidth);
    nodeObserver.observe(nodeSelect.value);
  }
});
watch(() => session.value?.output.length, scrollOutput);
onMounted(async () => {
  unlistenProgress = await onFrpDownloadProgress((progress) => {
    if (progress.provider === props.provider) downloadProgress.value = progress.percent;
  });
  await refresh();
  sessionTimer = window.setInterval(async () => {
    if (!session.value?.authenticated) return;
    try {
      session.value = await getFrpSessionStatus(props.provider);
      pollErrorShown = false;
    } catch (reason) {
      if (!pollErrorShown) reportError(reason);
      pollErrorShown = true;
    }
  }, 1000);
});
onUnmounted(() => {
  unlistenProgress?.();
  nodeObserver?.disconnect();
  if (sessionTimer !== null) window.clearInterval(sessionTimer);
});
</script>

<template>
  <div class="frp-view">
    <section class="frp-provider-header">
      <div>
        <h2>{{ providerName }}</h2>
        <p>{{ providerDescription }}</p>
      </div>
    </section>

    <section class="frp-provider-section">
      <template v-if="loading || !status?.installed">
        <div class="frp-section-heading">
          <div>
            <span>{{ t("frp.client") }}</span>
            <strong>{{ t("frp.clientManagement") }}</strong>
          </div>
        </div>

        <div v-if="loading" class="frp-checking">
          <LoaderCircle class="spin" :size="18" />
          <span>{{ t("frp.checking") }}</span>
        </div>

        <div v-else-if="status" class="frp-download-prompt">
          <div>
            <strong>{{ t("frp.downloadRequired") }}</strong>
            <p>{{ t("frp.downloadHint", { provider: providerName }) }}</p>
          </div>
          <Cmz_Button
            class="primary-button"
            type="button"
            :disabled="downloading"
            @click="download"
          >
            <LoaderCircle v-if="downloading" class="spin" :size="16" />
            <Download v-else :size="16" />
            {{ downloading ? t("frp.downloading") : t("frp.download") }}
          </Cmz_Button>
        </div>

        <Cmz_Progress
          v-if="downloading"
          :value="downloadProgress"
          :label="t('frp.downloading')"
          :show-percent="true"
        />
      </template>

      <template v-else>
        <template v-if="!session?.authenticated && props.provider === 'open_frp'">
          <div class="frp-connect-main">
            <strong>{{ t("frp.connectOpenFrp") }}</strong>
            <p>{{ t("frp.connectOpenFrpHint") }}</p>
            <Cmz_Button
              class="primary-button"
              type="button"
              :disabled="accountBusy"
              @click="loginBrowser"
            >
              <LoaderCircle v-if="accountBusy" class="spin" :size="16" />
              {{ accountBusy ? t("frp.waitingAuthorization") : t("frp.browserAuthorize") }}
              <ExternalLink v-if="!accountBusy" :size="15" />
            </Cmz_Button>
            <span class="frp-credential-note">
              <LockKeyhole :size="14" />
              {{ t("frp.secureCredential") }}
            </span>
          </div>
          <div class="frp-provider-footer">
            <div>
              <strong>{{ t("frp.openFrpPremium") }}</strong>
              <p>{{ t("frp.premiumDescription") }}</p>
              <small>{{ t("frp.premiumDisclaimer") }}</small>
            </div>
            <div class="frp-provider-links">
              <button type="button" @click="openPremium">
                {{ t("frp.learnPremium") }}
                <ExternalLink :size="14" />
              </button>
            </div>
          </div>
        </template>

        <template v-else-if="!session?.authenticated">
          <form class="frp-connect-main sakura-connect-main" @submit.prevent="login">
            <strong>{{ t("frp.connectSakuraFrp") }}</strong>
            <p>{{ t("frp.connectSakuraFrpHint") }}</p>
            <div class="sakura-login-row">
              <Cmz_Input
                v-model="credential"
                type="password"
                :placeholder="t('frp.sakuraCredential')"
                autocomplete="off"
              />
              <Cmz_Button
                class="primary-button"
                type="submit"
                :disabled="!credential.trim() || accountBusy"
              >
                <LoaderCircle v-if="accountBusy" class="spin" :size="16" />
                <LogIn v-else :size="16" />
                {{ t("frp.authorize") }}
              </Cmz_Button>
              <button class="sakura-key-link" type="button" @click="openSakuraKeys">
                {{ t("frp.getSakuraKey") }}
                <ExternalLink :size="14" />
              </button>
            </div>
            <span class="frp-credential-note">
              <LockKeyhole :size="14" />
              {{ t("frp.savedCredentialHint") }}
            </span>
          </form>
          <div class="frp-provider-footer">
            <div>
              <strong>{{ t("frp.sakuraServices") }}</strong>
              <p>{{ t("frp.sakuraServicesHint") }}</p>
              <small>{{ t("frp.premiumDisclaimer") }}</small>
            </div>
            <div class="frp-provider-links">
              <button type="button" @click="openSakuraPurchase">
                {{ t("frp.buySakuraService") }}
                <ExternalLink :size="14" />
              </button>
            </div>
          </div>
        </template>

        <template v-else>
          <div class="frp-section-heading">
            <div>
              <span>{{ t("frp.account") }}</span>
              <strong>{{ t("frp.authorization") }}</strong>
            </div>
            <span v-if="session?.authenticated" class="frp-account-name">
              {{ session.accountName }}
            </span>
          </div>
          <div v-if="session.running" class="frp-running-view">
            <div class="frp-running-header">
              <div class="frp-running-identity">
                <span class="frp-running-dot"></span>
                <div>
                  <span>{{ t("frp.running") }}</span>
                  <strong>{{ activeTunnel?.name ?? selectedTunnel?.name }}</strong>
                  <small>{{ activeTunnel?.node ?? selectedTunnel?.node }}</small>
                </div>
              </div>
              <Cmz_Button
                class="danger-button"
                type="button"
                :disabled="accountBusy"
                @click="toggleTunnel"
              >
                <LoaderCircle v-if="accountBusy" class="spin" :size="16" />
                <Square v-else :size="15" />
                {{ t("frp.stop") }}
              </Cmz_Button>
            </div>

            <div class="frp-share-address">
              <div>
                <span>{{ t("frp.publicAddress") }}</span>
                <strong>{{ activeEndpoint ?? t("frp.addressUnavailable") }}</strong>
              </div>
              <Cmz_Button
                variant="outline"
                type="button"
                :disabled="!activeEndpoint"
                @click="copyEndpoint"
              >
                <Check v-if="copied" :size="16" />
                <Copy v-else :size="16" />
                {{ copied ? t("frp.copiedAddress") : t("frp.copyAddress") }}
              </Cmz_Button>
            </div>

            <details
              v-if="session.output.length > 0"
              class="frp-output-panel"
              @toggle="toggleOutput"
            >
              <summary>
                <span>
                  <Terminal :size="15" />
                  {{ t("frp.clientOutput") }}
                </span>
                <ChevronDown class="frp-output-chevron" :size="16" />
              </summary>
              <pre ref="outputLog" aria-live="polite">{{ session.output.join("\n") }}</pre>
            </details>
          </div>

          <template v-else>
            <div class="frp-tunnel-toolbar">
              <strong>{{ t("frp.tunnels") }}</strong>
              <div>
                <button
                  class="icon-button"
                  type="button"
                  :title="t('frp.createTunnel')"
                  @click="toggleCreate"
                >
                  <Plus :size="16" />
                </button>
                <button
                  class="icon-button"
                  type="button"
                  :title="t('frp.refreshTunnels')"
                  :disabled="tunnelsLoading"
                  @click="loadTunnels"
                >
                  <RefreshCw :class="{ spin: tunnelsLoading }" :size="16" />
                </button>
                <button
                  class="icon-button frp-delete-button"
                  type="button"
                  :title="t('frp.deleteTunnel')"
                  :disabled="!selectedTunnelId || session.running || deleteBusy"
                  @click="askDelete"
                >
                  <Trash2 :size="16" />
                </button>
                <button class="icon-button" type="button" :title="t('frp.logout')" @click="logout">
                  <LogOut :size="16" />
                </button>
              </div>
            </div>

            <form v-if="creating" class="frp-create-form" @submit.prevent="createTunnel">
              <label>
                <span>{{ t("frp.node") }}</span>
                <div ref="nodeSelect" class="frp-node-select">
                  <Cmz_Select
                    class="settings-select"
                    :model-value="selectedNodeId"
                    :options="nodeOptions"
                    :disabled="nodesLoading"
                    :dropdown-width="nodeDropdownWidth"
                    searchable
                    @update:model-value="selectedNodeId = String($event)"
                  />
                </div>
              </label>
              <label>
                <span>{{ t("frp.tunnelName") }}</span>
                <Cmz_Input v-model="tunnelName" />
                <small v-if="tunnelName && !validTunnelName" class="frp-field-error">
                  {{ t("frp.invalidTunnelName") }}
                </small>
              </label>
              <label>
                <span>{{ t("frp.localPort") }}</span>
                <Cmz_Input v-model="localPort" inputmode="numeric" />
              </label>
              <label>
                <span>{{ t("frp.remotePort") }}</span>
                <Cmz_Input v-model="remotePort" inputmode="numeric" :placeholder="remotePortHint">
                  <template v-if="props.provider === 'open_frp'" #suffix>
                    <Cmz_Button
                      variant="ghost"
                      size="sm"
                      :icon-only="true"
                      type="button"
                      :title="t('frp.randomRemotePort')"
                      @click="randomizeRemotePort"
                    >
                      <Shuffle :size="16" />
                    </Cmz_Button>
                  </template>
                </Cmz_Input>
                <small v-if="remotePort && !validRemotePort" class="frp-field-error">
                  {{ t("frp.invalidRemotePort", { range: remotePortHint }) }}
                </small>
              </label>
              <div class="frp-create-actions">
                <Cmz_Button variant="outline" type="button" @click="creating = false">
                  {{ t("common.cancel") }}
                </Cmz_Button>
                <Cmz_Button
                  class="primary-button"
                  type="submit"
                  :disabled="!validCreateForm || accountBusy"
                >
                  <LoaderCircle v-if="accountBusy" class="spin" :size="16" />
                  <Plus v-else :size="16" />
                  {{ t("frp.createTunnel") }}
                </Cmz_Button>
              </div>
            </form>

            <template v-if="!creating">
              <div v-if="tunnelsLoading && tunnels.length === 0" class="frp-checking">
                <LoaderCircle class="spin" :size="18" />
                <span>{{ t("frp.loadingTunnels") }}</span>
              </div>
              <div v-else-if="tunnels.length > 0" class="frp-tunnel-list">
                <button
                  v-for="tunnel in tunnels"
                  :key="tunnel.id"
                  class="frp-tunnel-row"
                  :class="{ selected: selectedTunnelId === tunnel.id }"
                  type="button"
                  :disabled="session.running"
                  @click="selectedTunnelId = tunnel.id"
                >
                  <span class="frp-tunnel-state" :class="{ online: tunnel.online }"></span>
                  <span>
                    <strong>{{ tunnel.name }}</strong>
                    <small>{{ tunnel.node ?? `#${tunnel.id}` }}</small>
                  </span>
                  <code>{{ tunnel.remoteEndpoint ?? "--" }}</code>
                </button>
              </div>
              <p v-else>{{ t("frp.noTunnels") }}</p>
            </template>

            <div v-if="!creating && tunnels.length > 0" class="frp-session-actions">
              <span>
                {{ t("frp.stopped") }}
              </span>
              <Cmz_Button
                class="primary-button"
                type="button"
                :disabled="accountBusy || !selectedTunnelId"
                @click="toggleTunnel"
              >
                <LoaderCircle v-if="accountBusy" class="spin" :size="16" />
                <Play v-else :size="16" />
                {{ t("frp.start") }}
              </Cmz_Button>
            </div>
          </template>
        </template>
      </template>
    </section>

    <Cmz_Modal
      :visible="deleteOpen"
      :title="t('frp.deleteTunnel')"
      width="420px"
      :close-on-overlay="!deleteBusy"
      @close="!deleteBusy && (deleteOpen = false)"
    >
      <p class="modal-copy">
        {{ t("frp.deleteTunnelHint", { name: selectedTunnel?.name ?? "" }) }}
      </p>
      <template #footer>
        <Cmz_Button
          variant="outline"
          type="button"
          :disabled="deleteBusy"
          @click="deleteOpen = false"
        >
          {{ t("common.cancel") }}
        </Cmz_Button>
        <Cmz_Button
          class="danger-button"
          type="button"
          :disabled="deleteBusy"
          @click="removeTunnel"
        >
          <LoaderCircle v-if="deleteBusy" class="spin" :size="16" />
          <Trash2 v-else :size="16" />
          {{ deleteBusy ? t("frp.deletingTunnel") : t("frp.confirmDeleteTunnel") }}
        </Cmz_Button>
      </template>
    </Cmz_Modal>
  </div>
</template>
