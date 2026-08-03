<script lang="ts">
  import { onMount, tick } from "svelte";
  import { fade } from "svelte/transition";
  import {
    ArrowRight,
    Check,
    CircleAlert,
    CircleCheck,
    Cloud,
    Copy,
    Flower2,
    HousePlus,
    Info,
    Link,
    LoaderCircle,
    LogIn,
    Minus,
    Monitor,
    Moon,
    Palette,
    PanelLeftClose,
    PanelLeftOpen,
    RefreshCw,
    Settings,
    Square,
    Sun,
    Unplug,
    X,
  } from "@lucide/svelte";
  import logoUrl from "../assets/logo.png";
  import { get, type Writable } from "svelte/store";
  import { getAppVersion } from "@api/app";
  import {
    closeWindow,
    isWindowMaximized,
    minimizeWindow,
    onWindowResized,
    restartApplication,
    toggleMaximize,
  } from "@api/window";
  import { getInitialDeepLinks, onDeepLinks } from "@api/deeplink";
  import { backendMessage, locale, t } from "@i18n";
  import {
    getLanScan,
    probeHostPort,
    startHost,
    startLanScan,
    startJoin,
    stopJoin,
    stopLanScan,
    stopTunnel,
    validateInvite,
  } from "@api/p2p";
  import {
    downloadFrpClient,
    getFrpClientStatus,
    getFrpSessionStatus,
    listFrpNodes,
    listFrpTunnels,
    loginOpenFrp,
    loginSakuraFrp,
    logoutFrp,
    startFrpTunnel,
    stopFrpTunnel,
    createFrpTunnel,
    deleteFrpTunnel,
  } from "@api/frp";
  import {
    isSameInvite,
    inviteFromDeepLinkUrls,
    normalizeInvite,
    toWebInvite,
  } from "@domain/invitations";
  import type {
    CreateFrpTunnel,
    FrpNode,
    FrpProvider,
    FrpSessionStatus,
    FrpTunnel,
  } from "@models/frp";
  import type { HostUriLifetime, Preferences } from "@models/preferences";
  import type { P2pStatus } from "@models/p2p";
  import { getThemeOptions } from "@themes";
  import { MAX_FONT_SIZE, MIN_FONT_SIZE } from "@themes/typography";
  import Button from "../lib/components/ui/Button.svelte";
  import Dialog from "../lib/components/ui/Dialog.svelte";
  import Input from "../lib/components/ui/Input.svelte";
  import Select, { type Option } from "../lib/components/ui/Select.svelte";
  import Toggle from "../lib/components/ui/Toggle.svelte";
  import HostView from "../lib/components/HostView.svelte";
  import JoinView from "../lib/components/JoinView.svelte";
  import PersonalizeView from "../lib/components/PersonalizeView.svelte";
  import SettingsView from "../lib/components/SettingsView.svelte";
  import AboutView from "../lib/components/AboutView.svelte";
  import OpenFrpView from "../lib/components/OpenFrpView.svelte";
  import SakuraFrpView from "../lib/components/SakuraFrpView.svelte";
  import SplashScreen from "../lib/components/shared/SplashScreen.svelte";
  import {
    activeSection,
    changeLocale,
    consumeInvite,
    disposeSession,
    importInvite,
    initializeSession,
    incomingInvite,
    loadPreferences,
    navigate,
    preferences,
    session,
    setTheme,
    showToast,
    sidebarCollapsed,
    toasts,
    updateApplication,
    updateConnection,
    updateLightweight,
    updatePreferences,
  } from "../lib/state";

  let loading = $state(true);
  let splash = $state(true);
  let materialPrompt = $state(false);
  let restarting = $state(false);
  let maximized = $state(false);
  let isMacOS = $state(false);
  let splashDurationMs = $state(1000);
  let currentLocale = $derived($locale);
  let unlistenDeepLinks: (() => void) | null = null;
  let unlistenResize: (() => void) | null = null;
  let sidebarNav = $state<HTMLElement | null>(null);
  let navIndicator = $state<HTMLElement | null>(null);
  let indicatorFrame: number | null = null;
  let indicatorTimer: number | null = null;

  const sections = [
    { id: "create", label: "navigation.createRoom", icon: HousePlus },
    { id: "join", label: "navigation.joinRoom", icon: LogIn },
    { id: "openfrp", label: "navigation.createOpenFrp", icon: Cloud },
    { id: "sakurafrp", label: "navigation.createSakuraFrp", icon: Flower2 },
    { id: "personalize", label: "navigation.personalization", icon: Palette },
    { id: "settings", label: "navigation.settings", icon: Settings },
    { id: "about", label: "navigation.about", icon: Info },
  ] as const;

  const title = $derived(
    t(
      `navigation.${$activeSection === "create" ? "createRoom" : $activeSection === "join" ? "joinRoom" : $activeSection === "openfrp" ? "createOpenFrp" : $activeSection === "sakurafrp" ? "createSakuraFrp" : $activeSection === "personalize" ? "personalization" : $activeSection === "settings" ? "settings" : "about"}`,
    ),
  );
  const p2pState = $derived(
    $session.phase === "active" ? "active" : $session.phase === "idle" ? "idle" : "busy",
  );

  onMount(() => {
    void initialize();
    window.addEventListener("resize", updateNavIndicator);
    return () => {
      unlistenResize?.();
      unlistenDeepLinks?.();
      if (indicatorFrame != null) window.cancelAnimationFrame(indicatorFrame);
      if (indicatorTimer != null) window.clearTimeout(indicatorTimer);
      window.removeEventListener("resize", updateNavIndicator);
      disposeSession();
    };
  });

  async function initialize(): Promise<void> {
    isMacOS = /Macintosh|Mac OS X/i.test(navigator.userAgent);
    maximized = await isWindowMaximized().catch(() => false);
    unlistenResize = await onWindowResized(async () => {
      maximized = await isWindowMaximized();
    }).catch(() => null);
    try {
      unlistenDeepLinks = await onDeepLinks(handleDeepLinks);
      handleDeepLinks(await getInitialDeepLinks());
    } catch (error) {
      console.error("Failed to initialize deep links", error);
    }
    await loadPreferences();
    try {
      await initializeSession();
    } catch (error) {
      console.error("Failed to initialize session", error);
    }
    loading = false;
    splashDurationMs = get(preferences).splashDurationMs;
  }

  function handleDeepLinks(urls: string[]): void {
    const invite = inviteFromDeepLinkUrls(urls);
    if (invite) importInvite(invite);
  }

  async function restartForMaterialChange(): Promise<void> {
    if (restarting) return;
    restarting = true;
    try {
      await restartApplication();
    } catch (error) {
      restarting = false;
      console.error("Failed to restart application", error);
    }
  }

  function setPreference<K extends keyof Preferences>(key: K, value: Preferences[K]): void {
    const previous = get(preferences)[key];
    updatePreferences({ [key]: value });
    if (key === "windowMaterial" && previous !== "liquid_glass" && value === "liquid_glass")
      materialPrompt = true;
  }

  function sectionLabel(id: string): string {
    return t(sections.find((item) => item.id === id)?.label ?? id);
  }

  function updateNavIndicator(): void {
    if (indicatorFrame != null) window.cancelAnimationFrame(indicatorFrame);
    indicatorFrame = window.requestAnimationFrame(() => {
      indicatorFrame = null;
      const activeItem = sidebarNav?.querySelector<HTMLElement>(".nav-item.active");
      if (!sidebarNav || !navIndicator || !activeItem) return;
      const itemRect = activeItem.getBoundingClientRect();
      const navRect = sidebarNav.getBoundingClientRect();
      const top = itemRect.top - navRect.top + sidebarNav.scrollTop + (itemRect.height - 20) / 2;
      navIndicator.style.opacity = "1";
      navIndicator.style.transform = `translate3d(0, ${top}px, 0)`;
    });
  }

  function scheduleNavIndicator(): void {
    void tick().then(() => {
      if (indicatorTimer != null) window.clearTimeout(indicatorTimer);
      indicatorTimer = window.setTimeout(updateNavIndicator, $sidebarCollapsed ? 250 : 0);
      return undefined;
    });
  }

  $effect(() => {
    void $activeSection;
    void $sidebarCollapsed;
    scheduleNavIndicator();
  });
</script>

{#if splash}
  <div out:fade={{ duration: 250 }}>
    <SplashScreen
      {loading}
      durationMs={splashDurationMs}
      onReady={() => {
        splash = false;
        scheduleNavIndicator();
      }}
    />
  </div>
{:else}
  <div class:sidebar-collapsed={$sidebarCollapsed} class="app-shell">
    <aside class:collapsed={$sidebarCollapsed} class:macos-sidebar={isMacOS} class="sidebar">
      <div class="sidebar-brand" data-tauri-drag-region title="SeaLantern Connect">
        <img src={logoUrl} alt="" draggable="false" />
        <div class="sidebar-brand-name"><strong>SeaLantern</strong><small>Connect</small></div>
      </div>
      <nav
        bind:this={sidebarNav}
        class="sidebar-nav"
        aria-label={t("navigation.main")}
        onscroll={updateNavIndicator}
      >
        <div bind:this={navIndicator} class="nav-active-indicator" aria-hidden="true"></div>
        <div class="nav-group">
          {#each sections.slice(0, 4) as item, index (item.id)}
            {@const Icon = item.icon}
            {#if index === 2}<div class="nav-separator" aria-hidden="true"></div>{/if}
            <button
              class:active={$activeSection === item.id}
              class="nav-item"
              type="button"
              title={sectionLabel(item.id)}
              onclick={() => navigate(item.id)}
            >
              <Icon class="nav-icon" size={19} />
              <span class="nav-label">{sectionLabel(item.id)}</span>
              {#if p2pState !== "idle" && (($session.mode === "host" && item.id === "create") || ($session.mode === "join" && item.id === "join"))}<span
                  class:p2p-active={p2pState === "active"}
                  class:p2p-busy={p2pState === "busy"}
                  class="p2p-dot"
                ></span>{/if}
            </button>
          {/each}
        </div>
        <div class="nav-group nav-group-bottom">
          {#each sections.slice(4, 7) as item (item.id)}
            {@const Icon = item.icon}
            <button
              class:active={$activeSection === item.id}
              class="nav-item"
              type="button"
              title={sectionLabel(item.id)}
              onclick={() => navigate(item.id)}
            >
              <Icon class="nav-icon" size={19} /><span class="nav-label"
                >{sectionLabel(item.id)}</span
              >
            </button>
          {/each}
          <div class="nav-separator"></div>
          <button
            class="nav-item collapse-button"
            type="button"
            title={$sidebarCollapsed
              ? t("navigation.expandSidebar")
              : t("navigation.collapseSidebar")}
            onclick={() => sidebarCollapsed.update((value) => !value)}
          >
            {#if $sidebarCollapsed}<PanelLeftOpen
                class="nav-icon"
                size={19}
              />{:else}<PanelLeftClose class="nav-icon" size={19} />{/if}
            <span class="nav-label">{t("navigation.collapseSidebar")}</span>
          </button>
        </div>
      </nav>
    </aside>

    <header class:macos-overlay={isMacOS} class="titlebar" data-tauri-drag-region>
      <h1 class="page-title" data-tauri-drag-region>{title}</h1>
      <div class="titlebar-actions">
        <button
          class="header-language-button"
          type="button"
          title={currentLocale === "zh-CN"
            ? t("personalization.english")
            : t("personalization.simplifiedChinese")}
          onclick={() => changeLocale(currentLocale === "zh-CN" ? "en" : "zh-CN")}
          ><span aria-hidden="true">{currentLocale === "zh-CN" ? "中" : "EN"}</span></button
        >
        <div class="theme-switcher" role="group" aria-label={t("personalization.theme")}>
          <div
            class="theme-indicator"
            style={`transform: translateX(${["system", "light", "dark"].indexOf($preferences.theme) * 26}px)`}
          ></div>
          <button
            class:active={$preferences.theme === "system"}
            class="theme-button"
            type="button"
            title={t("personalization.followSystem")}
            onclick={() => setTheme("system")}><Monitor size={16} /></button
          >
          <button
            class:active={$preferences.theme === "light"}
            class="theme-button"
            type="button"
            title={t("personalization.light")}
            onclick={() => setTheme("light")}><Sun size={16} /></button
          >
          <button
            class:active={$preferences.theme === "dark"}
            class="theme-button"
            type="button"
            title={t("personalization.dark")}
            onclick={() => setTheme("dark")}><Moon size={16} /></button
          >
        </div>
        {#if !isMacOS}
          <div class="window-controls">
            <button class="window-button" title={t("window.minimize")} onclick={minimizeWindow}
              ><Minus size={12} /></button
            ><button
              class="window-button"
              title={maximized ? t("window.restore") : t("window.maximize")}
              onclick={toggleMaximize}
              >{#if maximized}<Copy size={12} />{:else}<Square size={12} />{/if}</button
            ><button
              class="window-button window-button-close"
              title={t("window.close")}
              onclick={closeWindow}><X size={12} /></button
            >
          </div>
        {/if}
      </div>
    </header>

    <main class="app-content">
      <div class="page-transition-frame">
        {#if $activeSection === "create"}<HostView
            status={$session}
            uriLifetime={$preferences.hostUriLifetime}
            onLifetime={(value) => setPreference("hostUriLifetime", value)}
          />
        {:else if $activeSection === "join"}<JoinView
            status={$session}
            savedInvite={$preferences.joinUri}
            savedPort={$preferences.joinPort}
            request={$incomingInvite}
          />
        {:else if $activeSection === "personalize"}<PersonalizeView
            value={$preferences}
            onupdate={(update) => updatePreferences(update)}
          />
        {:else if $activeSection === "settings"}<SettingsView value={$preferences} />
        {:else if $activeSection === "about"}<AboutView />
        {:else if $activeSection === "openfrp"}<OpenFrpView />
        {:else}<SakuraFrpView />{/if}
      </div>
    </main>
  </div>
{/if}

<div class="sr-only">{currentLocale}</div>
<div class="toast-region" aria-live="polite">
  {#each $toasts as item (item.id)}
    <div class={`toast-item toast-${item.tone}`} role={item.tone === "error" ? "alert" : "status"}>
      <span class="toast-icon"
        >{#if item.tone === "success"}<CircleCheck
            size={17}
          />{:else if item.tone === "error"}<CircleAlert size={17} />{:else}<Info
            size={17}
          />{/if}</span
      >
      <span class="toast-message">{item.message}</span>
      <button
        class="toast-close"
        type="button"
        aria-label={t("common.dismiss")}
        onclick={() => toasts.update((items) => items.filter((value) => value.id !== item.id))}
        ><X size={15} /></button
      >
    </div>
  {/each}
</div>

<Dialog bind:open={materialPrompt} title={t("personalization.restartTitle")}>
  <p>{t("personalization.restartHint")}</p>
  {#snippet footer()}<Button
      disabled={restarting}
      loading={restarting}
      onclick={restartForMaterialChange}>{t("personalization.restartApplication")}</Button
    >{/snippet}
</Dialog>
