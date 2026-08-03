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
  Download,
  ExternalLink,
  LoaderCircle,
  LockKeyhole,
  LogIn,
  LogOut,
  Play,
  Plus,
  RefreshCw,
  Square,
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
  loginFrp,
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

const status = ref<FrpClientStatus | null>(null);
const loading = ref(true);
const downloading = ref(false);
const error = ref("");
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
const nodeSelect = ref<HTMLElement | null>(null);
const nodeDropdownWidth = ref("200px");
let unlistenProgress: (() => void) | null = null;
let nodeObserver: ResizeObserver | null = null;

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
const validCreateForm = computed(() => {
  const port = Number(localPort.value);
  const remote = remotePort.value.trim();
  return (
    selectedNodeId.value.length > 0 &&
    tunnelName.value.trim().length > 0 &&
    Number.isInteger(port) &&
    port >= 1 &&
    port <= 65535 &&
    (remote.length === 0 ||
      (/^\d+$/.test(remote) && Number(remote) >= 1 && Number(remote) <= 65535))
  );
});

async function refresh(): Promise<void> {
  loading.value = true;
  error.value = "";
  try {
    status.value = await getFrpClientStatus(props.provider);
    session.value = await getFrpSessionStatus(props.provider);
    if (session.value.authenticated) await loadTunnels();
  } catch (reason) {
    error.value = String(reason);
  } finally {
    loading.value = false;
  }
}

async function loadTunnels(): Promise<void> {
  tunnelsLoading.value = true;
  error.value = "";
  try {
    tunnels.value = await listFrpTunnels(props.provider);
    if (!tunnels.value.some((tunnel) => tunnel.id === selectedTunnelId.value)) {
      selectedTunnelId.value = tunnels.value[0]?.id ?? "";
    }
  } catch (reason) {
    error.value = String(reason);
  } finally {
    tunnelsLoading.value = false;
  }
}

async function login(): Promise<void> {
  if (!credential.value.trim() || accountBusy.value) return;
  accountBusy.value = true;
  error.value = "";
  try {
    session.value = await loginFrp(props.provider, credential.value);
    credential.value = "";
    await loadTunnels();
  } catch (reason) {
    error.value = String(reason);
  } finally {
    accountBusy.value = false;
  }
}

async function loginBrowser(): Promise<void> {
  if (accountBusy.value) return;
  accountBusy.value = true;
  error.value = "";
  try {
    session.value = await loginOpenFrp();
    await loadTunnels();
  } catch (reason) {
    error.value = String(reason);
  } finally {
    accountBusy.value = false;
  }
}

async function logout(): Promise<void> {
  accountBusy.value = true;
  error.value = "";
  try {
    session.value = await logoutFrp(props.provider);
    tunnels.value = [];
    selectedTunnelId.value = "";
  } catch (reason) {
    error.value = String(reason);
  } finally {
    accountBusy.value = false;
  }
}

async function toggleTunnel(): Promise<void> {
  if (!session.value || accountBusy.value) return;
  accountBusy.value = true;
  error.value = "";
  try {
    session.value = session.value.running
      ? await stopFrpTunnel(props.provider)
      : await startFrpTunnel(props.provider, selectedTunnelId.value);
  } catch (reason) {
    error.value = String(reason);
  } finally {
    accountBusy.value = false;
  }
}

async function toggleCreate(): Promise<void> {
  creating.value = !creating.value;
  if (!creating.value || nodes.value.length > 0) return;
  nodesLoading.value = true;
  error.value = "";
  try {
    nodes.value = await listFrpNodes(props.provider);
    selectedNodeId.value = nodes.value[0]?.id ?? "";
    tunnelName.value = `SeaLantern_${Math.random().toString(36).slice(2, 8)}`;
  } catch (reason) {
    error.value = String(reason);
  } finally {
    nodesLoading.value = false;
  }
}

async function createTunnel(): Promise<void> {
  if (!validCreateForm.value || accountBusy.value) return;
  accountBusy.value = true;
  error.value = "";
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
    error.value = String(reason);
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
  error.value = "";
  try {
    tunnels.value = await deleteFrpTunnel(props.provider, selectedTunnel.value.id);
    selectedTunnelId.value = tunnels.value[0]?.id ?? "";
    deleteOpen.value = false;
  } catch (reason) {
    error.value = String(reason);
    deleteOpen.value = false;
  } finally {
    deleteBusy.value = false;
  }
}

async function download(): Promise<void> {
  if (downloading.value) return;
  downloading.value = true;
  downloadProgress.value = 0;
  error.value = "";
  try {
    status.value = await downloadFrpClient(props.provider);
    downloadProgress.value = 100;
  } catch (reason) {
    error.value = String(reason);
    await refresh();
  } finally {
    downloading.value = false;
  }
}

function syncNodeWidth(): void {
  const width = nodeSelect.value?.getBoundingClientRect().width ?? 0;
  if (width > 0) nodeDropdownWidth.value = `${Math.round(width)}px`;
}

watch(() => props.provider, refresh);
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
onMounted(async () => {
  unlistenProgress = await onFrpDownloadProgress((progress) => {
    if (progress.provider === props.provider) downloadProgress.value = progress.percent;
  });
  await refresh();
});
onUnmounted(() => {
  unlistenProgress?.();
  nodeObserver?.disconnect();
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

        <p v-if="error" class="form-error" role="alert">{{ error }}</p>
      </template>

      <template v-else>
        <template v-if="!session?.authenticated && props.provider === 'open_frp'">
          <div class="frp-connect-main">
            <strong>{{ t("frp.connectOpenFrp") }}</strong>
            <p>{{ t("frp.connectOpenFrpHint") }}</p>
            <p v-if="error" class="form-error" role="alert">{{ error }}</p>
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
            <p v-if="error" class="form-error" role="alert">{{ error }}</p>
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

          <p v-if="error" class="form-error" role="alert">{{ error }}</p>

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
            </label>
            <label>
              <span>{{ t("frp.localPort") }}</span>
              <Cmz_Input v-model="localPort" inputmode="numeric" />
            </label>
            <label>
              <span>{{ t("frp.remotePort") }}</span>
              <Cmz_Input
                v-model="remotePort"
                inputmode="numeric"
                :placeholder="t('frp.automatic')"
              />
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
            <span :class="{ active: session.running }">
              {{ session.running ? t("frp.running") : t("frp.stopped") }}
            </span>
            <Cmz_Button
              :class="session.running ? 'danger-button' : 'primary-button'"
              type="button"
              :disabled="accountBusy || (!session.running && !selectedTunnelId)"
              @click="toggleTunnel"
            >
              <LoaderCircle v-if="accountBusy" class="spin" :size="16" />
              <Square v-else-if="session.running" :size="15" />
              <Play v-else :size="16" />
              {{ session.running ? t("frp.stop") : t("frp.start") }}
            </Cmz_Button>
          </div>
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
