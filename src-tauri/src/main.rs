// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod connect;
mod settings;
mod tray;

fn main() {
    tauri::Builder::default()
        .manage(connect::ConnectState::new())
        .setup(|app| {
            connect::setup(app)?;
            tray::setup(app)?;
            Ok(())
        })
        .on_window_event(tray::handle_window_event)
        .invoke_handler(tauri::generate_handler![
            connect::validate_invite,
            connect::get_status,
            connect::start_join,
            connect::stop_join,
            settings::get_preferences,
            settings::set_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
