<script lang="ts">
  import { onMount, tick } from "svelte";
  import { fade, fly } from "svelte/transition";
  import {
    CircleAlert,
    CircleCheck,
    Cloud,
    Copy,
    Flower2,
    HousePlus,
    Info,
    LoaderCircle,
    LogIn,
    Minus,
    Monitor,
    Moon,
    Palette,
    PanelLeftClose,
    PanelLeftOpen,
    Settings,
    Square,
    Sun,
    Wrench,
    X,
  } from "@lucide/svelte";
  import logoUrl from "./assets/logo.png";
  import { get } from "svelte/store";
  import {
    closeWindow,
    isWindowMaximized,
    minimizeWindow,
    onWindowResized,
    restartApplication,
    toggleMaximize,
  } from "@api/window";
  import {
    checkForAppUpdate,
    installAppUpdate,
    markFrontendReady,
    markPageLoaded,
    type AppUpdate,
  } from "@api/app";
  import { getInitialDeepLinks, getPendingDeepLinks, onDeepLinks } from "@api/deeplink";
  import { preloadFrpProvider } from "@api/frp";
  import { locale, t } from "@i18n";
  import { inviteFromDeepLinkUrls } from "@domain/invitations";
  import type { Preferences } from "@models/preferences";
  import Button from "./lib/components/ui/Button.svelte";
  import Dialog from "./lib/components/ui/Dialog.svelte";
  import HostView from "./lib/components/HostView.svelte";
  import JoinView from "./lib/components/JoinView.svelte";
  import SplashScreen from "./lib/components/shared/SplashScreen.svelte";
  import {
    activeSection,
    changeLocale,
    disposeSession,
    importInvite,
    initializeSession,
    incomingInvite,
    loadPreferences,
    navigate,
    preferences,
    session,
    setTheme,
    sidebarCollapsed,
    startSystemThemeListener,
    type SectionId,
    toasts,
    updatePreferences,
  } from "./lib/state";

  let loading = $state(true);
  let startupReady = $state(false);
  let splash = $state(true);
  let materialPrompt = $state(false);
  let updatePrompt = $state(false);
  let availableUpdate = $state<AppUpdate | null>(null);
  let installingUpdate = $state(false);
  let restarting = $state(false);
  let maximized = $state(false);
  let isMacOS = $state(false);
  let splashDurationMs = $state(1000);
  let currentLocale = $derived($locale);
  let unlistenDeepLinks: (() => void) | null = null;
  let pendingDeepLinkTimer: number | null = null;
  let unlistenResize: (() => void) | null = null;
  let stopSystemThemeListener: (() => void) | null = null;
  let lastDeepLink: { uri: string; receivedAt: number } | null = null;
  let sidebarNav = $state<HTMLElement | null>(null);
  let navIndicator = $state<HTMLElement | null>(null);
  let indicatorFrame: number | null = null;
  let indicatorTimer: number | null = null;
  let PersonalizeView = $state<
    typeof import("./lib/components/PersonalizeView.svelte").default | null
  >(null);
  let ToolboxView = $state<typeof import("./lib/components/ToolboxView.svelte").default | null>(
    null,
  );
  let SettingsView = $state<typeof import("./lib/components/SettingsView.svelte").default | null>(
    null,
  );
  let AboutView = $state<typeof import("./lib/components/AboutView.svelte").default | null>(null);
  let OpenFrpView = $state<typeof import("./lib/components/OpenFrpView.svelte").default | null>(
    null,
  );
  let SakuraFrpView = $state<typeof import("./lib/components/SakuraFrpView.svelte").default | null>(
    null,
  );
  const viewLoads = new Map<SectionId, Promise<void>>();

  const sections = [
    { id: "create", label: "navigation.createRoom", icon: HousePlus },
    { id: "join", label: "navigation.joinRoom", icon: LogIn },
    { id: "openfrp", label: "navigation.createOpenFrp", icon: Cloud },
    { id: "sakurafrp", label: "navigation.createSakuraFrp", icon: Flower2 },
    { id: "toolbox", label: "navigation.toolbox", icon: Wrench },
    { id: "personalize", label: "navigation.personalization", icon: Palette },
    { id: "settings", label: "navigation.settings", icon: Settings },
    { id: "about", label: "navigation.about", icon: Info },
  ] as const;

  const title = $derived(
    t(
      `navigation.${$activeSection === "create" ? "createRoom" : $activeSection === "join" ? "joinRoom" : $activeSection === "openfrp" ? "createOpenFrp" : $activeSection === "sakurafrp" ? "createSakuraFrp" : $activeSection === "toolbox" ? "toolbox" : $activeSection === "personalize" ? "personalization" : $activeSection === "settings" ? "settings" : "about"}`,
    ),
  );
  const p2pState = $derived(
    $session.phase === "active" ? "active" : $session.phase === "idle" ? "idle" : "busy",
  );
  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  onMount(() => {
    void initialize();
    window.addEventListener("resize", updateNavIndicator);
    return () => {
      unlistenDeepLinks?.();
      if (pendingDeepLinkTimer != null) window.clearInterval(pendingDeepLinkTimer);
      unlistenResize?.();
      stopSystemThemeListener?.();
      if (indicatorFrame != null) window.cancelAnimationFrame(indicatorFrame);
      if (indicatorTimer != null) window.clearTimeout(indicatorTimer);
      window.removeEventListener("resize", updateNavIndicator);
      disposeSession();
    };
  });

  async function revealWindow(): Promise<void> {
    await tick();
    await new Promise<void>((resolve) => window.requestAnimationFrame(() => resolve()));
    await markFrontendReady();
  }

  async function initialize(): Promise<void> {
    isMacOS = /Macintosh|Mac OS X/i.test(navigator.userAgent);
    maximized = await isWindowMaximized().catch(() => false);
    unlistenResize = await onWindowResized(async () => {
      maximized = await isWindowMaximized();
    }).catch(() => null);
    try {
      unlistenDeepLinks = await onDeepLinks(handleDeepLinks);
      const urls = await getInitialDeepLinks();
      handleDeepLinks(urls);
      startPendingDeepLinkPolling();
    } catch (error) {
      console.error("Failed to initialize deep links", error);
    }
    await loadPreferences();
    void preloadFrpProvider("open_frp").catch(() => undefined);
    void preloadFrpProvider("sakura_frp").catch(() => undefined);
    splashDurationMs = get(preferences).splashDurationMs;
    if (splashDurationMs === 0) splash = false;
    startupReady = true;
    await revealWindow();
    stopSystemThemeListener = startSystemThemeListener();
    try {
      await initializeSession();
    } catch (error) {
      console.error("Failed to initialize session", error);
    }
    loading = false;
    if (!splash) void tick().then(() => checkForAutomaticUpdate());
  }

  function finishSplash(): void {
    if (!splash) return;
    splash = false;
    scheduleNavIndicator();
    void tick().then(() => checkForAutomaticUpdate());
  }

  async function checkForAutomaticUpdate(): Promise<void> {
    if (!get(preferences).autoUpdate) return;
    try {
      const update = await checkForAppUpdate();
      if (update) {
        availableUpdate = update;
        updatePrompt = true;
      }
    } catch (error) {
      console.error("Failed to check for updates automatically", error);
    }
  }

  async function installAvailableUpdate(): Promise<void> {
    if (!availableUpdate || installingUpdate) return;
    installingUpdate = true;
    try {
      await installAppUpdate(availableUpdate);
    } catch (error) {
      installingUpdate = false;
      console.error("Failed to install update", error);
    }
  }

  function handleDeepLinks(urls: string[]): void {
    const invite = inviteFromDeepLinkUrls(urls);
    if (!invite) return;
    const now = Date.now();
    if (lastDeepLink?.uri === invite && now - lastDeepLink.receivedAt < 1000) return;
    lastDeepLink = { uri: invite, receivedAt: now };
    importInvite(invite);
  }

  function startPendingDeepLinkPolling(): void {
    if (pendingDeepLinkTimer != null) return;
    void pollPendingDeepLinks();
    pendingDeepLinkTimer = window.setInterval(() => {
      void pollPendingDeepLinks();
    }, 500);
  }

  async function pollPendingDeepLinks(): Promise<void> {
    try {
      const urls = await getPendingDeepLinks();
      if (urls.length > 0) handleDeepLinks(urls);
    } catch (error) {
      console.error("Failed to read pending deep links", error);
    }
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

  $effect(() => {
    const section = $activeSection;
    void loadView(section).catch((error) => {
      console.error(`Failed to load ${section} view`, error);
    });
  });

  function loadView(section: SectionId): Promise<void> {
    const existing = viewLoads.get(section);
    if (existing) return existing;

    const load = (async (): Promise<void> => {
      switch (section) {
        case "join":
          break;
        case "personalize":
          PersonalizeView = (await import("./lib/components/PersonalizeView.svelte")).default;
          break;
        case "toolbox":
          ToolboxView = (await import("./lib/components/ToolboxView.svelte")).default;
          break;
        case "settings":
          SettingsView = (await import("./lib/components/SettingsView.svelte")).default;
          break;
        case "about":
          AboutView = (await import("./lib/components/AboutView.svelte")).default;
          break;
        case "openfrp":
          OpenFrpView = (await import("./lib/components/OpenFrpView.svelte")).default;
          break;
        case "sakurafrp":
          SakuraFrpView = (await import("./lib/components/SakuraFrpView.svelte")).default;
          break;
        case "create":
          break;
      }
      if (section !== "create" && section !== "join") {
        void markPageLoaded(section).catch(() => undefined);
      }
    })();
    viewLoads.set(section, load);
    void load.catch(() => viewLoads.delete(section));
    return load;
  }
</script>

{#if !startupReady}
  <div></div>
{:else if splash}
  <div out:fade={{ duration: 0 }}>
    <SplashScreen {loading} durationMs={splashDurationMs} onReady={finishSplash} />
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
                  class:active={p2pState === "active"}
                  class:busy={p2pState === "busy"}
                  class="p2p-dot"
                ></span>{/if}
            </button>
          {/each}
        </div>
        <div class="nav-group nav-group-bottom">
          {#each sections.slice(4, 8) as item (item.id)}
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
        {#key $activeSection}
          <div
            class="page-view"
            in:fly={{
              y: reducedMotion ? 0 : 8,
              duration: reducedMotion ? 0 : 180,
              delay: reducedMotion ? 0 : 100,
            }}
            out:fly={{ y: reducedMotion ? 0 : -4, duration: reducedMotion ? 0 : 100 }}
          >
            {#if $activeSection === "create"}<HostView
                status={$session}
                uriLifetime={$preferences.hostUriLifetime}
                onLifetime={(value) => setPreference("hostUriLifetime", value)}
              />
            {:else if $activeSection === "join"}
              <JoinView
                status={$session}
                savedInvite={$preferences.joinUri}
                savedPort={$preferences.joinPort}
                request={$incomingInvite}
              />
            {:else if $activeSection === "toolbox"}
              {#if ToolboxView}
                <ToolboxView value={$preferences} />
              {:else}
                {@render viewLoading()}
              {/if}
            {:else if $activeSection === "personalize"}
              {#if PersonalizeView}
                <PersonalizeView
                  value={$preferences}
                  onupdate={(update) => updatePreferences(update)}
                />
              {:else}
                {@render viewLoading()}
              {/if}
            {:else if $activeSection === "settings"}
              {#if SettingsView}
                <SettingsView value={$preferences} />
              {:else}
                {@render viewLoading()}
              {/if}
            {:else if $activeSection === "about"}
              {#if AboutView}
                <AboutView />
              {:else}
                {@render viewLoading()}
              {/if}
            {:else if $activeSection === "openfrp"}
              {#if OpenFrpView}
                <OpenFrpView />
              {:else}
                {@render viewLoading()}
              {/if}
            {:else if SakuraFrpView}
              <SakuraFrpView />
            {:else}
              {@render viewLoading()}
            {/if}
          </div>
        {/key}
      </div>
    </main>
  </div>
{/if}

{#snippet viewLoading()}
  <div class="view-loading" role="status" aria-label={t("common.loading")}>
    <LoaderCircle size={24} />
  </div>
{/snippet}

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

<Dialog bind:open={updatePrompt} title={t("update.availableTitle")}>
  <div class="update-dialog-copy">
    <p>{t("update.availableHint", { version: availableUpdate?.version ?? "" })}</p>
  </div>
  {#snippet footer()}<Button
      variant="outline"
      disabled={installingUpdate}
      onclick={() => (updatePrompt = false)}
    >
      {t("update.later")}
    </Button>
    <Button loading={installingUpdate} disabled={installingUpdate} onclick={installAvailableUpdate}>
      {installingUpdate ? t("update.installing") : t("update.install")}
    </Button>{/snippet}
</Dialog>
