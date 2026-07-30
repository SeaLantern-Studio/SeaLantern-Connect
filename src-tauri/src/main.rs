// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod connect;
mod settings;
mod tray;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .manage(connect::ConnectState::new())
        .setup(|app| {
            connect::setup(app)?;
            app.state::<settings::SettingsState>()
                .restore_window(app.handle());
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
            settings::set_join_port,
            settings::set_personalization,
            settings::set_connection_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
