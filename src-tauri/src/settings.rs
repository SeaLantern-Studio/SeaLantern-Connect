use sculk::persist::{self, HostState};
use sculk::tunnel::{AccessToken, SecretKey, ServiceId, TokenState};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;
use tauri::{AppHandle, Manager, State};

const APP_DIR_NAME: &str = "sealantern-connect";
const PREFERENCES_FILE: &str = "preferences.conf";
const KEY_FILE: &str = "secret.key";
const HOST_STATE_FILE: &str = "host.state";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    theme: String,
    join_uri: String,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: "system".to_owned(),
            join_uri: String::new(),
        }
    }
}

pub struct SettingsState {
    data_dir: PathBuf,
    path: PathBuf,
    secret_key: SecretKey,
    preferences: Mutex<Preferences>,
}

pub struct HostIdentity {
    pub secret_key: SecretKey,
    pub service_id: ServiceId,
    pub token: AccessToken,
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

    pub fn host_identity(&self) -> Result<HostIdentity, String> {
        let path = self.data_dir.join(HOST_STATE_FILE);
        let state = match persist::load_host_state(&path).map_err(|error| error.to_string())? {
            Some(state) => state,
            None => {
                let state = HostState {
                    service_id: ServiceId::generate(),
                    token_state: TokenState::new(AccessToken::generate(), SystemTime::now()),
                };
                persist::save_host_state(&path, &state).map_err(|error| error.to_string())?;
                state
            }
        };
        Ok(HostIdentity {
            secret_key: self.secret_key.clone(),
            service_id: state.service_id,
            token: state.token_state.token().clone(),
        })
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

fn save_preferences(path: &PathBuf, preferences: &Preferences) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "settings directory is unavailable".to_owned())?;
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let content = format!(
        "theme={}\njoin_uri={}\n",
        preferences.theme, preferences.join_uri
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
        } else if let Some(value) = line.strip_prefix("join_uri=") {
            preferences.join_uri = value.trim().to_owned();
        }
    }
    preferences
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
        let preferences = parse_preferences("theme=dark\njoin_uri=sculk://join/v1/example\n");

        assert_eq!(preferences.theme, "dark");
        assert_eq!(preferences.join_uri, "sculk://join/v1/example");
    }

    #[test]
    fn ignores_unknown_theme() {
        let preferences = parse_preferences("theme=midnight\n");

        assert_eq!(preferences.theme, "system");
    }
}
