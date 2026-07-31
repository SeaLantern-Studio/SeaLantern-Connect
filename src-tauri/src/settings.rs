use sculk::persist;
use sculk::tunnel::{RelayUrl, SecretKey};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

const APP_DIR_NAME: &str = "sealantern-connect";
const PREFERENCES_FILE: &str = "preferences.conf";
const KEY_FILE: &str = "secret.key";
const HOST_STATE_FILE: &str = "host.state";
const DEFAULT_JOIN_PORT: u16 = 25_565;
const SPLASH_DURATION_OPTIONS_MS: [u32; 5] = [0, 500, 1000, 1500, 2000];
pub const RECONNECT_TIMEOUT_OPTIONS_SECS: [u64; 5] = [10, 15, 20, 30, 60];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseAction {
    Ask,
    Exit,
    HideToTray,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    theme: String,
    color_theme: String,
    splash_duration_ms: u32,
    locale: String,
    remember_window_state: bool,
    close_action: String,
    host_uri_lifetime: String,
    join_uri: String,
    join_port: u16,
    reconnect_timeout_secs: Option<u64>,
    relay_custom: bool,
    relay_url: String,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: "system".to_owned(),
            color_theme: "default".to_owned(),
            splash_duration_ms: 1000,
            locale: "zh-CN".to_owned(),
            remember_window_state: true,
            close_action: "ask".to_owned(),
            host_uri_lifetime: "always".to_owned(),
            join_uri: String::new(),
            join_port: DEFAULT_JOIN_PORT,
            reconnect_timeout_secs: None,
            relay_custom: false,
            relay_url: String::new(),
        }
    }
}

pub struct SettingsState {
    data_dir: PathBuf,
    path: PathBuf,
    secret_key: SecretKey,
    preferences: Mutex<Preferences>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalizationUpdate {
    theme: String,
    color_theme: String,
    splash_duration_ms: u32,
    locale: String,
    remember_window_state: bool,
    close_action: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSettingsUpdate {
    relay_custom: bool,
    relay_url: String,
    reconnect_timeout_secs: Option<u64>,
}

impl SettingsState {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let data_dir = app
            .path()
            .data_dir()
            .map_err(|error| error.to_string())?
            .join(APP_DIR_NAME);
        std::fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
        let path = data_dir.join(PREFERENCES_FILE);
        migrate_legacy_preferences(app, &path)?;
        let preferences = std::fs::read_to_string(&path)
            .map(|content| parse_preferences(&content))
            .unwrap_or_default();
        let secret_key = persist::load_or_generate_key(&data_dir.join(KEY_FILE))
            .map_err(|error| error.to_string())?;
        Ok(Self {
            data_dir,
            path,
            secret_key,
            preferences: Mutex::new(preferences),
        })
    }

    pub fn remember_join_uri(&self, join_uri: String) -> Result<(), String> {
        self.update(|preferences| preferences.join_uri = join_uri)
    }

    pub fn set_join_port(&self, port: u16) -> Result<(), String> {
        if port == 0 {
            return Err("本地端口必须在 1 到 65535 之间".to_owned());
        }
        self.update(|preferences| preferences.join_port = port)
    }

    pub fn host_secret_key(&self) -> SecretKey {
        self.secret_key.clone()
    }

    pub fn host_state_path(&self) -> PathBuf {
        self.data_dir.join(HOST_STATE_FILE)
    }

    pub fn relay_url(&self) -> Result<Option<RelayUrl>, String> {
        let preferences = self
            .preferences
            .lock()
            .map_err(|_| "settings state is unavailable".to_owned())?;
        if !preferences.relay_custom {
            return Ok(None);
        }
        preferences
            .relay_url
            .trim()
            .parse::<RelayUrl>()
            .map(Some)
            .map_err(|error| error.to_string())
    }

    pub fn reconnect_timeout(&self) -> Result<Option<Duration>, String> {
        self.preferences
            .lock()
            .map(|preferences| preferences.reconnect_timeout_secs.map(Duration::from_secs))
            .map_err(|_| "settings state is unavailable".to_owned())
    }

    pub fn close_action(&self) -> CloseAction {
        self.preferences
            .lock()
            .map(|preferences| match preferences.close_action.as_str() {
                "exit" => CloseAction::Exit,
                "hide_to_tray" => CloseAction::HideToTray,
                _ => CloseAction::Ask,
            })
            .unwrap_or(CloseAction::Ask)
    }

    pub fn remembers_window_state(&self) -> bool {
        self.preferences
            .lock()
            .is_ok_and(|preferences| preferences.remember_window_state)
    }

    pub fn persist(&self) -> Result<(), String> {
        let preferences = self
            .preferences
            .lock()
            .map_err(|_| "settings state is unavailable".to_owned())?;
        save_preferences(&self.path, &preferences)
    }

    fn update(&self, apply: impl FnOnce(&mut Preferences)) -> Result<(), String> {
        let mut preferences = self
            .preferences
            .lock()
            .map_err(|_| "settings state is unavailable".to_owned())?;
        apply(&mut preferences);
        save_preferences(&self.path, &preferences)
    }
}

fn migrate_legacy_preferences(app: &AppHandle, destination: &PathBuf) -> Result<(), String> {
    if destination.exists() {
        return Ok(());
    }
    let source = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join(PREFERENCES_FILE);
    if source.exists() {
        std::fs::copy(source, destination).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_preferences(state: State<'_, SettingsState>) -> Result<Preferences, String> {
    state
        .preferences
        .lock()
        .map(|preferences| preferences.clone())
        .map_err(|_| "settings state is unavailable".to_owned())
}

#[tauri::command]
pub fn set_theme(theme: String, state: State<'_, SettingsState>) -> Result<(), String> {
    if !matches!(theme.as_str(), "system" | "light" | "dark") {
        return Err("invalid theme preference".to_owned());
    }
    state.update(|preferences| preferences.theme = theme)
}

#[tauri::command]
pub fn set_color_theme(color_theme: String, state: State<'_, SettingsState>) -> Result<(), String> {
    if !is_color_theme(&color_theme) {
        return Err("invalid color theme".to_owned());
    }
    state.update(|preferences| preferences.color_theme = color_theme)
}

#[tauri::command]
pub fn set_locale(locale: String, state: State<'_, SettingsState>) -> Result<(), String> {
    if !matches!(locale.as_str(), "zh-CN" | "en") {
        return Err("invalid locale preference".to_owned());
    }
    state.update(|preferences| preferences.locale = locale)
}

#[tauri::command]
pub fn set_close_action(
    close_action: String,
    state: State<'_, SettingsState>,
) -> Result<(), String> {
    if !matches!(close_action.as_str(), "ask" | "exit" | "hide_to_tray") {
        return Err("invalid close action".to_owned());
    }
    state.update(|preferences| preferences.close_action = close_action)
}

#[tauri::command]
pub fn set_host_uri_lifetime(
    lifetime: String,
    state: State<'_, SettingsState>,
) -> Result<(), String> {
    if !is_host_uri_lifetime(&lifetime) {
        return Err("invalid host URI lifetime".to_owned());
    }
    state.update(|preferences| preferences.host_uri_lifetime = lifetime)
}

#[tauri::command]
pub fn set_personalization(
    update: PersonalizationUpdate,
    state: State<'_, SettingsState>,
) -> Result<(), String> {
    if !matches!(update.theme.as_str(), "system" | "light" | "dark") {
        return Err("invalid theme preference".to_owned());
    }
    if !is_color_theme(&update.color_theme) {
        return Err("invalid color theme".to_owned());
    }
    if !SPLASH_DURATION_OPTIONS_MS.contains(&update.splash_duration_ms) {
        return Err("invalid splash duration".to_owned());
    }
    if !matches!(update.locale.as_str(), "zh-CN" | "en") {
        return Err("invalid locale preference".to_owned());
    }
    if !matches!(
        update.close_action.as_str(),
        "ask" | "exit" | "hide_to_tray"
    ) {
        return Err("invalid close action".to_owned());
    }
    state.update(|preferences| {
        preferences.theme = update.theme;
        preferences.color_theme = update.color_theme;
        preferences.splash_duration_ms = update.splash_duration_ms;
        preferences.locale = update.locale;
        preferences.remember_window_state = update.remember_window_state;
        preferences.close_action = update.close_action;
    })
}

#[tauri::command]
pub fn set_connection_settings(
    update: ConnectionSettingsUpdate,
    state: State<'_, SettingsState>,
) -> Result<(), String> {
    if let Some(timeout) = update.reconnect_timeout_secs
        && !RECONNECT_TIMEOUT_OPTIONS_SECS.contains(&timeout)
    {
        return Err("invalid reconnect timeout".to_owned());
    }
    let relay_url = update.relay_url.trim().to_owned();
    if update.relay_custom {
        relay_url
            .parse::<RelayUrl>()
            .map_err(|error| error.to_string())?;
    }
    state.update(|preferences| {
        preferences.relay_custom = update.relay_custom;
        preferences.relay_url = relay_url;
        preferences.reconnect_timeout_secs = update.reconnect_timeout_secs;
    })
}

#[tauri::command]
pub fn set_join_port(port: u16, state: State<'_, SettingsState>) -> Result<(), String> {
    state.set_join_port(port)
}

fn save_preferences(path: &PathBuf, preferences: &Preferences) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "settings directory is unavailable".to_owned())?;
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let content = format!(
        "theme={}\ncolor_theme={}\nsplash_duration_ms={}\nlocale={}\nremember_window_state={}\nclose_action={}\nhost_uri_lifetime={}\njoin_uri={}\njoin_port={}\nreconnect_timeout_secs={}\nrelay_custom={}\nrelay_url={}\n",
        preferences.theme,
        preferences.color_theme,
        preferences.splash_duration_ms,
        preferences.locale,
        preferences.remember_window_state,
        preferences.close_action,
        preferences.host_uri_lifetime,
        preferences.join_uri,
        preferences.join_port,
        preferences
            .reconnect_timeout_secs
            .map_or_else(|| "unlimited".to_owned(), |value| value.to_string()),
        preferences.relay_custom,
        preferences.relay_url,
    );
    std::fs::write(path, content).map_err(|error| error.to_string())
}

fn parse_preferences(content: &str) -> Preferences {
    let mut preferences = Preferences::default();
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("theme=") {
            let value = value.trim();
            if matches!(value, "system" | "light" | "dark") {
                preferences.theme = value.to_owned();
            }
        } else if let Some(value) = line.strip_prefix("color_theme=") {
            let value = value.trim();
            if is_color_theme(value) {
                preferences.color_theme = value.to_owned();
            }
        } else if let Some(value) = line.strip_prefix("splash_duration_ms=") {
            preferences.splash_duration_ms = value
                .trim()
                .parse::<u32>()
                .ok()
                .filter(|duration| SPLASH_DURATION_OPTIONS_MS.contains(duration))
                .unwrap_or(1000);
        } else if let Some(value) = line.strip_prefix("locale=") {
            let value = value.trim();
            if matches!(value, "zh-CN" | "en") {
                preferences.locale = value.to_owned();
            }
        } else if let Some(value) = line.strip_prefix("remember_window_state=") {
            preferences.remember_window_state = value.trim() == "true";
        } else if let Some(value) = line.strip_prefix("close_action=") {
            let value = value.trim();
            if matches!(value, "ask" | "exit" | "hide_to_tray") {
                preferences.close_action = value.to_owned();
            }
        } else if let Some(value) = line.strip_prefix("host_uri_lifetime=") {
            let value = value.trim();
            if is_host_uri_lifetime(value) {
                preferences.host_uri_lifetime = value.to_owned();
            }
        } else if let Some(value) = line.strip_prefix("join_uri=") {
            preferences.join_uri = value.trim().to_owned();
        } else if let Some(value) = line.strip_prefix("join_port=")
            && let Ok(port) = value.trim().parse::<u16>()
            && port != 0
        {
            preferences.join_port = port;
        } else if let Some(value) = line.strip_prefix("reconnect_timeout_secs=") {
            let value = value.trim();
            preferences.reconnect_timeout_secs = value
                .parse::<u64>()
                .ok()
                .filter(|timeout| RECONNECT_TIMEOUT_OPTIONS_SECS.contains(timeout));
        } else if let Some(value) = line.strip_prefix("relay_custom=") {
            preferences.relay_custom = value.trim() == "true";
        } else if let Some(value) = line.strip_prefix("relay_url=") {
            preferences.relay_url = value.trim().to_owned();
        }
    }
    preferences
}

fn is_color_theme(value: &str) -> bool {
    matches!(value, "default" | "midnight" | "ocean" | "rose" | "sunset")
}

fn is_host_uri_lifetime(value: &str) -> bool {
    matches!(
        value,
        "always" | "never" | "1h" | "3h" | "6h" | "12h" | "24h"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_values_use_defaults() {
        let preferences = parse_preferences("");

        assert_eq!(preferences, Preferences::default());
    }

    #[test]
    fn parses_saved_preferences() {
        let preferences = parse_preferences(
            "theme=dark\ncolor_theme=ocean\nsplash_duration_ms=2000\nlocale=en\nremember_window_state=false\nclose_action=exit\njoin_uri=sculk://join/v1/example\njoin_port=25566\nreconnect_timeout_secs=30\nrelay_custom=true\nrelay_url=https://relay.example.com\nwindow_x=100\nwindow_y=200\nwindow_width=960\nwindow_height=640\nwindow_maximized=true\n",
        );

        assert_eq!(preferences.theme, "dark");
        assert_eq!(preferences.color_theme, "ocean");
        assert_eq!(preferences.splash_duration_ms, 2000);
        assert_eq!(preferences.locale, "en");
        assert!(!preferences.remember_window_state);
        assert_eq!(preferences.close_action, "exit");
        assert_eq!(preferences.join_uri, "sculk://join/v1/example");
        assert_eq!(preferences.join_port, 25_566);
        assert_eq!(preferences.reconnect_timeout_secs, Some(30));
        assert!(preferences.relay_custom);
        assert_eq!(preferences.relay_url, "https://relay.example.com");
    }

    #[test]
    fn ignores_unknown_theme() {
        let preferences = parse_preferences(
            "theme=midnight\ncolor_theme=unknown\nsplash_duration_ms=4000\nlocale=fr\nclose_action=minimize\nreconnect_timeout_secs=45\n",
        );

        assert_eq!(preferences.theme, "system");
        assert_eq!(preferences.color_theme, "default");
        assert_eq!(preferences.splash_duration_ms, 1000);
        assert_eq!(preferences.locale, "zh-CN");
        assert_eq!(preferences.close_action, "ask");
        assert_eq!(preferences.reconnect_timeout_secs, None);
    }
}
