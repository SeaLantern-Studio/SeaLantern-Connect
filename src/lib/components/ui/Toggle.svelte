<script lang="ts">
  let {
    checked = $bindable(false),
    disabled = false,
    oncheckedchange,
    label = "Toggle",
  } = $props<{
    checked?: boolean;
    disabled?: boolean;
    oncheckedchange?: (checked: boolean) => void;
    label?: string;
  }>();
  function toggle(): void {
    checked = !checked;
    oncheckedchange?.(checked);
  }
</script>

<button
  class:checked
  class="ui-toggle cmz-toggle"
  type="button"
  role="switch"
  aria-label={label}
  aria-checked={checked}
  {disabled}
  onclick={toggle}
>
  <span></span>
</button>

<style>
  .ui-toggle {
    width: 38px;
    height: 22px;
    padding: 2px;
    border: 0;
    border-radius: 999px;
    background: var(--surface-strong);
    cursor: pointer;
    transition: background 0.2s ease;
  }
  .ui-toggle span {
    display: block;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--surface);
    box-shadow: 0 1px 4px #0003;
    transition: transform 0.2s ease;
  }
  .ui-toggle.checked {
    background: var(--primary);
  }
  .ui-toggle.checked span {
    transform: translateX(16px);
  }
</style>
