<script lang="ts">
  import { onMount, tick } from "svelte";
  import { markFrontendReady } from "@api/app";
  import { loadPreferences } from "./lib/state";
  import SplashScreen from "./lib/components/shared/SplashScreen.svelte";

  let App = $state<typeof import("./App.svelte").default | null>(null);
  let prepared = $state(false);

  onMount(() => {
    void prepare();
  });

  async function prepare(): Promise<void> {
    await loadPreferences();
    prepared = true;
    void loadApp();
    void revealWindow();
  }

  async function loadApp(): Promise<void> {
    App = (await import("./App.svelte")).default;
  }

  async function revealWindow(): Promise<void> {
    await tick();
    await new Promise<void>((resolve) => window.requestAnimationFrame(() => resolve()));
    await markFrontendReady();
  }
</script>

{#if prepared}
  {#if App}
    <App />
  {:else}
    <SplashScreen loading durationMs={0} onReady={() => undefined} />
  {/if}
{/if}
