// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod connect;
mod settings;
mod tray;

use tauri::Manager;
use tauri_plugin_window_state::{StateFlags, WindowExt};

fn window_state_flags() -> StateFlags {
    StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED
}

fn main() {
    tauri::Builder::default()
        .manage(connect::ConnectState::new())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(window_state_flags())
                .skip_initial_state("main")
                .build(),
        )
        .setup(|app| {
            connect::setup(app)?;
            if app
                .state::<settings::SettingsState>()
                .remembers_window_state()
                && let Some(window) = app.get_webview_window("main")
            {
                window.restore_state(window_state_flags())?;
            }
            tray::setup(app)?;
            Ok(())
        })
        .on_window_event(tray::handle_window_event)
        .invoke_handler(tauri::generate_handler![
            connect::validate_invite,
            connect::get_status,
            connect::start_lan_scan,
            connect::get_lan_scan,
            connect::restart_lan_scan,
            connect::stop_lan_scan,
            connect::start_host,
            connect::start_join,
            connect::stop_join,
            connect::stop_tunnel,
            settings::get_preferences,
            settings::set_theme,
            settings::set_color_theme,
            settings::set_locale,
            settings::set_close_action,
            settings::set_join_port,
            settings::set_personalization,
            settings::set_connection_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
