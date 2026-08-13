<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { cubicOut } from "svelte/easing";
  import { prefersReducedMotion, Tween } from "svelte/motion";
  import {
    Check,
    ChevronDown,
    Copy,
    Download,
    ExternalLink,
    LockKeyhole,
    LogIn,
    LoaderCircle,
    LogOut,
    Play,
    Plus,
    RefreshCw,
    Shuffle,
    Square,
    Terminal,
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
    onFrpDownloadProgress,
    cacheFrpSnapshot,
    clearCachedFrpSnapshot,
    getCachedFrpSnapshot,
    preloadFrpProvider,
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
  import Dialog from "./ui/Dialog.svelte";
  import Input from "./ui/Input.svelte";
  import Select, { type Option } from "./ui/Select.svelte";

  let { provider } = $props<{ provider: FrpProvider }>();
  const snapshot = untrack(() => getCachedFrpSnapshot(provider));
  let client = $state<FrpClientStatus | null>(snapshot?.client ?? null);
  let session = $state<FrpSessionStatus | null>(snapshot?.session ?? null);
  let tunnels = $state<FrpTunnel[]>(snapshot?.tunnels ?? []);
  let nodes = $state<FrpNode[]>([]);
  let loading = $state(!snapshot);
  let busy = $state(false);
  let downloading = $state(false);
  let downloadProgress = $state(0);
  let creating = $state(false);
  let credential = $state("");
  let selectedTunnelId = $state(snapshot?.tunnels[0]?.id ?? "");
  let selectedNodeId = $state("");
  let tunnelName = $state("");
  let localPort = $state("25565");
  let remotePort = $state("");
  let copied = $state(false);
  let error = $state("");
  let deleteOpen = $state(false);
  let nodesLoading = $state(false);
  let tunnelsLoading = $state(false);
  let sessionTimer: number | null = null;
  let outputLog = $state<HTMLPreElement | null>(null);
  const outputScroll = new Tween(0, { duration: 260, easing: cubicOut });
  let nodeOptions = $derived<Option[]>(
    nodes.map((node) => ({
      label: node.vip ? `${node.name} · VIP` : node.name,
      value: node.id,
    })),
  );
  let selectedTunnel = $derived(tunnels.find((tunnel) => tunnel.id === selectedTunnelId) ?? null);
  let activeTunnel = $derived(
    tunnels.find((tunnel) => tunnel.id === session?.tunnelId) ?? selectedTunnel,
  );
  let selectedNode = $derived(nodes.find((node) => node.id === selectedNodeId) ?? null);
  let remotePortRange = $derived.by(() => {
    const match = selectedNode?.allowPort?.match(/^\(\s*(\d+)\s*,\s*(\d+)\s*\)$/);
    if (!match) return [1, 65535] as const;
    const min = Math.max(1, Number(match[1]));
    const max = Math.min(65535, Number(match[2]));
    return min <= max ? ([min, max] as const) : ([1, 65535] as const);
  });
  let remotePortHint = $derived(
    provider === "open_frp" ? (selectedNode?.allowPort ?? "1-65535") : t("frp.automatic"),
  );
  let activeEndpoint = $derived(activeTunnel?.remoteEndpoint ?? null);
  let outputLength = $derived(session?.output.length ?? 0);
  let validTunnelName = $derived(/^[A-Za-z][A-Za-z0-9_-]{1,31}$/.test(tunnelName.trim()));
  let validRemotePort = $derived.by(() => {
    const value = remotePort.trim();
    if (!/^\d+$/.test(value)) return false;
    const port = Number(value);
    return port >= remotePortRange[0] && port <= remotePortRange[1];
  });
  let validCreate = $derived(
    Boolean(
      selectedNodeId &&
      validTunnelName &&
      Number.isInteger(Number(localPort)) &&
      Number(localPort) >= 1 &&
      Number(localPort) <= 65535 &&
      (provider === "open_frp" ? validRemotePort : !remotePort.trim() || validRemotePort),
    ),
  );

  $effect(() => {
    void provider;
    void load();
  });

  $effect(() => {
    if (!outputLength || !outputLog?.closest("details")?.open) return;
    requestAnimationFrame(scrollOutput);
  });

  $effect(() => {
    if (outputLog) outputLog.scrollTop = outputScroll.current;
  });

  $effect(() => {
    if (client && session) cacheFrpSnapshot(provider, { client, session, tunnels });
  });

  onMount(() => {
    let disposed = false;
    let cleanup: (() => void) | null = null;
    const listener = onFrpDownloadProgress((progress) => {
      if (progress.provider !== provider) return;
      downloading = true;
      downloadProgress = progress.percent;
    });
    void (async () => {
      const unlisten = await listener;
      if (disposed) unlisten();
      else cleanup = unlisten;
    })();
    sessionTimer = window.setInterval(() => {
      if (!session?.authenticated) return;
      void (async () => {
        try {
          session = await getFrpSessionStatus(provider);
        } catch {
          // Keep the last known state while a background poll fails.
        }
      })();
    }, 1000);
    return () => {
      disposed = true;
      cleanup?.();
      if (sessionTimer != null) window.clearInterval(sessionTimer);
    };
  });

  async function load(): Promise<void> {
    if (snapshot) {
      void refreshCachedState();
      return;
    }
    loading = true;
    error = "";
    try {
      await preloadFrpProvider(provider);
      const cached = getCachedFrpSnapshot(provider);
      if (cached) {
        client = cached.client;
        session = cached.session;
        tunnels = cached.tunnels;
        selectedTunnelId = cached.tunnels[0]?.id ?? "";
        return;
      }
      client = await getFrpClientStatus(provider);
      if (!client.installed) return;
      session = await getFrpSessionStatus(provider);
      if (session.authenticated) await loadTunnels();
    } catch (reason) {
      error = String(reason);
    } finally {
      if (client && session) cacheFrpSnapshot(provider, { client, session, tunnels });
      loading = false;
    }
  }

  async function refreshCachedState(): Promise<void> {
    try {
      const nextClient = await getFrpClientStatus(provider);
      if (!nextClient.installed) {
        clearCachedFrpSnapshot(provider);
        client = nextClient;
        session = null;
        tunnels = [];
        selectedTunnelId = "";
        return;
      }
      const nextSession = await getFrpSessionStatus(provider);
      const nextTunnels = nextSession.authenticated ? await listFrpTunnels(provider) : [];
      client = nextClient;
      session = nextSession;
      tunnels = nextTunnels;
      if (!nextTunnels.some((tunnel) => tunnel.id === selectedTunnelId)) {
        selectedTunnelId = nextTunnels[0]?.id ?? "";
      }
    } catch {
      // Keep the cached view when a background refresh cannot reach the provider.
    }
  }

  async function loadTunnels(): Promise<void> {
    tunnelsLoading = true;
    try {
      tunnels = await listFrpTunnels(provider);
      if (!tunnels.some((tunnel) => tunnel.id === selectedTunnelId)) {
        selectedTunnelId = tunnels[0]?.id ?? "";
      }
    } catch (reason) {
      error = String(reason);
    } finally {
      tunnelsLoading = false;
    }
  }
  async function download(): Promise<void> {
    if (downloading) return;
    downloading = true;
    downloadProgress = 0;
    busy = true;
    try {
      client = await downloadFrpClient(provider);
      downloadProgress = 100;
      session = await getFrpSessionStatus(provider);
      if (session.authenticated) await loadTunnels();
      if (client && session) cacheFrpSnapshot(provider, { client, session, tunnels });
    } catch (reason) {
      error = String(reason);
    } finally {
      downloading = false;
      busy = false;
    }
  }
  async function login(): Promise<void> {
    if (busy) return;
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
    if (busy) return;
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
    if (nodes.length > 0) return;
    nodesLoading = true;
    try {
      nodes = await listFrpNodes(provider);
      selectedNodeId = nodes[0]?.id ?? "";
      tunnelName = `SeaLantern_${Math.random().toString(36).slice(2, 8)}`;
    } catch (reason) {
      error = String(reason);
    } finally {
      nodesLoading = false;
    }
  }

  function randomizeRemotePort(): void {
    const [min, max] = remotePortRange;
    remotePort = String(Math.floor(Math.random() * (max - min + 1)) + min);
  }
  async function saveTunnel(): Promise<void> {
    if (!validCreate || busy) return;
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
    if (busy || (!selectedTunnel && !session?.running)) return;
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
    if (!selectedTunnel || busy) return;
    busy = true;
    try {
      tunnels = await deleteFrpTunnel(provider, selectedTunnel.id);
      selectedTunnelId = tunnels[0]?.id ?? "";
      deleteOpen = false;
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
  async function copyEndpoint(): Promise<void> {
    if (!activeEndpoint) return;
    await navigator.clipboard.writeText(activeEndpoint);
    copied = true;
    window.setTimeout(() => (copied = false), 1600);
  }

  function handleOutputToggle(event: Event): void {
    const panel = event.currentTarget;
    if (panel instanceof HTMLDetailsElement && panel.open) requestAnimationFrame(scrollOutput);
  }

  function scrollOutput(): void {
    const element = outputLog;
    if (!element) return;

    const target = Math.max(0, element.scrollHeight - element.clientHeight);
    if (Math.abs(outputScroll.current - element.scrollTop) > 1) {
      void outputScroll.set(element.scrollTop, { duration: 0 });
    }
    void outputScroll.set(target, { duration: prefersReducedMotion.current ? 0 : 260 });
  }
</script>

<div class="workspace frp-view">
  <section class="frp-provider-header">
    <h2>{provider === "open_frp" ? "OpenFRP" : "SakuraFRP"}</h2>
    <p>{provider === "open_frp" ? t("frp.openFrpDescription") : t("frp.sakuraFrpDescription")}</p>
  </section>
  <section class="frp-provider-section">
    {#if loading || !client?.installed}
      <div class="frp-section-heading">
        <div><span>{t("frp.client")}</span><strong>{t("frp.clientManagement")}</strong></div>
      </div>
      {#if loading}
        <div class="frp-checking"><LoaderCircle class="spin" size={18} />{t("frp.checking")}</div>
      {:else}<div class="frp-download-prompt">
          <div>
            <strong>{t("frp.downloadRequired")}</strong>
            <p>
              {t("frp.downloadHint", {
                provider: provider === "open_frp" ? "OpenFRP" : "SakuraFRP",
              })}
            </p>
          </div>
          <Button
            class="primary-button"
            loading={downloading}
            disabled={downloading}
            onclick={download}
            >{#if !downloading}<Download size={17} />{/if}{downloading
              ? t("frp.downloading")
              : t("frp.download")}</Button
          >
        </div>
        {#if downloading}<div class="frp-download-progress" aria-live="polite">
            <div class="frp-download-progress-label">
              <span>{t("frp.downloading")}</span><span>{downloadProgress}%</span>
            </div>
            <div
              class="frp-progress-track"
              role="progressbar"
              aria-valuenow={downloadProgress}
              aria-valuemin="0"
              aria-valuemax="100"
            >
              <div class="frp-progress-value" style={`width: ${downloadProgress}%`}></div>
            </div>
          </div>{/if}{/if}
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
              type="submit"
              disabled={!credential.trim() || busy}
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
            {t("frp.secureCredential")}
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
        <div><span>{t("frp.account")}</span><strong>{t("frp.authorization")}</strong></div>
        <div class="frp-account-row">
          <span class="frp-account-name">{session.accountName ?? "--"}</span>
          <Button variant="ghost" size="sm" title={t("frp.logout")} onclick={logout}
            ><LogOut size={15} /></Button
          >
        </div>
      </div>
      {#if tunnelsLoading && tunnels.length === 0}
        <div class="frp-checking">
          <LoaderCircle class="spin" size={18} />{t("frp.loadingTunnels")}
        </div>
      {:else}
        <div class="frp-tunnel-layout">
          <section class="frp-tunnel-panel">
            <div class="frp-tunnel-panel-head">
              <div class="frp-tunnel-panel-title">
                <strong>{t("frp.tunnels")}</strong>
                <span class="frp-tunnel-count">{tunnels.length}</span>
              </div>
              <div class="frp-tunnel-panel-actions">
                <Button
                  variant="ghost"
                  size="sm"
                  title={t("frp.createTunnel")}
                  onclick={beginCreate}><Plus size={16} /></Button
                ><Button
                  variant="ghost"
                  size="sm"
                  disabled={tunnelsLoading}
                  title={t("frp.refreshTunnels")}
                  onclick={loadTunnels}
                  ><RefreshCw class={tunnelsLoading ? "spin" : ""} size={16} /></Button
                ><Button
                  variant="ghost"
                  size="sm"
                  disabled={!selectedTunnel || busy}
                  title={t("frp.deleteTunnel")}
                  onclick={() => (deleteOpen = true)}><Trash2 size={16} /></Button
                >
              </div>
            </div>
            {#if tunnels.length}<div class="frp-tunnel-list">
                {#each tunnels as tunnel (tunnel.id)}<button
                    class:selected={selectedTunnelId === tunnel.id}
                    class="frp-tunnel-row"
                    type="button"
                    disabled={session.running}
                    onclick={() => (selectedTunnelId = tunnel.id)}
                    ><span class:online={tunnel.online} class="frp-tunnel-state"></span><span
                      ><strong>{tunnel.name}</strong><small>{tunnel.node ?? "--"}</small></span
                    ><code>{tunnel.remoteEndpoint ?? "--"}</code></button
                  >{/each}
              </div>{/if}
          </section>
          <aside class="frp-tunnel-detail">
            {#if activeTunnel}
              <div class="frp-detail-identity">
                <span class:online={session.running || activeTunnel.online} class="frp-tunnel-state"
                ></span>
                <div>
                  <strong>{activeTunnel.name}</strong>
                  <small>{activeTunnel.node ?? "--"}</small>
                </div>
              </div>
              <div class:active={session.running} class="frp-detail-status">
                <span class:online={session.running} class="frp-tunnel-state"
                ></span>{session.running ? t("frp.running") : t("frp.stopped")}
              </div>
              <div class="frp-detail-address">
                <span>{t("frp.publicAddress")}</span>
                <div class="frp-detail-address-row">
                  <code>{activeEndpoint ?? t("frp.addressUnavailable")}</code>
                </div>
              </div>
              <div class="frp-detail-actions">
                <Button variant="outline" disabled={!activeEndpoint} onclick={copyEndpoint}
                  >{#if copied}<Check size={15} />{:else}<Copy size={15} />{/if}{copied
                    ? t("frp.copiedAddress")
                    : t("frp.copyAddress")}</Button
                >
                {#if session.running}<Button
                    variant="danger"
                    disabled={busy}
                    loading={busy}
                    onclick={toggleTunnel}><Square size={15} />{t("frp.stop")}</Button
                  >{:else}<Button disabled={busy} loading={busy} onclick={toggleTunnel}
                    ><Play size={15} />{t("frp.start")}</Button
                  >{/if}
              </div>
            {:else if tunnels.length}
              <div class="frp-detail-empty">
                <p>{t("frp.selectTunnelHint")}</p>
              </div>
            {:else}
              <div class="frp-detail-empty">
                <p>{t("frp.noTunnels")}</p>
                <Button variant="outline" size="sm" onclick={beginCreate}
                  ><Plus size={15} />{t("frp.createTunnel")}</Button
                >
              </div>
            {/if}
          </aside>
        </div>
        {#if session.running}<details class="frp-output-panel" open ontoggle={handleOutputToggle}>
            <summary
              ><span><Terminal size={15} />{t("frp.clientOutput")}</span><ChevronDown
                class="frp-output-chevron"
                size={16}
              /></summary
            >
            <pre bind:this={outputLog} aria-live="polite">{session.output.join("\n")}</pre>
          </details>{/if}
      {/if}
    {/if}
    {#if error}<p class="field-error">{error}</p>{/if}
  </section>
  <Dialog bind:open={creating} title={t("frp.createTunnel")} width="520px">
    <form
      class="frp-create-form"
      onsubmit={(event) => {
        event.preventDefault();
        void saveTunnel();
      }}
    >
      <label
        ><span>{t("frp.node")}</span><Select
          bind:value={selectedNodeId}
          options={nodeOptions}
          disabled={nodesLoading}
          portal
        /></label
      ><label
        ><span>{t("frp.tunnelName")}</span><Input
          bind:value={tunnelName}
          class={tunnelName && !validTunnelName ? "invalid" : ""}
        />{#if tunnelName && !validTunnelName}<small class="frp-field-error"
            >{t("frp.invalidTunnelName")}</small
          >{/if}</label
      ><label
        ><span>{t("frp.localPort")}</span><Input
          bind:value={localPort}
          inputmode="numeric"
        /></label
      ><label
        ><span>{t("frp.remotePort")}</span>
        <div class="frp-port-input">
          <Input
            bind:value={remotePort}
            inputmode="numeric"
            placeholder={remotePortHint}
            class={remotePort && !validRemotePort ? "invalid" : ""}
          />{#if provider === "open_frp"}<button
              class="frp-random-port"
              type="button"
              title={t("frp.randomRemotePort")}
              onclick={randomizeRemotePort}><Shuffle size={16} /></button
            >{/if}
        </div>
        {#if remotePort && !validRemotePort}<small class="frp-field-error"
            >{t("frp.invalidRemotePort", { range: remotePortHint })}</small
          >{/if}</label
      >
      {#if error}<small class="frp-field-error frp-dialog-error">{error}</small>{/if}
      <div class="frp-create-actions">
        <Button variant="outline" type="button" onclick={() => (creating = false)}
          >{t("common.cancel")}</Button
        ><Button type="submit" disabled={!validCreate} loading={busy}
          ><Plus size={16} />{t("frp.createTunnel")}</Button
        >
      </div>
    </form>
  </Dialog>
  <Dialog bind:open={deleteOpen} title={t("frp.deleteTunnel")}>
    <p class="modal-copy">{t("frp.deleteTunnelHint", { name: selectedTunnel?.name ?? "" })}</p>
    {#snippet footer()}
      <Button variant="outline" disabled={busy} onclick={() => (deleteOpen = false)}
        >{t("common.cancel")}</Button
      >
      <Button variant="danger" disabled={busy} loading={busy} onclick={removeTunnel}
        ><Trash2 size={15} />{busy ? t("frp.deletingTunnel") : t("frp.confirmDeleteTunnel")}</Button
      >
    {/snippet}
  </Dialog>
</div>
