<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { Cmz_Select, Cmz_Toggle, type SelectOption } from "cmzya-modern-ui";
import { getSystemFonts, supportsLiquidGlass } from "@api";
import { t } from "../../i18n";
import type {
  PersonalizationUpdate,
  Preferences,
  SplashDurationMs,
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
const closeActionOptions = computed<SelectOption[]>(() => [
  { label: t("personalization.askOnClose"), value: "ask" },
  { label: t("personalization.exitApplication"), value: "exit" },
  { label: t("personalization.hideToTray"), value: "hide_to_tray" },
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
const splashDurationOptions = computed<SelectOption[]>(() => [
  { label: t("personalization.disabled"), value: 0 },
  ...[500, 1000, 1500, 2000].map((durationMs) => ({
    label: t("personalization.seconds", { value: durationMs / 1000 }),
    value: durationMs,
  })),
]);
const fontSizeProgress = computed(
  () => `${((form.value.fontSize - MIN_FONT_SIZE) / (MAX_FONT_SIZE - MIN_FONT_SIZE)) * 100}%`,
);
const usesSolidWindowMaterial = computed(() => form.value.windowMaterial === "solid");
function pickPreferences(preferences: Preferences): PersonalizationUpdate {
  return {
    theme: preferences.theme,
    colorTheme: preferences.colorTheme,
    fontSize: preferences.fontSize,
    fontFamily: preferences.fontFamily,
    splashDurationMs: preferences.splashDurationMs,
    locale: preferences.locale,
    rememberWindowState: preferences.rememberWindowState,
    closeAction: preferences.closeAction,
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
  () => props.preferences.fontSize,
  (fontSize) => (form.value.fontSize = fontSize),
);
watch(
  () => props.preferences.fontFamily,
  (fontFamily) => (form.value.fontFamily = fontFamily),
);
watch(
  () => props.preferences.splashDurationMs,
  (splashDurationMs) => (form.value.splashDurationMs = splashDurationMs),
);
watch(
  () => props.preferences.locale,
  (locale) => (form.value.locale = locale),
);
watch(
  () => props.preferences.rememberWindowState,
  (rememberWindowState) => (form.value.rememberWindowState = rememberWindowState),
);
watch(
  () => props.preferences.closeAction,
  (closeAction) => (form.value.closeAction = closeAction),
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

function updateRememberWindowState(value: boolean) {
  form.value.rememberWindowState = value;
  persist();
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

function setFontFamily(value: string | number) {
  if (typeof value !== "string") return;
  form.value.fontFamily = value;
  persist();
}

function setFontSize(event: Event) {
  form.value.fontSize = Number((event.target as HTMLInputElement).value);
  persist();
}

function setCloseAction(value: string | number) {
  if (value !== "ask" && value !== "exit" && value !== "hide_to_tray") return;
  form.value.closeAction = value;
  persist();
}

function setSplashDuration(value: string | number) {
  if (!splashDurationOptions.value.some((option) => option.value === value)) return;
  form.value.splashDurationMs = value as SplashDurationMs;
  persist();
}

function persist() {
  emit("change", { ...form.value });
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
            class="settings-slider"
            type="range"
            :min="MIN_FONT_SIZE"
            :max="MAX_FONT_SIZE"
            step="1"
            :value="form.fontSize"
            :style="{ '--slider-progress': fontSizeProgress }"
            @input="setFontSize"
          />
          <output>{{ form.fontSize }}px</output>
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

    <section class="settings-section">
      <div class="settings-section-heading">
        <div>
          <h2>{{ t("personalization.startup") }}</h2>
          <p>{{ t("personalization.startupHint") }}</p>
        </div>
      </div>

      <div class="preference-row">
        <span>{{ t("personalization.splashDuration") }}</span>
        <Cmz_Select
          class="settings-select"
          :model-value="form.splashDurationMs"
          :options="splashDurationOptions"
          @update:model-value="setSplashDuration"
        />
      </div>
    </section>

    <section class="settings-section">
      <div class="settings-section-heading">
        <div>
          <h2>{{ t("personalization.windowBehavior") }}</h2>
          <p>{{ t("personalization.windowBehaviorHint") }}</p>
        </div>
      </div>

      <div class="preference-row">
        <span>{{ t("personalization.closeAction") }}</span>
        <Cmz_Select
          class="settings-select"
          :model-value="form.closeAction"
          :options="closeActionOptions"
          @update:model-value="setCloseAction"
        />
      </div>

      <div class="preference-row switch-row">
        <span>{{ t("personalization.rememberWindowState") }}</span>
        <Cmz_Toggle
          :model-value="form.rememberWindowState"
          variant="switch"
          size="sm"
          @update:model-value="updateRememberWindowState"
        />
      </div>
    </section>
  </div>
</template>
