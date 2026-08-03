<script lang="ts">
  import AwesomeColorPicker from "svelte-awesome-color-picker";
  import "./ColorPicker.css";

  let {
    value,
    label,
    disabled = false,
    onvaluechange,
  } = $props<{
    value: string;
    label: string;
    disabled?: boolean;
    onvaluechange: (value: string) => void;
  }>();

  const swatches = [
    "#ffffff",
    "#f1f5f9",
    "#94a3b8",
    "#475569",
    "#0f172a",
    "#0ea5e9",
    "#22c55e",
    "#f59e0b",
    "#ef4444",
    "#a855f7",
  ];

  function updateColor(hex: string | null): void {
    if (!hex || hex.toUpperCase() === value.toUpperCase()) return;
    onvaluechange(hex.toUpperCase());
  }
</script>

{#if disabled}
  <span class="ui-color-picker-swatch" style={`background: ${value}`} aria-label={label}></span>
{:else}
  <div class="ui-color-picker">
    <AwesomeColorPicker
      hex={value}
      {label}
      isAlpha={false}
      isTextInput
      textInputModes={["hex"]}
      {swatches}
      onInput={({ hex }) => updateColor(hex)}
    />
  </div>
{/if}
