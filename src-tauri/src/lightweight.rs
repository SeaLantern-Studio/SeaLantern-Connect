use crate::settings::SettingsState;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use tauri::menu::CheckMenuItem;
use tauri::{AppHandle, Manager, WebviewWindowBuilder, Wry};
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};

const MAIN_WINDOW_LABEL: &str = "main";

pub(crate) struct LightweightState {
    active: AtomicBool,
    tray_item: Mutex<Option<CheckMenuItem<Wry>>>,
}

impl LightweightState {
    pub(crate) fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            tray_item: Mutex::new(None),
        }
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub(crate) fn set_tray_item(&self, item: CheckMenuItem<Wry>) {
        if let Ok(mut tray_item) = self.tray_item.lock() {
            *tray_item = Some(item);
        }
    }

    fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Release);
        if let Ok(tray_item) = self.tray_item.lock()
            && let Some(tray_item) = tray_item.as_ref()
            && tray_item.is_checked().ok() != Some(active)
        {
            let _ = tray_item.set_checked(active);
        }
    }
}

fn window_state_flags() -> StateFlags {
    StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED
}

pub(crate) fn enter(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<LightweightState>();
    if state.is_active() {
        return Ok(());
    }

    let result = (|| {
        let window = app
            .get_webview_window(MAIN_WINDOW_LABEL)
            .ok_or_else(|| "main window is unavailable".to_owned())?;
        app.save_window_state(window_state_flags())
            .map_err(|error| error.to_string())?;
        #[cfg(target_os = "windows")]
        let _ = window.set_skip_taskbar(true);
        window.destroy().map_err(|error| error.to_string())
    })();

    match result {
        Ok(()) => {
            #[cfg(target_os = "macos")]
            let _ = app.set_activation_policy(ActivationPolicy::Accessory);
            state.set_active(true);
            Ok(())
        }
        Err(error) => {
            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                let _ = window.set_skip_taskbar(false);
            }
            state.set_active(false);
            Err(error)
        }
    }
}

pub(crate) fn exit(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<LightweightState>();
    if !state.is_active() {
        return Ok(());
    }

    let result = (|| {
        #[cfg(target_os = "macos")]
        app.set_activation_policy(ActivationPolicy::Regular)
            .map_err(|error| error.to_string())?;

        let window = if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
            window
        } else {
            let config = app
                .config()
                .app
                .windows
                .iter()
                .find(|config| config.label == MAIN_WINDOW_LABEL)
                .ok_or_else(|| "main window configuration is unavailable".to_owned())?;
            WebviewWindowBuilder::from_config(app, config)
                .map_err(|error| error.to_string())?
                .build()
                .map_err(|error| error.to_string())?
        };

        if app.state::<SettingsState>().remembers_window_state() {
            let _ = window.restore_state(window_state_flags());
        }
        window.unminimize().map_err(|error| error.to_string())?;
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        #[cfg(target_os = "windows")]
        let _ = window.set_skip_taskbar(false);
        Ok(())
    })();

    match result {
        Ok(()) => {
            state.set_active(false);
            Ok(())
        }
        Err(error) => {
            #[cfg(target_os = "macos")]
            let _ = app.set_activation_policy(ActivationPolicy::Accessory);
            state.set_active(true);
            Err(error)
        }
    }
}
