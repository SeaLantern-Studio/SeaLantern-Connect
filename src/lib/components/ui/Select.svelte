<script lang="ts">
  import { ChevronDown, Search } from "@lucide/svelte";
  import { Select } from "bits-ui";
  export interface Option {
    label: string;
    value: string | number;
    fontFamily?: string;
  }
  export interface PointerOrigin {
    x: number;
    y: number;
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
    portal = false,
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
    portal?: boolean;
    onValueChange?: (value: string, origin?: PointerOrigin) => void;
  }>();
  const normalized: NormalizedOption[] = $derived(
    options.map((item: Option) => ({ ...item, value: String(item.value) })),
  );
  function matchesFuzzy(label: string, query: string): boolean {
    let queryIndex = 0;
    const normalizedLabel = label.toLocaleLowerCase();
    const normalizedQuery = query.toLocaleLowerCase();
    for (const character of normalizedLabel) {
      if (character === normalizedQuery[queryIndex]) queryIndex += 1;
      if (queryIndex === normalizedQuery.length) return true;
    }
    return normalizedQuery.length === 0;
  }
  let selected = $state<string | undefined>(value == null ? undefined : String(value));
  let open = $state(false);
  let optionsReady = $state(!searchable);
  let searchTerm = $state("");
  let searchInput = $state<HTMLInputElement | null>(null);
  let lastPointerOrigin = $state<PointerOrigin | undefined>();
  const initialOptionLimit = 100;
  const selectedOption = $derived(
    normalized.find((option: NormalizedOption) => option.value === selected),
  );
  const visibleOptions = $derived(
    searchable && searchTerm.trim()
      ? normalized.filter((option) => matchesFuzzy(option.label, searchTerm.trim()))
      : normalized.slice(0, initialOptionLimit),
  );
  const renderedOptions = $derived(optionsReady ? visibleOptions : []);
  $effect(() => {
    const next = value == null ? undefined : String(value);
    if (selected !== next) selected = next;
  });
  function update(next: string | undefined): void {
    if (next == null) return;
    selected = next;
    value = next;
    const origin = lastPointerOrigin;
    lastPointerOrigin = undefined;
    onValueChange?.(next, origin);
  }
  function fontFamilyStyle(fontFamily: string | undefined): string | undefined {
    return fontFamily ? `font-family: ${JSON.stringify(fontFamily)};` : undefined;
  }
  function marqueeIfOverflow(node: HTMLElement): { destroy: () => void } {
    const update = (): void => {
      const shift = Math.min(0, node.clientWidth - node.scrollWidth);
      node.classList.toggle("ui-select-marquee", shift < -1);
      node.style.setProperty("--ui-select-marquee-shift", `${shift}px`);
    };
    const observer = new ResizeObserver(update);
    observer.observe(node);
    requestAnimationFrame(update);
    return { destroy: () => observer.disconnect() };
  }
  function focusSearchInput(): void {
    if (!open || !searchable) return;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        if (open && searchable) searchInput?.focus({ preventScroll: true });
      });
    });
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
    if (nextOpen) {
      optionsReady = false;
      requestAnimationFrame(() => {
        if (open) optionsReady = true;
      });
      window.setTimeout(focusSearchInput, 32);
    } else {
      searchTerm = "";
      optionsReady = !searchable;
    }
  }
</script>

{#snippet selectContent()}
  <Select.Content
    align="end"
    class="ui-select-content"
  >
    {#if searchable}<div class="ui-select-search-wrap">
        <Search class="ui-select-search-icon" size={15} strokeWidth={2} aria-hidden="true" />
        <input
          bind:this={searchInput}
          class="ui-select-search"
          type="search"
          value={searchTerm}
          placeholder={searchPlaceholder}
          aria-label={searchPlaceholder}
          oninput={(event) => (searchTerm = event.currentTarget.value)}
          onblur={(event) => {
            const nextTarget = event.relatedTarget as HTMLElement | null;
            if (open && searchable && !nextTarget?.closest(".ui-select-item")) {
              window.setTimeout(focusSearchInput, 0);
            }
          }}
          onkeydown={(event) => {
            if (event.key !== "Escape") event.stopPropagation();
          }}
        />
      </div>{/if}
    {#if renderedOptions.length === 0 && searchTerm}<div class="ui-select-empty">
        {emptyLabel}
      </div>{/if}
    {#each renderedOptions as option (option.value)}
      <Select.Item
        value={option.value}
        label={option.label}
        class="ui-select-item"
        onpointerdown={(event) => (lastPointerOrigin = { x: event.clientX, y: event.clientY })}
        ><span
          class="ui-select-item-label"
          style={fontFamilyStyle(option.fontFamily)}
          use:marqueeIfOverflow
          ><span>{option.label}</span></span
        ></Select.Item
      >
    {/each}
  </Select.Content>
{/snippet}

<Select.Root
  type="single"
  bind:open
  {disabled}
  value={selected}
  items={renderedOptions.map((option) => ({ value: option.value, label: option.label }))}
  onValueChange={update}
  onOpenChange={handleOpenChange}
>
  <Select.Trigger
    class={`ui-select ${className}`}
    style={fontFamilyStyle(selectedOption?.fontFamily)}
    onkeydown={handleTriggerKeydown}
  >
    <span
      class:ui-select-placeholder={!selectedOption}
      class="ui-select-value"
      use:marqueeIfOverflow
    >
      <span>{selectedOption?.label ?? placeholder}</span>
    </span>
    <ChevronDown class="ui-select-chevron" size={16} strokeWidth={2} aria-hidden="true" />
  </Select.Trigger>
  {#if portal}
    <Select.Portal>{@render selectContent()}</Select.Portal>
  {:else}
    {@render selectContent()}
  {/if}
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
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  :global(.ui-select-placeholder) {
    color: var(--muted);
  }
  :global(.ui-select-content) {
    z-index: 900;
    margin-top: 6px;
    width: var(--bits-floating-anchor-width, 0px);
    min-width: 0;
    max-width: calc(100vw - 32px);
    max-height: 280px;
    overflow: auto;
    padding: 5px;
    color: var(--text);
    background: var(--overlay-surface);
    border: 1px solid color-mix(in srgb, var(--border) 82%, var(--text) 10%);
    border-radius: var(--sl-radius-md);
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
  :global(.ui-select-item) {
    overflow: hidden;
    white-space: nowrap;
  }
  :global(.ui-select-item-label) {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  :global(.ui-select-value > span),
  :global(.ui-select-item-label > span) {
    display: inline-block;
    min-width: max-content;
    white-space: nowrap;
  }
  :global(.ui-select:hover .ui-select-marquee > span),
  :global(.ui-select-item:hover .ui-select-marquee > span),
  :global(.ui-select-item[data-highlighted] .ui-select-marquee > span) {
    animation: ui-select-marquee 1.6s ease-in-out 0.2s infinite alternate;
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
  @keyframes ui-select-marquee {
    to {
      transform: translateX(var(--ui-select-marquee-shift));
    }
  }
  @media (prefers-reduced-motion: reduce) {
    :global(.ui-select:hover .ui-select-marquee > span),
    :global(.ui-select-item:hover .ui-select-marquee > span),
    :global(.ui-select-item[data-highlighted] .ui-select-marquee > span) {
      animation: none;
    }
  }
</style>
