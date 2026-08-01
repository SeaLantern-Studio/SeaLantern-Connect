<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, watchEffect } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Cmz_Select, Cmz_Toggle, type SelectOption } from "cmzya-modern-ui";
import { t } from "../../i18n";
import type { PersonalizationUpdate, Preferences, SplashDurationMs } from "../../preferences";
import { getThemeOptions, type ColorThemeId } from "../../themes";
import { useUiStore } from "../../stores/ui";
import { toast } from "../../toast";
import { applyTypography, MAX_FONT_SIZE, MIN_FONT_SIZE } from "../../typography";

const props = defineProps<{ preferences: Preferences }>();
const uiStore = useUiStore();
const emit = defineEmits<{
  changeColorTheme: [colorTheme: ColorThemeId];
  changeTheme: [theme: PersonalizationUpdate["theme"]];
  saved: [update: PersonalizationUpdate];
}>();

const form = ref<PersonalizationUpdate>(pickPreferences(props.preferences));
const persisted = ref<PersonalizationUpdate>(pickPreferences(props.preferences));
const saving = ref(false);
const fontsLoading = ref(false);
const systemFonts = ref<string[]>([]);
const fontFamilyOptions = computed<SelectOption[]>(() => {
  const fonts = systemFonts.value.includes(form.value.fontFamily)
    ? systemFonts.value
    : [form.value.fontFamily, ...systemFonts.value].filter(Boolean);
  return [
    { label: t("personalization.systemFont"), value: "" },
    ...fonts.map((font) => ({ label: font, value: font })),
  ];
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
const hasChanges = computed(() => {
  const current = persisted.value;
  const update = form.value;
  return (
    update.theme !== current.theme ||
    update.colorTheme !== current.colorTheme ||
    update.fontSize !== current.fontSize ||
    update.fontFamily !== current.fontFamily ||
    update.splashDurationMs !== current.splashDurationMs ||
    update.locale !== current.locale ||
    update.rememberWindowState !== current.rememberWindowState ||
    update.closeAction !== current.closeAction
  );
});

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
  };
}

watch(
  () => props.preferences.theme,
  (theme) => {
    form.value.theme = theme;
    persisted.value.theme = theme;
  },
);
watch(
  () => props.preferences.colorTheme,
  (colorTheme) => {
    form.value.colorTheme = colorTheme;
    persisted.value.colorTheme = colorTheme;
  },
);
watch(
  () => props.preferences.fontSize,
  (fontSize) => {
    form.value.fontSize = fontSize;
    persisted.value.fontSize = fontSize;
  },
);
watch(
  () => props.preferences.fontFamily,
  (fontFamily) => {
    form.value.fontFamily = fontFamily;
    persisted.value.fontFamily = fontFamily;
  },
);
watch(
  () => props.preferences.splashDurationMs,
  (splashDurationMs) => {
    form.value.splashDurationMs = splashDurationMs;
    persisted.value.splashDurationMs = splashDurationMs;
  },
);
watch(
  () => props.preferences.locale,
  (locale) => {
    form.value.locale = locale;
    persisted.value.locale = locale;
  },
);
watch(
  () => props.preferences.rememberWindowState,
  (rememberWindowState) => {
    form.value.rememberWindowState = rememberWindowState;
    persisted.value.rememberWindowState = rememberWindowState;
  },
);
watch(
  () => props.preferences.closeAction,
  (closeAction) => {
    form.value.closeAction = closeAction;
    persisted.value.closeAction = closeAction;
  },
);

async function save() {
  if (!hasChanges.value || saving.value) return;
  saving.value = true;
  const update = { ...form.value };
  try {
    await invoke("set_personalization", { update });
    persisted.value = { ...update };
    emit("saved", update);
    toast.success(t("personalization.saved"));
  } catch (error) {
    console.error("Failed to save personalization", error);
    toast.error(t("common.saveFailed"));
  } finally {
    saving.value = false;
  }
}

const saveAction = () => save();

watch(
  () => [form.value.fontSize, form.value.fontFamily] as const,
  ([fontSize, fontFamily]) => applyTypography(fontSize, fontFamily),
  { immediate: true },
);

watchEffect(() => {
  uiStore.setSaveState("personalize", !saving.value && hasChanges.value, saving.value);
});

onMounted(() => {
  uiStore.registerSaveAction("personalize", saveAction);
  void loadSystemFonts();
});
onUnmounted(() => {
  uiStore.unregisterSaveAction("personalize", saveAction);
  applyTypography(persisted.value.fontSize, persisted.value.fontFamily);
});

async function loadSystemFonts() {
  fontsLoading.value = true;
  try {
    systemFonts.value = await invoke<string[]>("get_system_fonts");
  } catch (error) {
    console.error("Failed to load system fonts", error);
  } finally {
    fontsLoading.value = false;
  }
}

function updateRememberWindowState(value: boolean) {
  form.value.rememberWindowState = value;
}

function setTheme(value: string | number) {
  if (value !== "system" && value !== "light" && value !== "dark") return;
  form.value.theme = value;
  emit("changeTheme", value);
}

function setColorTheme(value: string | number) {
  if (!colorThemeOptions.value.some((option) => option.value === value)) return;
  form.value.colorTheme = value as ColorThemeId;
  emit("changeColorTheme", form.value.colorTheme);
}

function setFontFamily(value: string | number) {
  if (typeof value !== "string") return;
  form.value.fontFamily = value;
}

function setFontSize(event: Event) {
  form.value.fontSize = Number((event.target as HTMLInputElement).value);
}

function setCloseAction(value: string | number) {
  if (value !== "ask" && value !== "exit" && value !== "hide_to_tray") return;
  form.value.closeAction = value;
}

function setSplashDuration(value: string | number) {
  if (!splashDurationOptions.value.some((option) => option.value === value)) return;
  form.value.splashDurationMs = value as SplashDurationMs;
}
</script>

<template>
  <div class="workspace settings-workspace">
    <section class="settings-section">
      <div class="settings-section-heading">
        <div>
          <h2>{{ t("personalization.themeSection") }}</h2>
          <p>{{ t("personalization.appearanceHint") }}</p>
        </div>
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

      <div class="preference-row">
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
        <Cmz_Select
          class="settings-select font-family-select"
          :model-value="form.fontFamily"
          :options="fontFamilyOptions"
          :searchable="true"
          :loading="fontsLoading"
          :preview-font="true"
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
          :disabled="saving"
          @update:model-value="updateRememberWindowState"
        />
      </div>
    </section>
  </div>
</template>
