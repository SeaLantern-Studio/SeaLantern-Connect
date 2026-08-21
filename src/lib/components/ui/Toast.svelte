<script lang="ts">
  import { Toaster } from "svelte-sonner";
</script>

<Toaster
  position="bottom-right"
  theme="system"
  richColors={false}
  visibleToasts={3}
  toastOptions={{ class: "sl-toast" }}
/>

<style>
  :global([data-sonner-toaster]) {
    z-index: 1200;
    --width: min(360px, calc(100vw - 32px));
    font-family: var(--app-font-family) !important;
    --normal-border: var(--border);
    --normal-text: var(--primary);
  }
  :global([data-sonner-toast].sl-toast) {
    width: fit-content !important;
    min-width: min(240px, calc(100vw - 32px));
    max-width: min(360px, calc(100vw - 32px));
    white-space: normal;
    overflow-wrap: anywhere;
    --normal-text: var(--primary) !important;
    color: var(--primary) !important;
    font-family: var(--app-font-family) !important;
    backdrop-filter: blur(18px) saturate(1.08);
    box-shadow: var(--shadow);
  }
  :global([data-sonner-toast].sl-toast [data-icon]) {
    color: var(--primary) !important;
  }
</style>
