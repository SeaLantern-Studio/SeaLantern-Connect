import { invoke } from "@tauri-apps/api/core";
import type {
  ApplicationSettingsUpdate,
  ConnectionSettingsUpdate,
  HostUriLifetime,
  Locale,
  LightweightSettingsUpdate,
  PersonalizationUpdate,
  Preferences,
  SystemTheme,
  ThemePreference,
} from "@models/preferences";

export function getPreferences(): Promise<Preferences> {
  return invoke("get_preferences");
}

export function getSystemFonts(): Promise<string[]> {
  return invoke("get_system_fonts");
}

export function getSystemTheme(): Promise<SystemTheme> {
  return invoke("get_system_theme");
}

export function supportsLiquidGlass(): Promise<boolean> {
  return invoke("supports_liquid_glass");
}

export function saveTheme(theme: ThemePreference, systemTheme: SystemTheme): Promise<void> {
  return invoke("set_theme", { theme, systemTheme });
}

export function saveLocale(locale: Locale): Promise<void> {
  return invoke("set_locale", { locale });
}

export function saveInviteLifetime(lifetime: HostUriLifetime): Promise<void> {
  return invoke("set_invite_lifetime", { lifetime });
}

export function savePersonalization(
  update: PersonalizationUpdate,
  systemTheme: SystemTheme,
): Promise<void> {
  return invoke("set_personalization", { update, systemTheme });
}

export function saveApplicationSettings(update: ApplicationSettingsUpdate): Promise<void> {
  return invoke("set_application_settings", { update });
}

export function saveConnectionSettings(update: ConnectionSettingsUpdate): Promise<void> {
  return invoke("set_connection_settings", { update });
}

export function saveLightweightSettings(update: LightweightSettingsUpdate): Promise<void> {
  return invoke("set_lightweight_settings", { update });
}
