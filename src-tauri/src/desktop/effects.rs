#[cfg(any(target_os = "windows", target_os = "macos"))]
use super::window_state::MAIN_WINDOW_LABEL;
use tauri::AppHandle;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use tauri::Manager;

pub(crate) fn set_material(app: &AppHandle, material: &str, theme: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use tauri::{
            Theme,
            window::{Color, Effect, EffectsBuilder},
        };

        let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
            return Ok(());
        };

        window
            .set_theme(match theme {
                "dark" => Some(Theme::Dark),
                "light" => Some(Theme::Light),
                _ => None,
            })
            .map_err(|error| error.to_string())?;

        let effects = match material {
            "mica" => Some(
                EffectsBuilder::new()
                    .effect(match theme {
                        "dark" => Effect::MicaDark,
                        "light" => Effect::MicaLight,
                        _ => Effect::Mica,
                    })
                    .build(),
            ),
            "acrylic" => Some(
                EffectsBuilder::new()
                    .effect(Effect::Acrylic)
                    .color(match theme {
                        "dark" => Color(32, 32, 32, 225),
                        "light" => Color(245, 245, 245, 215),
                        _ => Color(0, 0, 0, 0),
                    })
                    .build(),
            ),
            _ => None,
        };
        window
            .set_effects(effects)
            .map_err(|error| error.to_string())
    }

    #[cfg(target_os = "macos")]
    {
        use tauri::{
            Theme,
            window::{Effect, EffectsBuilder},
        };

        let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
            return Ok(());
        };
        window
            .set_theme(match theme {
                "dark" => Some(Theme::Dark),
                "light" => Some(Theme::Light),
                _ => None,
            })
            .map_err(|error| error.to_string())?;
        let effects = (material == "vibrancy").then(|| {
            EffectsBuilder::new()
                .effect(Effect::UnderWindowBackground)
                .build()
        });
        window
            .set_effects(effects)
            .map_err(|error| error.to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (app, material, theme);
        Ok(())
    }
}
