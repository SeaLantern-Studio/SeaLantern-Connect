<script setup lang="ts">
import { Cmz_Select, type SelectOption } from "cmzya-modern-ui";
import {
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
  type ComponentPublicInstance,
} from "vue";

const props = withDefaults(
  defineProps<{
    modelValue?: string | number;
    options: SelectOption[];
    label?: string;
    placeholder?: string;
    disabled?: boolean;
    searchable?: boolean;
    loading?: boolean;
    maxHeight?: string;
    collapsed?: boolean;
    dropdownAlign?: "left" | "right";
    dropdownWidth?: string;
  }>(),
  {
    modelValue: undefined,
    label: undefined,
    placeholder: "Select",
    disabled: false,
    searchable: false,
    loading: false,
    maxHeight: "280px",
    collapsed: false,
    dropdownAlign: "left",
    dropdownWidth: "200px",
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string | number];
}>();

const selectComponent = ref<ComponentPublicInstance | null>(null);
let active = false;
let dropdownObserver: MutationObserver | null = null;

function fontFamily(value: string | number): string {
  if (typeof value !== "string" || !value.trim()) {
    return 'Inter, "PingFang SC", "Microsoft YaHei", sans-serif';
  }
  const family = value.trim().replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  return `"${family}", sans-serif`;
}

function selectRoot(): HTMLElement | null {
  return (selectComponent.value?.$el as HTMLElement | undefined) ?? null;
}

function openDropdown(): HTMLElement | null {
  const dropdowns = document.querySelectorAll<HTMLElement>(".cmz-select-dropdown");
  return dropdowns.item(dropdowns.length - 1);
}

function applySelectedPreview() {
  const value = selectRoot()?.querySelector<HTMLElement>(".cmz-select-value");
  const selected = props.options.find((option) => option.value === props.modelValue);
  if (value && selected) value.style.fontFamily = fontFamily(selected.value);
}

function applyOptionPreviews() {
  if (!active) return;
  const optionsByLabel = new Map(props.options.map((option) => [option.label, option]));
  openDropdown()
    ?.querySelectorAll<HTMLElement>(".cmz-select-option")
    .forEach((element) => {
      const label = element.querySelector<HTMLElement>(".option-label");
      const option = label ? optionsByLabel.get(label.textContent?.trim() ?? "") : undefined;
      if (label && option) label.style.fontFamily = fontFamily(option.value);
    });
}

function activate() {
  active = true;
  void nextTick(() => {
    applySelectedPreview();
    applyOptionPreviews();
  });
}

function update(value: string | number) {
  active = false;
  emit("update:modelValue", value);
}

function handleDocumentClick(event: MouseEvent) {
  const target = event.target as Node;
  if (selectRoot()?.contains(target) || openDropdown()?.contains(target)) return;
  active = false;
}

watch(
  () => [props.modelValue, props.options] as const,
  () => void nextTick(applySelectedPreview),
  { immediate: true },
);

onMounted(() => {
  dropdownObserver = new MutationObserver(applyOptionPreviews);
  dropdownObserver.observe(document.body, { childList: true, subtree: true });
  document.addEventListener("click", handleDocumentClick);
  void nextTick(applySelectedPreview);
});

onBeforeUnmount(() => {
  dropdownObserver?.disconnect();
  document.removeEventListener("click", handleDocumentClick);
});
</script>

<template>
  <Cmz_Select
    ref="selectComponent"
    :model-value="modelValue"
    :options="options"
    :label="label"
    :placeholder="placeholder"
    :disabled="disabled"
    :searchable="searchable"
    :loading="loading"
    :max-height="maxHeight"
    :collapsed="collapsed"
    :dropdown-align="dropdownAlign"
    :dropdown-width="dropdownWidth"
    @click.capture="activate"
    @keydown.capture="activate"
    @update:model-value="update"
  />
</template>
