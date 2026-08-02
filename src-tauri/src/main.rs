// Protocol activation starts a short-lived second process on Windows. Using the GUI
// subsystem in debug builds too prevents that process from flashing a console window.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod connection;
mod desktop;
mod logging;
mod settings;

use connection::{host, join};
use desktop::{autodelay, deeplink, effects, tray, window_state};
use tauri::Manager;
use tauri_plugin_window_state::{StateFlags, WindowExt};

#[cfg(all(target_os = "windows", debug_assertions))]
fn attach_parent_console() {
    use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};

    // The debug executable keeps the GUI subsystem to avoid protocol-activation flashes.
    // Attaching an existing parent console restores stdout without creating a new window.
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

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
    #[cfg(all(target_os = "windows", debug_assertions))]
    attach_parent_console();

    let app = tauri::Builder::default()
        .plugin(logging::plugin())
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
            let material = app.state::<settings::SettingsState>().window_material();
            let theme = app.state::<settings::SettingsState>().theme();
            log::info!("startup: {material}/{theme}");
            deeplink::setup(app)?;
            join::setup(app);
            if app
                .state::<settings::SettingsState>()
                .remembers_window_state()
                && let Some(window) = app.get_webview_window("main")
            {
                window.restore_state(window_state_flags())?;
            }
            if let Err(error) = effects::set_material(app.handle(), &material, &theme) {
                log::error!("native material failed: {error}");
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

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested {
            api, code: None, ..
        } => api.prevent_exit(),
        tauri::RunEvent::Exit => {
            log::info!("exit");
        }
        _ => {}
    });
}
