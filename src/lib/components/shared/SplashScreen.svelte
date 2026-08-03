<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "@i18n";
  import logoUrl from "../../../assets/logo.png";
  import "./SplashScreen.css";

  let { loading, durationMs, onReady } = $props<{
    loading: boolean;
    durationMs: number;
    onReady: () => void;
  }>();

  let logoScale = $state(0);
  let contentVisible = $state(false);
  let animationComplete = $state(false);
  let startedAt = 0;
  let logoTimer: number | null = null;
  let contentTimer: number | null = null;
  let completionTimer: number | null = null;

  function finishWhenReady(): void {
    if (animationComplete && !loading) onReady();
  }

  function scheduleCompletion(): void {
    if (completionTimer != null) window.clearTimeout(completionTimer);
    const elapsedMs = performance.now() - startedAt;
    completionTimer = window.setTimeout(
      () => {
        animationComplete = true;
        finishWhenReady();
      },
      Math.max(0, durationMs - elapsedMs),
    );
  }

  $effect(() => {
    if (startedAt === 0) return;
    scheduleCompletion();
    finishWhenReady();
  });

  onMount(() => {
    startedAt = performance.now();
    logoTimer = window.setTimeout(() => (logoScale = 1), 50);
    contentTimer = window.setTimeout(() => (contentVisible = true), 200);
    scheduleCompletion();
    return () => {
      if (logoTimer != null) window.clearTimeout(logoTimer);
      if (contentTimer != null) window.clearTimeout(contentTimer);
      if (completionTimer != null) window.clearTimeout(completionTimer);
    };
  });
</script>

<div class="splash-screen">
  <div class="splash-content">
    <div class="splash-logo" style={`transform: scale(${logoScale})`}>
      <img src={logoUrl} alt="SeaLantern Connect" width="96" height="96" />
    </div>
    <div class:visible={contentVisible} class="splash-text">
      <h1>SeaLantern Connect</h1>
      <p>{t("splash.subtitle")}</p>
    </div>
    <div class:visible={contentVisible} class="splash-loader" aria-label={t("splash.starting")}>
      <span></span><span></span><span></span>
    </div>
  </div>
</div>
