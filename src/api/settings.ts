import { invoke } from "@tauri-apps/api/core";
import type {
  ApplicationSettingsUpdate,
  ConnectionSettingsUpdate,
  HostUriLifetime,
  Locale,
  LightweightSettingsUpdate,
  PersonalizationUpdate,
  Preferences,
  ThemePreference,
} from "../models/preferences";

export function getPreferences(): Promise<Preferences> {
  return invoke("get_preferences");
}

export function getSystemFonts(): Promise<string[]> {
  return invoke("get_system_fonts");
}

export function supportsLiquidGlass(): Promise<boolean> {
  return invoke("supports_liquid_glass");
}

export function saveTheme(theme: ThemePreference): Promise<void> {
  return invoke("set_theme", { theme });
}

export function saveLocale(locale: Locale): Promise<void> {
  return invoke("set_locale", { locale });
}

export function saveInviteLifetime(lifetime: HostUriLifetime): Promise<void> {
  return invoke("set_invite_lifetime", { lifetime });
}

export function savePersonalization(update: PersonalizationUpdate): Promise<void> {
  return invoke("set_personalization", { update });
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
