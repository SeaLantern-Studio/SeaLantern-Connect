import { invoke } from "@tauri-apps/api/core";
import type {
  CloseAction,
  ConnectionSettingsUpdate,
  HostUriLifetime,
  Locale,
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

export function saveTheme(theme: ThemePreference): Promise<void> {
  return invoke("set_theme", { theme });
}

export function saveLocale(locale: Locale): Promise<void> {
  return invoke("set_locale", { locale });
}

export function saveCloseAction(closeAction: CloseAction): Promise<void> {
  return invoke("set_close_action", { closeAction });
}

export function saveInviteLifetime(lifetime: HostUriLifetime): Promise<void> {
  return invoke("set_invite_lifetime", { lifetime });
}

export function savePersonalization(update: PersonalizationUpdate): Promise<void> {
  return invoke("set_personalization", { update });
}

export function saveConnectionSettings(update: ConnectionSettingsUpdate): Promise<void> {
  return invoke("set_connection_settings", { update });
}
