use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, Window, WindowEvent};

const MAIN_WINDOW_LABEL: &str = "main";
const SHOW_MENU_ID: &str = "tray-show";
const QUIT_MENU_ID: &str = "tray-quit";

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, SHOW_MENU_ID, "显示主窗口", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
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
            QUIT_MENU_ID => app.exit(0),
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
    if window.label() == MAIN_WINDOW_LABEL
        && let WindowEvent::CloseRequested { api, .. } = event
    {
        api.prevent_close();
        let _ = window.hide();
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
