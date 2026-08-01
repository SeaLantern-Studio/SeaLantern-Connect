use tauri::App;

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(any(target_os = "linux", all(debug_assertions, target_os = "windows")))]
    {
        use tauri_plugin_deep_link::DeepLinkExt;

        app.deep_link().register_all()?;
    }

    Ok(())
}
