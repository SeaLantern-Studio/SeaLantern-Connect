use crate::lightweight::{self, LightweightState};
use crate::settings::{CloseAction, SettingsState};
use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter, Manager, Window, WindowEvent};

const MAIN_WINDOW_LABEL: &str = "main";
const SHOW_MENU_ID: &str = "tray-show";
const LIGHTWEIGHT_MENU_ID: &str = "tray-lightweight";
const QUIT_MENU_ID: &str = "tray-quit";

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, SHOW_MENU_ID, "显示主窗口", true, None::<&str>)?;
    let lightweight = CheckMenuItem::with_id(
        app,
        LIGHTWEIGHT_MENU_ID,
        "轻量模式",
        true,
        false,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &lightweight, &quit])?;
    app.state::<LightweightState>()
        .set_tray_item(lightweight.clone());
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
