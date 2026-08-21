<script lang="ts">
  import { onMount } from "svelte";
  import { getSystemFonts, supportsLiquidGlass } from "@api/settings";
  import { t } from "@i18n";
  import type { ColorThemeId } from "@models/preferences";
  import type { Preferences, ThemeColors } from "@models/preferences";
  import { getThemeOptions } from "@themes";
  import { MAX_FONT_SIZE, MIN_FONT_SIZE } from "@themes/typography";
  import ColorPicker from "./ui/ColorPicker.svelte";
  import Select, { type Option, type PointerOrigin } from "./ui/Select.svelte";
  import Toggle from "./ui/Toggle.svelte";
  import { X } from "@lucide/svelte";

  let { value, onupdate, onthemechange } = $props<{
    value: Preferences;
    onupdate: (value: Partial<Preferences>) => void;
    onthemechange?: (theme: Preferences["theme"], origin: PointerOrigin) => void;
  }>();
  let fonts = $state<string[]>([]);
  let liquidGlassSupported = $state(false);
  let platform = $state<"macos" | "windows" | "other">("other");
  let customPlan = $state<"light" | "dark">("light");
  let backgroundFileInput = $state<HTMLInputElement>();
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

  function chooseBackground(): void {
    backgroundFileInput?.click();
  }

  function handleBackgroundFile(event: Event): void {
    const file = (event.currentTarget as HTMLInputElement).files?.[0];
    if (!file || !file.type.startsWith("image/")) return;
    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result === "string") onupdate({ backgroundImage: reader.result, backgroundEnabled: true });
    };
    reader.readAsDataURL(file);
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
        onValueChange={(next, origin) => {
          const theme = next as Preferences["theme"];
          if (origin) onthemechange?.(theme, origin);
          else onupdate({ theme });
        }}
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
  </section>
  <section class="settings-section background-section">
    <div class="settings-section-heading"><h2>{t("personalization.backgroundSection")}</h2></div>
    <div class="preference-row switch-row">
      <span>{t("personalization.backgroundEnabled")}</span><Toggle
        checked={value.backgroundEnabled}
        label={t("personalization.backgroundEnabled")}
        oncheckedchange={(checked) => onupdate({ backgroundEnabled: checked })}
      />
    </div>
    {#if value.backgroundEnabled}<div class="background-settings">
        <input
          bind:this={backgroundFileInput}
          class="background-file-input"
          type="file"
          accept="image/*"
          onchange={handleBackgroundFile}
        />
        <div class="preference-row background-preview-row">
          <span>{t("personalization.backgroundPreview")}</span>
          <div class="background-picker">
            <div
              class:has-image={Boolean(value.backgroundImage)}
              class="background-preview"
              style={value.backgroundImage
                ? `background-image: url(${JSON.stringify(value.backgroundImage)})`
                : ""}
              role="button"
              tabindex="0"
              aria-label={t("personalization.chooseBackground")}
              onclick={() => !value.backgroundImage && chooseBackground()}
              onkeydown={(event) => {
                if (!value.backgroundImage && (event.key === "Enter" || event.key === " ")) chooseBackground();
              }}
            >
              {#if value.backgroundImage}<button
                  class="background-remove-button"
                  type="button"
                  aria-label={t("personalization.clearBackground")}
                  onclick={(event) => {
                    event.stopPropagation();
                    onupdate({ backgroundImage: "" });
                  }}><X size={13} /></button
                >{/if}
              {#if !value.backgroundImage}<span>{t("personalization.chooseBackground")}</span>{/if}
            </div>
          </div>
        </div>
        <div class="preference-row"><span>{t("personalization.backgroundOpacity")}</span><div class="font-size-control"><input class="settings-slider" type="range" min="0" max="1" step="0.05" value={value.backgroundOpacity} style={`--slider-progress: ${value.backgroundOpacity * 100}%`} oninput={(event) => onupdate({ backgroundOpacity: Number(event.currentTarget.value) })} /><output>{Math.round(value.backgroundOpacity * 100)}%</output></div></div>
        <div class="preference-row"><span>{t("personalization.backgroundBlur")}</span><div class="font-size-control"><input class="settings-slider" type="range" min="0" max="20" step="1" value={value.backgroundBlur} style={`--slider-progress: ${value.backgroundBlur * 5}%`} oninput={(event) => onupdate({ backgroundBlur: Number(event.currentTarget.value) })} /><output>{value.backgroundBlur}px</output></div></div>
        <div class="preference-row"><span>{t("personalization.backgroundBrightness")}</span><div class="font-size-control"><input class="settings-slider" type="range" min="0.5" max="1.5" step="0.1" value={value.backgroundBrightness} style={`--slider-progress: ${(value.backgroundBrightness - 0.5) * 100}%`} oninput={(event) => onupdate({ backgroundBrightness: Number(event.currentTarget.value) })} /><output>{value.backgroundBrightness.toFixed(1)}</output></div></div>
        <div class="preference-row"><span>{t("personalization.backgroundCardBlur")}</span><div class="font-size-control"><input class="settings-slider" type="range" min="8" max="30" step="1" value={value.backgroundCardBlur} style={`--slider-progress: ${((value.backgroundCardBlur - 8) / 22) * 100}%`} oninput={(event) => onupdate({ backgroundCardBlur: Number(event.currentTarget.value) })} /><output>{value.backgroundCardBlur}px</output></div></div>
      </div>{/if}
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
