<script lang="ts">
  import { RefreshCw, ShieldAlert, ShieldCheck } from "@lucide/svelte";
  import {
    runNetworkDiagnostics,
    runRelayDiagnostics,
    type NetworkDiagnostics,
    type RelayDiagnostics,
  } from "@api/toolbox";
  import { t } from "@i18n";
  import type { Preferences } from "@models/preferences";
  import Button from "./ui/Button.svelte";
  import Dialog from "./ui/Dialog.svelte";

  let { value } = $props<{ value: Preferences }>();
  let network = $state<NetworkDiagnostics | null>(null);
  let relay = $state<RelayDiagnostics | null>(null);
  let networkChecking = $state(false);
  let relayChecking = $state(false);
  let error = $state("");
  let activeTool = $state<"network" | "relay" | null>(null);
  let dialogOpen = $state(false);
  let relayTimer: number | null = null;
  const directStatus = $derived(
    !network
      ? "unknown"
      : !network.udpAvailable || network.mappingVariesByDestination === true
        ? "limited"
        : network.mappingVariesByDestination === false
          ? "available"
          : "unknown",
  );

  async function checkNetwork(): Promise<void> {
    if (networkChecking) return;
    networkChecking = true;
    error = "";
    try {
      network = await runNetworkDiagnostics();
    } catch (reason) {
      showError(reason);
    } finally {
      networkChecking = false;
    }
  }

  async function checkRelay(): Promise<void> {
    if (relayChecking) return;
    relayChecking = true;
    error = "";
    try {
      relay = await runRelayDiagnostics(value.relayCustom ? value.relayUrl : null);
    } catch (reason) {
      showError(reason);
    } finally {
      relayChecking = false;
    }
  }

  function showError(reason: unknown): void {
    const code = reason instanceof Error ? reason.message : "";
    error = t(
      code === "toolbox_network_timeout"
        ? "toolbox.errors.timeout"
        : code === "toolbox_relay_invalid_url"
          ? "toolbox.errors.invalidRelay"
          : code === "toolbox_network_start_failed"
            ? "toolbox.errors.startFailed"
            : "toolbox.errors.failed",
    );
  }

  function resetTool(): void {
    activeTool = null;
    network = null;
    relay = null;
    error = "";
    if (relayTimer != null) window.clearInterval(relayTimer);
    relayTimer = null;
  }

  function openTool(tool: "network" | "relay"): void {
    resetTool();
    activeTool = tool;
    dialogOpen = true;
    if (tool === "network") void checkNetwork();
    if (tool === "relay") {
      void checkRelay();
      relayTimer = window.setInterval(() => void checkRelay(), 1000);
    }
  }

  $effect(() => {
    if (!dialogOpen && activeTool !== null) resetTool();
  });
</script>

<div class="workspace toolbox-workspace">
  <div class="toolbox-grid">
    <section class="tool-card network-card">
      <div class="tool-card-heading">
        <div class="tool-card-title"><h2>{t("toolbox.networkTitle")}</h2></div>
        <Button class="tool-action" onclick={() => openTool("network")}><RefreshCw size={15} />{t("toolbox.check")}</Button>
      </div>
      <p class="tool-card-description">{t("toolbox.networkDescription")}</p>
    </section>
    <section class="tool-card">
      <div class="tool-card-heading"><div class="tool-card-title"><h2>{t("toolbox.relayTitle")}</h2></div>
        <Button class="tool-action" onclick={() => openTool("relay")}><RefreshCw size={15} />{t("toolbox.check")}</Button>
      </div>
      <p class="tool-card-description">{value.relayCustom ? t("toolbox.customRelayHint") : t("toolbox.defaultRelayHint")}</p>
    </section>
  </div>
  {#if error}<p class="field-error toolbox-error">{error}</p>{/if}
</div>

<Dialog bind:open={dialogOpen} title={t(`toolbox.${activeTool ?? "network"}Title`)} width="480px">
  {#if activeTool === "network"}
    <div class="tool-dialog-content">
      {#if network}
        <div class="toolbox-status" class:available={directStatus === "available"} class:limited={directStatus === "limited"}>
          {#if directStatus === "available"}<ShieldCheck size={19} />{:else}<ShieldAlert size={19} />{/if}
          <div><strong>{t(`toolbox.direct.${directStatus}.title`)}</strong><p>{t(`toolbox.direct.${directStatus}.hint`)}</p></div>
        </div>
        <div class="toolbox-mini-results">
          <span>{t("toolbox.publicIpv4")}<strong>{network.publicIpv4 ?? t("toolbox.unavailable")}</strong></span>
          <span>{t("toolbox.publicIpv6")}<strong>{network.publicIpv6 ?? t("toolbox.unavailable")}</strong></span>
          <span>{t("toolbox.udp")}<strong>{network.udpAvailable ? t("toolbox.available") : t("toolbox.unavailable")}</strong></span>
          <span>{t("toolbox.natMapping")}<strong>{network.mappingVariesByDestination === null ? t("toolbox.unknown") : network.mappingVariesByDestination ? t("toolbox.varies") : t("toolbox.stable")}</strong></span>
          <span>{t("toolbox.relay")}<strong>{network.relayAvailable ? t("toolbox.available") : t("toolbox.unavailable")}</strong></span>
        </div>
      {:else}<div class="tool-empty">{networkChecking ? t("toolbox.checking") : t("toolbox.readyHint")}</div>{/if}
    </div>
  {:else if activeTool === "relay"}
    {#if relay}<div class="tool-value-list"><span>{t("toolbox.relayAddress")}<strong>{relay.relayUrl ?? t("toolbox.unavailable")}</strong></span><span>{t("toolbox.latency")}<strong>{relay.latencyMs == null ? t("toolbox.unavailable") : `${relay.latencyMs} ms`}</strong></span></div>{:else}<div class="tool-empty">{relayChecking ? t("toolbox.checking") : t("toolbox.relayReadyHint")}</div>{/if}
  {/if}
</Dialog>

<style>
  .toolbox-workspace { width: min(1000px, calc(100% - 48px)); }
  :global(.tool-action) { min-width: 158px; }
  .toolbox-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 16px; }
  .tool-card { position: relative; display: grid; align-content: start; gap: 13px; min-width: 0; height: 180px; box-sizing: border-box; padding: 18px; border: 1px solid var(--border); border-radius: var(--cmz-radius-sm); background: var(--surface-soft); }
  .tool-card-heading { display: block; }
  .tool-card-heading :global(.tool-action) { position: absolute; right: 18px; bottom: 18px; }
  .tool-card-title { display: flex; align-items: center; gap: 9px; min-width: 0; }
  .tool-card-title h2 { margin: 0; font-size: 1rem; }
  .tool-card-description { min-height: 42px; margin: 0; color: var(--muted); line-height: 1.55; }
  .toolbox-status { display: flex; gap: 10px; padding: 11px; border: 1px solid var(--border); border-radius: var(--cmz-radius-sm); color: var(--muted); }
  .toolbox-status.available { color: var(--success); }
  .toolbox-status.limited { color: var(--warning); }
  .toolbox-status strong { color: var(--text); }
  .toolbox-status p { margin: 4px 0 0; color: var(--muted); font-size: 0.8571rem; line-height: 1.45; }
  .toolbox-mini-results, .tool-value-list { display: grid; gap: 7px; }
  .toolbox-mini-results span, .tool-value-list span { display: flex; justify-content: space-between; gap: 12px; color: var(--muted); font-size: 0.8571rem; }
  .toolbox-mini-results strong, .tool-value-list strong { color: var(--text); font-family: inherit; font-weight: 500; text-align: right; overflow-wrap: anywhere; }
  .tool-empty { display: flex; align-items: center; min-height: 54px; color: var(--muted); font-size: 0.8571rem; line-height: 1.5; }
  .toolbox-error { margin: 0; }
  .tool-dialog-content { display: grid; gap: 16px; }
  :global(.tool-dialog-action) { justify-self: end; }
  @media (max-width: 760px) { .toolbox-workspace { width: calc(100% - 28px); } .toolbox-grid { grid-template-columns: 1fr; } }
</style>
