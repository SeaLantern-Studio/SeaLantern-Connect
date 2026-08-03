<script lang="ts">
  import {
    Check,
    ChevronDown,
    Copy,
    ExternalLink,
    LockKeyhole,
    LogIn,
    LoaderCircle,
    LogOut,
    Plus,
    RefreshCw,
    Trash2,
  } from "@lucide/svelte";
  import {
    createFrpTunnel,
    deleteFrpTunnel,
    downloadFrpClient,
    getFrpClientStatus,
    getFrpSessionStatus,
    listFrpNodes,
    listFrpTunnels,
    loginOpenFrp,
    loginSakuraFrp,
    logoutFrp,
    openPremium,
    openSakuraKeys,
    openSakuraPurchase,
    startFrpTunnel,
    stopFrpTunnel,
  } from "@api/frp";
  import { t } from "@i18n";
  import type {
    FrpClientStatus,
    FrpNode,
    FrpProvider,
    FrpSessionStatus,
    FrpTunnel,
  } from "@models/frp";
  import Button from "./ui/Button.svelte";
  import Input from "./ui/Input.svelte";
  import Select, { type Option } from "./ui/Select.svelte";

  let { provider } = $props<{ provider: FrpProvider }>();
  let client = $state<FrpClientStatus | null>(null);
  let session = $state<FrpSessionStatus | null>(null);
  let tunnels = $state<FrpTunnel[]>([]);
  let nodes = $state<FrpNode[]>([]);
  let loading = $state(true);
  let busy = $state(false);
  let creating = $state(false);
  let credential = $state("");
  let selectedTunnelId = $state("");
  let selectedNodeId = $state("");
  let tunnelName = $state("");
  let localPort = $state("25565");
  let remotePort = $state("");
  let copied = $state(false);
  let error = $state("");
  let nodeOptions = $derived<Option[]>(
    nodes.map((node) => ({
      label: node.vip ? `${node.name} · VIP` : node.name,
      value: node.id,
    })),
  );
  let selectedTunnel = $derived(tunnels.find((tunnel) => tunnel.id === selectedTunnelId) ?? null);
  let validCreate = $derived(
    Boolean(
      selectedNodeId &&
      tunnelName.trim() &&
      Number.isInteger(Number(localPort)) &&
      Number(localPort) >= 1 &&
      Number(localPort) <= 65535,
    ),
  );

  $effect(() => {
    void provider;
    void load();
  });
  async function load(): Promise<void> {
    loading = true;
    error = "";
    try {
      client = await getFrpClientStatus(provider);
      if (!client.installed) return;
      session = await getFrpSessionStatus(provider);
      if (session.authenticated) await loadTunnels();
    } catch (reason) {
      error = String(reason);
    } finally {
      loading = false;
    }
  }
  async function loadTunnels(): Promise<void> {
    try {
      tunnels = await listFrpTunnels(provider);
      if (!tunnels.some((tunnel) => tunnel.id === selectedTunnelId)) {
        selectedTunnelId = tunnels[0]?.id ?? "";
      }
    } catch (reason) {
      error = String(reason);
    }
  }
  async function download(): Promise<void> {
    busy = true;
    try {
      client = await downloadFrpClient(provider);
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
  async function login(): Promise<void> {
    busy = true;
    error = "";
    try {
      session =
        provider === "open_frp" ? await loginOpenFrp() : await loginSakuraFrp(credential.trim());
      credential = "";
      if (session.authenticated) await loadTunnels();
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
  async function openExternal(action: () => Promise<void>): Promise<void> {
    try {
      await action();
    } catch (reason) {
      error = String(reason);
    }
  }
  async function logout(): Promise<void> {
    busy = true;
    try {
      session = await logoutFrp(provider);
      tunnels = [];
      selectedTunnelId = "";
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
  async function beginCreate(): Promise<void> {
    creating = true;
    error = "";
    try {
      nodes = await listFrpNodes(provider);
      selectedNodeId = nodes[0]?.id ?? "";
    } catch (reason) {
      error = String(reason);
    }
  }
  async function saveTunnel(): Promise<void> {
    if (!validCreate) return;
    busy = true;
    try {
      tunnels = await createFrpTunnel(provider, {
        nodeId: selectedNodeId,
        name: tunnelName.trim(),
        localPort: Number(localPort),
        remotePort: remotePort.trim(),
      });
      creating = false;
      selectedTunnelId = tunnels.at(-1)?.id ?? "";
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
  async function toggleTunnel(): Promise<void> {
    if (!selectedTunnel && !session?.running) return;
    busy = true;
    try {
      session = session?.running
        ? await stopFrpTunnel(provider)
        : await startFrpTunnel(provider, selectedTunnel!.id);
      await loadTunnels();
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
  async function removeTunnel(): Promise<void> {
    if (!selectedTunnel) return;
    busy = true;
    try {
      tunnels = await deleteFrpTunnel(provider, selectedTunnel.id);
      selectedTunnelId = tunnels[0]?.id ?? "";
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
  async function copyEndpoint(): Promise<void> {
    if (!selectedTunnel?.remoteEndpoint) return;
    await navigator.clipboard.writeText(selectedTunnel.remoteEndpoint);
    copied = true;
    window.setTimeout(() => (copied = false), 1600);
  }
</script>

<div class="workspace frp-view">
  <section class="frp-provider-header">
    <h2>{provider === "open_frp" ? "OpenFRP" : "SakuraFRP"}</h2>
    <p>{provider === "open_frp" ? t("frp.openFrpDescription") : t("frp.sakuraFrpDescription")}</p>
  </section>
  <section class="frp-provider-section">
    {#if loading}<div class="frp-checking">
        <LoaderCircle class="spin" size={18} />{t("frp.checking")}
      </div>
    {:else if !client?.installed}<div class="frp-download-prompt">
        <div>
          <strong>{t("frp.downloadRequired")}</strong>
          <p>
            {t("frp.downloadHint", { provider: provider === "open_frp" ? "OpenFRP" : "SakuraFRP" })}
          </p>
        </div>
        <Button class="primary-button" loading={busy} onclick={download}>{t("frp.download")}</Button
        >
      </div>
    {:else if !session?.authenticated}
      {#if provider === "open_frp"}<div class="frp-connect-main">
          <strong>{t("frp.connectOpenFrp")}</strong>
          <p>{t("frp.connectOpenFrpHint")}</p>
          <Button class="primary-button" loading={busy} onclick={() => void login()}>
            {busy ? t("frp.waitingAuthorization") : t("frp.browserAuthorize")}
            {#if !busy}<ExternalLink size={15} />{/if}
          </Button>
          <span class="frp-credential-note">
            <LockKeyhole size={14} />
            {t("frp.secureCredential")}
          </span>
        </div>
        <div class="frp-provider-footer">
          <div>
            <strong>{t("frp.openFrpPremium")}</strong>
            <p>{t("frp.premiumDescription")}</p>
            <small>{t("frp.premiumDisclaimer")}</small>
          </div>
          <div class="frp-provider-links">
            <button type="button" onclick={() => void openExternal(openPremium)}>
              {t("frp.learnPremium")}
              <ExternalLink size={14} />
            </button>
          </div>
        </div>
      {:else}<form
          class="frp-connect-main sakura-connect-main"
          onsubmit={(event) => {
            event.preventDefault();
            void login();
          }}
        >
          <strong>{t("frp.connectSakuraFrp")}</strong>
          <p>{t("frp.connectSakuraFrpHint")}</p>
          <div class="sakura-login-row">
            <Input
              bind:value={credential}
              type="password"
              placeholder={t("frp.sakuraCredential")}
              autocomplete="off"
            />
            <Button
              class="primary-button"
              loading={busy}
              disabled={!credential.trim() || busy}
              onclick={() => void login()}
            >
              {#if !busy}<LogIn size={16} />{/if}{t("frp.authorize")}
            </Button>
            <button
              class="sakura-key-link"
              type="button"
              onclick={() => void openExternal(openSakuraKeys)}
            >
              {t("frp.getSakuraKey")}
              <ExternalLink size={14} />
            </button>
          </div>
          <span class="frp-credential-note">
            <LockKeyhole size={14} />
            {t("frp.savedCredentialHint")}
          </span>
        </form>
        <div class="frp-provider-footer">
          <div>
            <strong>{t("frp.sakuraServices")}</strong>
            <p>{t("frp.sakuraServicesHint")}</p>
            <small>{t("frp.premiumDisclaimer")}</small>
          </div>
          <div class="frp-provider-links">
            <button type="button" onclick={() => void openExternal(openSakuraPurchase)}>
              {t("frp.buySakuraService")}
              <ExternalLink size={14} />
            </button>
          </div>
        </div>{/if}
    {:else}
      <div class="frp-section-heading">
        <div><span>{t("frp.account")}</span><strong>{session.accountName ?? "--"}</strong></div>
        <Button variant="ghost" size="sm" onclick={logout}
          ><LogOut size={15} />{t("frp.logout")}</Button
        >
      </div>
      {#if session.running}<div class="frp-running-view">
          <div class="frp-running-header">
            <div class="frp-running-identity">
              <span class="frp-running-dot"></span>
              <div>
                <strong>{selectedTunnel?.name ?? t("frp.tunnels")}</strong><small
                  >{t("frp.running")}</small
                >
              </div>
            </div>
            <Button variant="danger" loading={busy} onclick={toggleTunnel}>{t("frp.stop")}</Button>
          </div>
          {#if selectedTunnel?.remoteEndpoint}<div class="frp-share-address">
              <div>
                <span>{t("frp.publicAddress")}</span><strong>{selectedTunnel.remoteEndpoint}</strong
                >
              </div>
              <Button variant="outline" size="sm" onclick={copyEndpoint}
                >{#if copied}<Check size={15} />{:else}<Copy size={15} />{/if}{t(
                  "frp.copyAddress",
                )}</Button
              >
            </div>{/if}
          <details class="frp-output-panel">
            <summary>{t("frp.clientOutput")}<ChevronDown size={16} /></summary>
            <pre>{session.output.join("\n")}</pre>
          </details>
        </div>{/if}
      <div class="frp-tunnel-toolbar">
        <strong>{t("frp.tunnels")}</strong>
        <div>
          <Button variant="ghost" size="sm" title={t("frp.createTunnel")} onclick={beginCreate}
            ><Plus size={16} /></Button
          ><Button variant="ghost" size="sm" title={t("frp.refreshTunnels")} onclick={loadTunnels}
            ><RefreshCw size={16} /></Button
          ><Button
            variant="ghost"
            size="sm"
            disabled={!selectedTunnel}
            title={t("frp.deleteTunnel")}
            onclick={removeTunnel}><Trash2 size={16} /></Button
          >
        </div>
      </div>
      {#if creating}<div class="frp-create-form">
          <label
            ><span>{t("frp.node")}</span><Select
              bind:value={selectedNodeId}
              options={nodeOptions}
            /></label
          ><label><span>{t("frp.tunnelName")}</span><Input bind:value={tunnelName} /></label><label
            ><span>{t("frp.localPort")}</span><Input
              bind:value={localPort}
              inputmode="numeric"
            /></label
          ><label
            ><span>{t("frp.remotePort")}</span><Input
              bind:value={remotePort}
              inputmode="numeric"
            /></label
          >
          <div class="frp-create-actions">
            <Button variant="outline" onclick={() => (creating = false)}
              >{t("common.cancel")}</Button
            ><Button disabled={!validCreate} loading={busy} onclick={saveTunnel}
              >{t("frp.createTunnel")}</Button
            >
          </div>
        </div>
      {:else if tunnels.length}<div class="frp-tunnel-list">
          {#each tunnels as tunnel (tunnel.id)}<button
              class:selected={selectedTunnelId === tunnel.id}
              class="frp-tunnel-row"
              type="button"
              onclick={() => (selectedTunnelId = tunnel.id)}
              ><span class:online={tunnel.online} class="frp-tunnel-state"></span><span
                ><strong>{tunnel.name}</strong><small>{tunnel.node ?? "--"}</small></span
              ><code>{tunnel.remoteEndpoint ?? "--"}</code></button
            >{/each}
        </div>{:else}<p>{t("frp.noTunnels")}</p>{/if}
      {#if !creating && selectedTunnel}<div class="frp-session-actions">
          <span class:active={session.running}
            >{session.running ? t("frp.running") : t("frp.stopped")}</span
          ><Button disabled={busy} loading={busy} onclick={toggleTunnel}
            >{session.running ? t("frp.stop") : t("frp.start")}</Button
          >
        </div>{/if}
    {/if}
    {#if error}<p class="field-error">{error}</p>{/if}
  </section>
</div>
