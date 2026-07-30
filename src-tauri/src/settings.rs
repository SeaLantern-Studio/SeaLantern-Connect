use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

const PREFERENCES_FILE: &str = "preferences.conf";

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
    path: PathBuf,
    preferences: Mutex<Preferences>,
}

impl SettingsState {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let path = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?
            .join(PREFERENCES_FILE);
        let preferences = std::fs::read_to_string(&path)
            .map(|content| parse_preferences(&content))
            .unwrap_or_default();
        Ok(Self {
            path,
            preferences: Mutex::new(preferences),
        })
    }

    pub fn remember_join_uri(&self, join_uri: String) -> Result<(), String> {
        self.update(|preferences| preferences.join_uri = join_uri)
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
    let content = format!("theme={}\njoin_uri={}\n", preferences.theme, preferences.join_uri);
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
