// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod connect;

fn main() {
    tauri::Builder::default()
        .manage(connect::ConnectState::new())
        .setup(connect::setup)
        .invoke_handler(tauri::generate_handler![
            connect::validate_invite,
            connect::get_status,
            connect::start_join,
            connect::stop_join,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
