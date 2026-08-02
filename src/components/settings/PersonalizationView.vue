<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { Cmz_Select, type SelectOption } from "cmzya-modern-ui";
import { getSystemFonts, supportsLiquidGlass } from "@api";
import { t } from "../../i18n";
import type {
  CustomTheme,
  PersonalizationUpdate,
  Preferences,
  ThemeColors,
  ColorThemeId,
  WindowMaterial,
} from "../../models/preferences";
import { getThemeOptions } from "../../themes";
import { applyTypography, MAX_FONT_SIZE, MIN_FONT_SIZE } from "../../ui/typography";
import FontSelect from "./FontSelect.vue";

const props = defineProps<{ preferences: Preferences }>();
const emit = defineEmits<{
  change: [update: PersonalizationUpdate];
}>();

const form = ref<PersonalizationUpdate>(pickPreferences(props.preferences));
const fontsLoading = ref(false);
const systemFonts = ref<string[]>([]);
const liquidGlassSupported = ref(false);
const customPlan = ref<keyof CustomTheme>(
  props.preferences.theme === "dark" ||
    (props.preferences.theme === "system" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches)
    ? "dark"
    : "light",
);
const customColorFields: Array<{ key: keyof ThemeColors; label: string }> = [
  { key: "bg", label: "background" },
  { key: "bgSecondary", label: "surface" },
  { key: "bgTertiary", label: "surfaceStrong" },
  { key: "primary", label: "primary" },
  { key: "primarySolid", label: "primarySolid" },
  { key: "primarySolidHover", label: "primarySolidHover" },
  { key: "secondary", label: "secondary" },
  { key: "textPrimary", label: "textPrimary" },
  { key: "textSecondary", label: "textSecondary" },
  { key: "border", label: "border" },
];
const fontFamilyOptions = computed<SelectOption[]>(() => {
  return fontOptions(form.value.fontFamily, t("personalization.systemFont"));
});
const colorThemeOptions = computed<SelectOption[]>(() =>
  getThemeOptions().map((option) => ({
    label: t(`personalization.colorThemes.${option.value}`),
    value: option.value,
  })),
);
const themeOptions = computed<SelectOption[]>(() => [
  { label: t("personalization.followSystem"), value: "system" },
  { label: t("personalization.light"), value: "light" },
  { label: t("personalization.dark"), value: "dark" },
]);
const customPlanOptions = computed<SelectOption[]>(() => [
  { label: t("personalization.customThemeLight"), value: "light" },
  { label: t("personalization.customThemeDark"), value: "dark" },
]);
const windowMaterialOptions = computed<SelectOption[]>(() => {
  if (/Macintosh|Mac OS X/i.test(navigator.userAgent)) {
    const options: SelectOption[] = [
      { label: t("personalization.windowMaterials.solid"), value: "solid" },
      { label: t("personalization.windowMaterials.vibrancy"), value: "vibrancy" },
    ];
    if (liquidGlassSupported.value) {
      options.push({
        label: t("personalization.windowMaterials.liquidGlass"),
        value: "liquid_glass",
      });
    }
    return options;
  }
  if (/Windows/i.test(navigator.userAgent)) {
    return [
      { label: t("personalization.windowMaterials.solid"), value: "solid" },
      { label: t("personalization.windowMaterials.acrylic"), value: "acrylic" },
      { label: t("personalization.windowMaterials.mica"), value: "mica" },
    ];
  }
  return [{ label: t("personalization.windowMaterials.solid"), value: "solid" }];
});
const fontSizeProgress = computed(
  () => `${((form.value.fontSize - MIN_FONT_SIZE) / (MAX_FONT_SIZE - MIN_FONT_SIZE)) * 100}%`,
);
const usesSolidWindowMaterial = computed(() => form.value.windowMaterial === "solid");
const usesCustomTheme = computed(
  () => usesSolidWindowMaterial.value && form.value.colorTheme === "custom",
);
const customColors = computed(() => form.value.customTheme[customPlan.value]);

function cloneCustomTheme(theme: CustomTheme): CustomTheme {
  return {
    light: { ...theme.light },
    dark: { ...theme.dark },
  };
}

function pickPreferences(preferences: Preferences): PersonalizationUpdate {
  return {
    theme: preferences.theme,
    colorTheme: preferences.colorTheme,
    customTheme: cloneCustomTheme(preferences.customTheme),
    fontSize: preferences.fontSize,
    fontFamily: preferences.fontFamily,
    windowMaterial: preferences.windowMaterial,
  };
}

function fontOptions(selected: string, defaultLabel: string): SelectOption[] {
  const fonts = systemFonts.value.includes(selected)
    ? systemFonts.value
    : [selected, ...systemFonts.value].filter(Boolean);
  return [
    { label: defaultLabel, value: "" },
    ...fonts.map((font) => ({ label: font, value: font })),
  ];
}

watch(
  () => props.preferences.theme,
  (theme) => (form.value.theme = theme),
);
watch(
  () => props.preferences.colorTheme,
  (colorTheme) => (form.value.colorTheme = colorTheme),
);
watch(
  () => props.preferences.customTheme,
  (customTheme) => (form.value.customTheme = cloneCustomTheme(customTheme)),
  { deep: true },
);
watch(
  () => props.preferences.fontSize,
  (fontSize) => (form.value.fontSize = fontSize),
);
watch(
  () => props.preferences.fontFamily,
  (fontFamily) => (form.value.fontFamily = fontFamily),
);
watch(
  () => props.preferences.windowMaterial,
  (windowMaterial) => (form.value.windowMaterial = windowMaterial),
);

watch(
  () => [form.value.fontSize, form.value.fontFamily] as const,
  ([fontSize, fontFamily]) => applyTypography(fontSize, fontFamily),
  { immediate: true },
);

onMounted(() => {
  void loadSystemFonts();
  void loadMaterialCapabilities();
});

async function loadMaterialCapabilities() {
  try {
    liquidGlassSupported.value = await supportsLiquidGlass();
  } catch (error) {
    console.error("Failed to load native material capabilities", error);
  }
}

async function loadSystemFonts() {
  fontsLoading.value = true;
  try {
    systemFonts.value = await getSystemFonts();
  } catch (error) {
    console.error("Failed to load system fonts", error);
  } finally {
    fontsLoading.value = false;
  }
}

function setWindowMaterial(value: string | number) {
  if (!windowMaterialOptions.value.some((option) => option.value === value)) return;
  form.value.windowMaterial = value as WindowMaterial;
  persist();
}

function setTheme(value: string | number) {
  if (value !== "system" && value !== "light" && value !== "dark") return;
  form.value.theme = value;
  persist();
}

function setColorTheme(value: string | number) {
  if (!colorThemeOptions.value.some((option) => option.value === value)) return;
  form.value.colorTheme = value as ColorThemeId;
  persist();
}

function setCustomPlan(value: string | number) {
  if (value === "light" || value === "dark") customPlan.value = value;
}

function setCustomColor(field: keyof ThemeColors, event: Event) {
  form.value.customTheme[customPlan.value][field] = (event.target as HTMLInputElement).value;
  persist();
}

function setFontFamily(value: string | number) {
  if (typeof value !== "string") return;
  form.value.fontFamily = value;
  persist();
}

function setFontSize(event: Event) {
  form.value.fontSize = Number((event.target as HTMLInputElement).value);
  persist();
}

function persist() {
  emit("change", { ...form.value, customTheme: cloneCustomTheme(form.value.customTheme) });
}
</script>

<template>
  <div class="workspace settings-workspace">
    <section class="settings-section">
      <div class="settings-section-heading">
        <div>
          <h2>{{ t("personalization.themeSection") }}</h2>
          <p>
            {{
              t(
                usesSolidWindowMaterial
                  ? "personalization.appearanceHint"
                  : "personalization.nativeMaterialHint",
              )
            }}
          </p>
        </div>
      </div>

      <div class="preference-row">
        <span>{{ t("personalization.windowMaterial") }}</span>
        <Cmz_Select
          class="settings-select"
          :model-value="form.windowMaterial"
          :options="windowMaterialOptions"
          @update:model-value="setWindowMaterial"
        />
      </div>

      <div class="preference-row">
        <span>{{ t("personalization.themeMode") }}</span>
        <Cmz_Select
          class="settings-select"
          :model-value="form.theme"
          :options="themeOptions"
          @update:model-value="setTheme"
        />
      </div>

      <div v-if="usesSolidWindowMaterial" class="preference-row">
        <span>{{ t("personalization.colorTheme") }}</span>
        <Cmz_Select
          class="settings-select"
          :model-value="form.colorTheme"
          :options="colorThemeOptions"
          @update:model-value="setColorTheme"
        />
      </div>

      <div class="preference-row">
        <span>{{ t("personalization.fontSize") }}</span>
        <div class="font-size-control">
          <input
            id="font-size-slider"
            class="settings-slider"
            type="range"
            :min="MIN_FONT_SIZE"
            :max="MAX_FONT_SIZE"
            step="1"
            :value="form.fontSize"
            :style="{ '--slider-progress': fontSizeProgress }"
            :aria-label="t('personalization.fontSize')"
            :aria-valuetext="`${form.fontSize}px`"
            @input="setFontSize"
          />
          <output for="font-size-slider">{{ form.fontSize }}px</output>
        </div>
      </div>

      <div class="preference-row">
        <span>{{ t("personalization.fontFamily") }}</span>
        <FontSelect
          class="settings-select font-family-select"
          :model-value="form.fontFamily"
          :options="fontFamilyOptions"
          :searchable="true"
          :loading="fontsLoading"
          :placeholder="t('personalization.searchFont')"
          @update:model-value="setFontFamily"
        />
      </div>
    </section>

    <section
      class="settings-section custom-theme-section"
      :class="{ disabled: !usesCustomTheme }"
      :aria-disabled="!usesCustomTheme"
    >
      <div class="settings-section-heading">
        <div>
          <h2>{{ t("personalization.customTheme") }}</h2>
          <p>{{ t("personalization.customThemeHint") }}</p>
        </div>
      </div>

      <div class="preference-row">
        <span>{{ t("personalization.customThemePalette") }}</span>
        <Cmz_Select
          class="settings-select"
          :model-value="customPlan"
          :options="customPlanOptions"
          :disabled="!usesCustomTheme"
          @update:model-value="setCustomPlan"
        />
      </div>

      <div class="custom-theme-grid">
        <label v-for="field in customColorFields" :key="field.key" class="custom-color-field">
          <span>{{ t(`personalization.customColors.${field.label}`) }}</span>
          <span class="custom-color-control">
            <input
              type="color"
              :value="customColors[field.key]"
              :disabled="!usesCustomTheme"
              :aria-label="t(`personalization.customColors.${field.label}`)"
              @input="setCustomColor(field.key, $event)"
            />
            <code>{{ customColors[field.key].toUpperCase() }}</code>
          </span>
        </label>
      </div>
    </section>
  </div>
</template>
