// Protocol activation starts a short-lived second process on Windows. Using the GUI
// subsystem in debug builds too prevents that process from flashing a console window.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod connection;
mod desktop;
mod settings;

use connection::{host, join};
use desktop::{autodelay, deeplink, effects, tray, window_state};
use tauri::Manager;
use tauri_plugin_window_state::{StateFlags, WindowExt};

fn window_state_flags() -> StateFlags {
    StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED
}

#[tauri::command]
async fn stop_tunnel(app: tauri::AppHandle) -> Result<(), String> {
    match app.state::<connection::ConnectState>().active_mode() {
        Some(connection::ConnectMode::Host) => host::stop(&app),
        Some(connection::ConnectMode::Join) => join::stop(&app).await,
        None => Ok(()),
    }
}

#[tauri::command]
fn exit_application(app: tauri::AppHandle) {
    app.exit(0);
}

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            deeplink::stash_restore_links(app, &args);
            tray::show_main_window(app);
        }))
        .manage(connection::ConnectState::new())
        .manage(host::HostState::new())
        .manage(join::JoinState::new())
        .manage(window_state::MainWindowState::new())
        .manage(autodelay::AutoDelay::new())
        .manage(deeplink::PendingDeepLinks::default())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(window_state_flags())
                .skip_initial_state("main")
                .build(),
        )
        .setup(|app| {
            app.manage(settings::SettingsState::load(app.handle())?);
            deeplink::setup(app)?;
            join::setup(app);
            if app
                .state::<settings::SettingsState>()
                .remembers_window_state()
                && let Some(window) = app.get_webview_window("main")
            {
                window.restore_state(window_state_flags())?;
            }
            if let Err(error) = effects::set_material(
                app.handle(),
                app.state::<settings::SettingsState>()
                    .window_material()
                    .as_str(),
                app.state::<settings::SettingsState>().theme().as_str(),
            ) {
                eprintln!("failed to apply native window effects: {error}");
            }
            tray::setup(app)?;
            tray::show_main_window(app.handle());
            Ok(())
        })
        .on_window_event(tray::handle_window_event)
        .invoke_handler(tauri::generate_handler![
            connection::get_status,
            stop_tunnel,
            exit_application,
            host::start_lan_scan,
            host::get_lan_scan,
            host::restart_lan_scan,
            host::stop_lan_scan,
            host::probe_host_port,
            host::start_host,
            join::validate_invite,
            join::start_join,
            join::stop_join,
            settings::get_preferences,
            settings::get_system_fonts,
            effects::supports_liquid_glass,
            settings::set_theme,
            settings::set_color_theme,
            settings::set_locale,
            settings::set_close_action,
            settings::set_invite_lifetime,
            settings::set_join_port,
            settings::set_personalization,
            settings::set_connection_settings,
            settings::set_lightweight_settings,
            deeplink::take_pending_links,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| {
        if let tauri::RunEvent::ExitRequested {
            api, code: None, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}
