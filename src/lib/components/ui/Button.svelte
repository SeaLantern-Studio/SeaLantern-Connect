<script lang="ts">
  import { LoaderCircle } from "@lucide/svelte";
  let {
    variant = "solid",
    size = "md",
    loading = false,
    disabled = false,
    type = "button",
    class: className = "",
    onclick,
    title,
    children,
  } = $props<{
    variant?: "solid" | "outline" | "ghost" | "danger";
    size?: "sm" | "md" | "lg";
    loading?: boolean;
    disabled?: boolean;
    type?: "button" | "submit" | "reset";
    class?: string;
    onclick?: (event: MouseEvent) => void;
    title?: string;
    children: import("svelte").Snippet;
  }>();
</script>

<button
  class={`ui-button ui-button-${variant} ui-button-${size} ${className}`}
  {disabled}
  {title}
  aria-busy={loading}
  {type}
  {onclick}
>
  {#if loading}<LoaderCircle class="spin" size={16} />{/if}
  {@render children()}
</button>

<style>
  .ui-button {
    min-height: 38px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    padding: 0 18px;
    border: 1px solid transparent;
    border-radius: 8px;
    border-color: color-mix(in srgb, var(--border) 72%, transparent);
    background: color-mix(
      in srgb,
      var(--select-material-bg, var(--material-content-bg)) 50%,
      transparent
    );
    box-shadow:
      inset 0 1px rgba(255, 255, 255, 0.18),
      inset 0 -1px rgba(0, 0, 0, 0.08),
      0 4px 14px rgba(0, 0, 0, 0.06);
    backdrop-filter: blur(20px) saturate(1.18);
    -webkit-backdrop-filter: blur(20px) saturate(1.18);
    font-weight: 600;
    cursor: pointer;
    transform: translateY(0);
    transform-origin: center;
    will-change: transform;
    transition:
      transform 0.15s ease,
      background-color 0.15s ease,
      border-color 0.15s ease;
  }
  .ui-button:hover:not(:disabled) {
    transform: translateY(-1px);
  }
  .ui-button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .ui-button[aria-busy="true"] :global(svg:not(.spin)) {
    display: none;
  }
  .ui-button-solid {
    color: #fff;
    border-color: color-mix(in srgb, var(--primary) 58%, var(--border));
    background: color-mix(in srgb, var(--primary-solid, var(--primary)) 78%, transparent);
  }
  .ui-button-solid:hover:not(:disabled) {
    background: color-mix(
      in srgb,
      var(--primary-solid-hover, var(--primary-hover)) 88%,
      transparent
    );
  }
  .ui-button-outline {
    color: var(--primary);
    border-color: color-mix(in srgb, var(--primary) 72%, var(--border));
  }
  .ui-button-outline:hover:not(:disabled) {
    background: color-mix(in srgb, var(--primary) 18%, transparent);
  }
  .ui-button-ghost {
    color: var(--muted);
    background: transparent;
  }
  .ui-button-ghost:hover:not(:disabled) {
    color: var(--text);
    background: color-mix(in srgb, var(--text) 8%, transparent);
  }
  .ui-button-danger {
    color: var(--danger);
    border-color: color-mix(in srgb, var(--danger) 70%, transparent);
  }
  .ui-button-danger:hover:not(:disabled) {
    background: color-mix(in srgb, var(--danger) 10%, transparent);
  }
  .ui-button-sm {
    min-height: 30px;
    padding: 0 11px;
    font-size: 0.86rem;
  }
  .ui-button-lg {
    min-height: 44px;
    padding: 0 22px;
  }
</style>
