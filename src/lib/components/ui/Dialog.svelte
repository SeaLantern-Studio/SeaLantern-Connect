<script lang="ts">
  import { Dialog } from "bits-ui";
  let {
    open = $bindable(false),
    title,
    children,
    footer,
  } = $props<{
    open?: boolean;
    title: string;
    children: import("svelte").Snippet;
    footer?: import("svelte").Snippet;
  }>();
</script>

<Dialog.Root bind:open>
  <Dialog.Portal>
    <Dialog.Overlay class="ui-dialog-overlay" />
    <Dialog.Content class="ui-dialog-content">
      <Dialog.Title class="ui-dialog-title">{title}</Dialog.Title>
      <div class="ui-dialog-body">{@render children()}</div>
      {#if footer}<div class="ui-dialog-footer">{@render footer()}</div>{/if}
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<style>
  :global(.ui-dialog-overlay) {
    position: fixed;
    inset: 0;
    z-index: 800;
    background: #07111d80;
    backdrop-filter: blur(4px);
  }
  :global(.ui-dialog-content) {
    position: fixed;
    z-index: 801;
    top: 50%;
    left: 50%;
    width: min(440px, calc(100vw - 32px));
    transform: translate(-50%, -50%);
    padding: 22px;
    color: var(--text);
    background: var(--overlay-surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: var(--shadow);
  }
  :global(.ui-dialog-title) {
    margin: 0 0 16px;
    font-size: 1.15rem;
  }
  :global(.ui-dialog-body) {
    color: var(--muted);
    line-height: 1.55;
  }
  :global(.ui-dialog-footer) {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 20px;
  }
</style>
