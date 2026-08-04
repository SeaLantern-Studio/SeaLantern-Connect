<script lang="ts">
  import { ChevronDown, Search } from "@lucide/svelte";
  import { Select } from "bits-ui";
  export interface Option {
    label: string;
    value: string | number;
    fontFamily?: string;
  }
  interface NormalizedOption {
    label: string;
    value: string;
    fontFamily?: string;
  }
  let {
    value = $bindable<string | number>(),
    options,
    placeholder = "Select",
    disabled = false,
    searchable = false,
    searchPlaceholder = "Search",
    emptyLabel = "No results",
    class: className = "",
    onValueChange,
  } = $props<{
    value?: string | number;
    options: Option[];
    placeholder?: string;
    disabled?: boolean;
    searchable?: boolean;
    searchPlaceholder?: string;
    emptyLabel?: string;
    class?: string;
    onValueChange?: (value: string) => void;
  }>();
  const normalized: NormalizedOption[] = $derived(
    options.map((item: Option) => ({ ...item, value: String(item.value) })),
  );
  let selected = $state<string | undefined>(value == null ? undefined : String(value));
  let open = $state(false);
  let searchTerm = $state("");
  let searchInput = $state<HTMLInputElement | null>(null);
  let focusFrameFirst = 0;
  let focusFrameSecond = 0;
  const selectedOption = $derived(
    normalized.find((option: NormalizedOption) => option.value === selected),
  );
  const hasFontPreview = $derived(normalized.some((option) => option.fontFamily));
  const visibleOptions = $derived(
    searchable && searchTerm.trim()
      ? normalized.filter((option) =>
          option.label.toLocaleLowerCase().includes(searchTerm.trim().toLocaleLowerCase()),
        )
      : normalized,
  );
  $effect(() => {
    const next = value == null ? undefined : String(value);
    if (selected !== next) selected = next;
  });
  $effect(() => {
    if (!open || !searchable) return;
    focusFrameFirst = requestAnimationFrame(() => {
      focusFrameSecond = requestAnimationFrame(() => {
        if (open && searchable) searchInput?.focus({ preventScroll: true });
      });
    });
    return () => {
      cancelAnimationFrame(focusFrameFirst);
      cancelAnimationFrame(focusFrameSecond);
    };
  });
  function update(next: string | undefined): void {
    if (next == null) return;
    selected = next;
    value = next;
    onValueChange?.(next);
  }
  function fontFamilyStyle(fontFamily: string | undefined): string | undefined {
    return fontFamily ? `font-family: ${JSON.stringify(fontFamily)};` : undefined;
  }
  function focusOnMount(node: HTMLInputElement): void {
    node.focus({ preventScroll: true });
  }
  function handleTriggerKeydown(event: KeyboardEvent): void {
    if (
      !open ||
      !searchable ||
      event.defaultPrevented ||
      event.isComposing ||
      event.ctrlKey ||
      event.metaKey ||
      event.altKey ||
      event.key.length !== 1
    ) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    searchInput?.focus({ preventScroll: true });
    searchTerm += event.key;
  }
  function handleOpenChange(nextOpen: boolean): void {
    if (!nextOpen) {
      searchTerm = "";
      cancelAnimationFrame(focusFrameFirst);
      cancelAnimationFrame(focusFrameSecond);
    }
  }
</script>

<Select.Root
  type="single"
  bind:open
  {disabled}
  value={selected}
  items={visibleOptions.map((option) => ({ value: option.value, label: option.label }))}
  onValueChange={update}
  onOpenChange={handleOpenChange}
>
  <Select.Trigger
    class={`ui-select ${className}`}
    style={fontFamilyStyle(selectedOption?.fontFamily)}
    onkeydown={handleTriggerKeydown}
  >
    <span class:ui-select-placeholder={!selectedOption} class="ui-select-value">
      {selectedOption?.label ?? placeholder}
    </span>
    <ChevronDown class="ui-select-chevron" size={16} strokeWidth={2} aria-hidden="true" />
  </Select.Trigger>
  <Select.Content
    align="end"
    class={`ui-select-content ${hasFontPreview ? "ui-select-font-content" : ""}`}
  >
    {#if searchable}<div class="ui-select-search-wrap">
        <Search class="ui-select-search-icon" size={15} strokeWidth={2} aria-hidden="true" />
        <input
          bind:this={searchInput}
          class="ui-select-search"
          type="search"
          use:focusOnMount
          value={searchTerm}
          placeholder={searchPlaceholder}
          aria-label={searchPlaceholder}
          oninput={(event) => (searchTerm = event.currentTarget.value)}
          onkeydown={(event) => {
            if (event.key !== "Escape") event.stopPropagation();
          }}
        />
      </div>{/if}
    {#if visibleOptions.length === 0}<div class="ui-select-empty">{emptyLabel}</div>{/if}
    {#each visibleOptions as option (option.value)}
      <Select.Item value={option.value} label={option.label} class="ui-select-item"
        ><span class="ui-select-item-label" style={fontFamilyStyle(option.fontFamily)}
          >{option.label}</span
        ></Select.Item
      >
    {/each}
  </Select.Content>
</Select.Root>

<style>
  :global(.ui-select) {
    width: 100%;
    height: 38px;
    min-height: 38px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    color: var(--text);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    font-size: 0.9286rem;
    line-height: 1.4;
    text-align: left;
  }
  :global(.ui-select:hover) {
    border-color: color-mix(in srgb, var(--primary) 60%, var(--border));
  }
  :global(.ui-select-chevron) {
    flex: none;
    color: var(--muted);
    transition: transform 0.15s ease;
  }
  :global(.ui-select[data-state="open"] .ui-select-chevron) {
    transform: rotate(180deg);
  }
  :global(.ui-select-value) {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  :global(.ui-select-placeholder) {
    color: var(--muted);
  }
  :global(.ui-select-content) {
    z-index: 100;
    min-width: var(--bits-floating-anchor-width, 0px);
    max-height: 280px;
    overflow: auto;
    padding: 5px;
    color: var(--text);
    background: var(--overlay-surface);
    border: 1px solid color-mix(in srgb, var(--border) 82%, var(--text) 10%);
    border-radius: var(--cmz-radius-md);
    box-shadow:
      0 10px 28px rgba(0, 0, 0, 0.16),
      0 2px 8px rgba(0, 0, 0, 0.08);
    font-size: 0.9286rem;
    transform-origin: top right;
    will-change: opacity, transform;
  }
  :global(.ui-select-content[data-state="open"]) {
    animation: ui-select-content-in 0.16s cubic-bezier(0.22, 1, 0.36, 1);
  }
  :global(.ui-select-content.ui-select-font-content) {
    width: min(280px, calc(100vw - 32px));
    min-width: min(280px, calc(100vw - 32px));
    max-width: calc(100vw - 32px);
  }
  :global(.ui-select-search-wrap) {
    min-height: 34px;
    display: flex;
    align-items: center;
    gap: 7px;
    margin: 2px 2px 6px;
    padding: 0 9px;
    color: var(--muted);
    background: color-mix(in srgb, var(--surface-soft) 82%, transparent);
    border: 1px solid transparent;
    border-radius: 6px;
    transition:
      background-color 0.15s ease,
      border-color 0.15s ease,
      box-shadow 0.15s ease;
  }
  :global(.ui-select-search-wrap:focus-within) {
    background: var(--surface);
    border-color: color-mix(in srgb, var(--primary) 68%, var(--border));
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 13%, transparent);
  }
  :global(.ui-select-search-icon) {
    flex: none;
  }
  :global(.ui-select-search) {
    width: 100%;
    min-width: 0;
    min-height: 32px;
    padding: 0;
    color: var(--text);
    background: transparent;
    border: 0;
    outline: 0;
    font: inherit;
  }
  :global(.ui-select-empty) {
    padding: 12px 10px;
    color: var(--muted);
    font-size: 0.8571rem;
    text-align: center;
  }
  :global(.ui-select-item) {
    min-height: 36px;
    display: flex;
    align-items: center;
    padding: 8px 10px;
    border-radius: 6px;
    cursor: pointer;
    outline: 0;
  }
  :global(.ui-select-item:hover),
  :global(.ui-select-item[data-highlighted]) {
    background: color-mix(in srgb, var(--primary) 11%, transparent);
  }
  :global(.ui-select-font-content .ui-select-item) {
    overflow: hidden;
    white-space: nowrap;
  }
  :global(.ui-select-font-content .ui-select-item-label) {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  @keyframes ui-select-content-in {
    from {
      opacity: 0;
      transform: translateY(-3px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
