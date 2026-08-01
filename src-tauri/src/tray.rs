use crate::lightweight::{self, LightweightState};
use crate::settings::{CloseAction, SettingsState};
use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter, Manager, Window, WindowEvent, Wry};

const MAIN_WINDOW_LABEL: &str = "main";
const SHOW_MENU_ID: &str = "tray-show";
const LIGHTWEIGHT_MENU_ID: &str = "tray-lightweight";
const QUIT_MENU_ID: &str = "tray-quit";

struct TrayMenuState {
    show: MenuItem<Wry>,
    lightweight: CheckMenuItem<Wry>,
    quit: MenuItem<Wry>,
}

#[derive(Debug, PartialEq, Eq)]
struct TrayLabels {
    show: &'static str,
    lightweight: &'static str,
    quit: &'static str,
}

fn tray_labels(locale: &str) -> TrayLabels {
    if locale == "zh-CN" {
        TrayLabels {
            show: "显示主窗口",
            lightweight: "轻量模式",
            quit: "退出",
        }
    } else {
        TrayLabels {
            show: "Show Main Window",
            lightweight: "Lightweight Mode",
            quit: "Quit",
        }
    }
}

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let labels = tray_labels(&app.state::<SettingsState>().locale());
    let show = MenuItem::with_id(app, SHOW_MENU_ID, labels.show, true, None::<&str>)?;
    let lightweight = CheckMenuItem::with_id(
        app,
        LIGHTWEIGHT_MENU_ID,
        labels.lightweight,
        true,
        false,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, labels.quit, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &lightweight, &quit])?;
    app.state::<LightweightState>()
        .set_tray_item(lightweight.clone());
    app.manage(TrayMenuState {
        show: show.clone(),
        lightweight: lightweight.clone(),
        quit: quit.clone(),
    });
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| std::io::Error::other("default application icon is unavailable"))?;

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("SeaLantern Connect")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SHOW_MENU_ID => show_main_window(app),
            LIGHTWEIGHT_MENU_ID => toggle_lightweight_mode(app),
            QUIT_MENU_ID => {
                let _ = app.state::<SettingsState>().persist();
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn update_locale(app: &AppHandle) -> Result<(), String> {
    let labels = tray_labels(&app.state::<SettingsState>().locale());
    let state = app.state::<TrayMenuState>();
    state
        .show
        .set_text(labels.show)
        .map_err(|error| error.to_string())?;
    state
        .lightweight
        .set_text(labels.lightweight)
        .map_err(|error| error.to_string())?;
    state
        .quit
        .set_text(labels.quit)
        .map_err(|error| error.to_string())
}

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW_LABEL {
        return;
    }
    let settings = window.state::<SettingsState>();
    if let WindowEvent::CloseRequested { api, .. } = event {
        let _ = settings.persist();
        match settings.close_action() {
            CloseAction::HideToTray => {
                api.prevent_close();
                let _ = window.hide();
            }
            CloseAction::Ask => {
                api.prevent_close();
                let _ = window.emit("close-action-requested", ());
            }
            CloseAction::Exit => {
                api.prevent_close();
                window.app_handle().exit(0);
            }
        }
    }
}

fn show_main_window(app: &AppHandle) {
    let lightweight_state = app.state::<LightweightState>();
    if lightweight_state.is_active() {
        if let Err(error) = lightweight::exit(app) {
            eprintln!("failed to exit lightweight mode: {error}");
        }
        return;
    }
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn toggle_lightweight_mode(app: &AppHandle) {
    let result = if app.state::<LightweightState>().is_active() {
        lightweight::exit(app)
    } else {
        lightweight::enter(app)
    };
    if let Err(error) = result {
        eprintln!("failed to toggle lightweight mode: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_tray_labels_for_locale() {
        assert_eq!(tray_labels("zh-CN").quit, "退出");
        assert_eq!(tray_labels("en").lightweight, "Lightweight Mode");
        assert_eq!(tray_labels("unknown").show, "Show Main Window");
    }
}
