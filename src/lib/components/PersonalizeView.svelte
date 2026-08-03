<script lang="ts">
  import { onMount } from "svelte";
  import { getSystemFonts, supportsLiquidGlass } from "@api/settings";
  import { t } from "@i18n";
  import type { ColorThemeId } from "@models/preferences";
  import type { Preferences, ThemeColors } from "@models/preferences";
  import { getThemeOptions } from "@themes";
  import { MAX_FONT_SIZE, MIN_FONT_SIZE } from "@themes/typography";
  import ColorPicker from "./ui/ColorPicker.svelte";
  import Select, { type Option } from "./ui/Select.svelte";

  let { value, onupdate } = $props<{
    value: Preferences;
    onupdate: (value: Partial<Preferences>) => void;
  }>();
  let fonts = $state<string[]>([]);
  let liquidGlassSupported = $state(false);
  let platform = $state<"macos" | "windows" | "other">("other");
  let customPlan = $state<"light" | "dark">("light");
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
  const themeOptions = $derived([
    { label: t("personalization.followSystem"), value: "system" },
    { label: t("personalization.light"), value: "light" },
    { label: t("personalization.dark"), value: "dark" },
  ]);
  const materialOptions = $derived<Option[]>(
    platform === "macos"
      ? [
          { label: t("personalization.windowMaterials.solid"), value: "solid" },
          { label: t("personalization.windowMaterials.vibrancy"), value: "vibrancy" },
          ...(liquidGlassSupported
            ? [
                {
                  label: t("personalization.windowMaterials.liquidGlass"),
                  value: "liquid_glass",
                },
              ]
            : []),
        ]
      : platform === "windows"
        ? [
            { label: t("personalization.windowMaterials.solid"), value: "solid" },
            { label: t("personalization.windowMaterials.acrylic"), value: "acrylic" },
            { label: t("personalization.windowMaterials.mica"), value: "mica" },
          ]
        : [{ label: t("personalization.windowMaterials.solid"), value: "solid" }],
  );
  const colorOptions = $derived<Option[]>(
    getThemeOptions().map((option) => ({
      ...option,
      label: t(`personalization.colorThemes.${option.value}`),
    })),
  );
  const customPlanOptions = $derived<Option[]>([
    { label: t("personalization.customThemeLight"), value: "light" },
    { label: t("personalization.customThemeDark"), value: "dark" },
  ]);
  const usesCustomTheme = $derived(
    value.windowMaterial === "solid" && value.colorTheme === "custom",
  );
  const customColors = $derived(value.customTheme[customPlan]);
  const fontSizeProgress = $derived(
    `${((value.fontSize - MIN_FONT_SIZE) / (MAX_FONT_SIZE - MIN_FONT_SIZE)) * 100}%`,
  );
  const fontOptions = $derived([
    { label: t("personalization.systemFont"), value: "" },
    ...(value.fontFamily && !fonts.includes(value.fontFamily)
      ? [{ label: value.fontFamily, value: value.fontFamily, fontFamily: value.fontFamily }]
      : []),
    ...fonts.map((font) => ({ label: font, value: font, fontFamily: font })),
  ]);

  function updateCustomColor(field: keyof ThemeColors, color: string): void {
    onupdate({
      customTheme: {
        ...value.customTheme,
        [customPlan]: { ...value.customTheme[customPlan], [field]: color },
      },
    });
  }

  onMount(async () => {
    platform = /Macintosh|Mac OS X/i.test(navigator.userAgent)
      ? "macos"
      : /Windows/i.test(navigator.userAgent)
        ? "windows"
        : "other";
    try {
      fonts = await getSystemFonts();
    } catch (error) {
      console.error("Failed to load fonts", error);
    }
    if (platform === "macos") {
      try {
        liquidGlassSupported = await supportsLiquidGlass();
      } catch {
        liquidGlassSupported = false;
      }
    }
  });
</script>

<div class="workspace settings-workspace">
  <section class="settings-section">
    <div class="settings-section-heading"><h2>{t("personalization.themeSection")}</h2></div>
    <div class="preference-row">
      <span>{t("personalization.windowMaterial")}</span><Select
        class="settings-select"
        value={value.windowMaterial}
        options={materialOptions}
        onValueChange={(next) =>
          onupdate({ windowMaterial: next as Preferences["windowMaterial"] })}
      />
    </div>
    <div class="preference-row">
      <span>{t("personalization.themeMode")}</span><Select
        class="settings-select"
        value={value.theme}
        options={themeOptions}
        onValueChange={(next) => onupdate({ theme: next as Preferences["theme"] })}
      />
    </div>
    {#if value.windowMaterial === "solid"}<div class="preference-row">
        <span>{t("personalization.colorTheme")}</span><Select
          class="settings-select"
          value={value.colorTheme}
          options={colorOptions}
          onValueChange={(next) => onupdate({ colorTheme: next as ColorThemeId })}
        />
      </div>{/if}
    <div class="preference-row">
      <span>{t("personalization.fontSize")}</span>
      <div class="font-size-control">
        <input
          id="font-size-slider"
          class="settings-slider"
          type="range"
          min={MIN_FONT_SIZE}
          max={MAX_FONT_SIZE}
          step="1"
          value={value.fontSize}
          style={`--slider-progress: ${fontSizeProgress}`}
          aria-label={t("personalization.fontSize")}
          aria-valuetext={`${value.fontSize}px`}
          oninput={(event) => onupdate({ fontSize: Number(event.currentTarget.value) })}
        /><output for="font-size-slider">{value.fontSize}px</output>
      </div>
    </div>
    <div class="preference-row">
      <span>{t("personalization.fontFamily")}</span><Select
        class="settings-select font-family-select"
        value={value.fontFamily}
        options={fontOptions}
        searchable
        searchPlaceholder={t("personalization.searchFont")}
        emptyLabel={t("common.noResults")}
        onValueChange={(next) => onupdate({ fontFamily: next })}
      />
    </div>
  </section>
  <section class:disabled={!usesCustomTheme} class="settings-section custom-theme-section">
    <div class="settings-section-heading"><h2>{t("personalization.customTheme")}</h2></div>
    <div class="preference-row">
      <span>{t("personalization.customThemePalette")}</span><Select
        class="settings-select"
        value={customPlan}
        options={customPlanOptions}
        disabled={!usesCustomTheme}
        onValueChange={(next) => {
          if (next === "light" || next === "dark") customPlan = next;
        }}
      />
    </div>
    <div class="custom-theme-grid">
      {#each customColorFields as field (field.key)}<label class="custom-color-field">
          <span>{t(`personalization.customColors.${field.label}`)}</span>
          <span class="custom-color-control">
            <ColorPicker
              value={customColors[field.key]}
              disabled={!usesCustomTheme}
              label={t(`personalization.customColors.${field.label}`)}
              onvaluechange={(color) => updateCustomColor(field.key, color)}
            />
            <code>{customColors[field.key].toUpperCase()}</code>
          </span>
        </label>{/each}
    </div>
  </section>
</div>
