use crate::services::settings_service;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub fn register_default_shortcuts(_app: &AppHandle) -> tauri::Result<()> {
    let settings = settings_service::get_settings(_app)
        .unwrap_or_else(|_| settings_service::default_settings());
    register_shortcut(_app, &settings.global_shortcut)
}

pub fn validate_shortcut(shortcut: &str) -> tauri::Result<()> {
    let normalized = normalize_shortcut(shortcut);
    normalized
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map(|_| ())
        .map_err(|error| tauri::Error::Anyhow(error.into()))
}

fn register_shortcut(app: &AppHandle, shortcut: &str) -> tauri::Result<()> {
    let shortcut = normalize_shortcut(shortcut);
    let global_shortcut = app.global_shortcut();
    let _ = global_shortcut.unregister_all();

    global_shortcut
        .on_shortcut(shortcut.as_str(), move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let _ = crate::toggle_main_window(app);
            }
        })
        .map_err(|error| tauri::Error::Anyhow(error.into()))?;

    Ok(())
}

fn normalize_shortcut(shortcut: &str) -> String {
    shortcut
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| match part.to_lowercase().as_str() {
            "ctrl" | "control" => "Control".to_string(),
            "shift" => "Shift".to_string(),
            "alt" | "option" => "Alt".to_string(),
            "cmd" | "command" | "super" => "Super".to_string(),
            single if single.len() == 1 && single.chars().all(|c| c.is_ascii_alphabetic()) => {
                format!("Key{}", single.to_uppercase())
            }
            single if single.len() == 1 && single.chars().all(|c| c.is_ascii_digit()) => {
                format!("Digit{single}")
            }
            value => value.to_string(),
        })
        .collect::<Vec<_>>()
        .join("+")
}
