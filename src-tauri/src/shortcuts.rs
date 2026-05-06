use crate::error::CliplyError;
use crate::services::settings_service;
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutCheck {
    pub ok: bool,
    pub normalized: String,
    pub display: String,
    pub reason: String,
    pub message: String,
}

#[derive(Debug, Clone)]
struct NormalizedShortcut {
    normalized: String,
    display: String,
}

pub fn register_default_shortcuts(app: &AppHandle) -> tauri::Result<()> {
    let settings = settings_service::get_settings(app)
        .unwrap_or_else(|_| settings_service::default_settings());
    register_shortcut(app, &settings.global_shortcut)
}

pub fn validate_shortcut(shortcut: &str) -> tauri::Result<()> {
    let normalized = normalize_shortcut(shortcut).map_err(shortcut_validation_error)?;
    normalized
        .normalized
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map(|_| ())
        .map_err(|error| tauri::Error::Anyhow(error.into()))
}

pub fn check_shortcut(
    app: &AppHandle,
    shortcut: &str,
    current_shortcut: Option<&str>,
) -> ShortcutCheck {
    let normalized = match normalize_shortcut(shortcut) {
        Ok(normalized) => normalized,
        Err(message) => {
            return ShortcutCheck {
                ok: false,
                normalized: String::new(),
                display: shortcut.trim().to_string(),
                reason: "invalid".to_string(),
                message,
            };
        }
    };

    if let Err(error) = normalized
        .normalized
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
    {
        return ShortcutCheck {
            ok: false,
            normalized: normalized.normalized,
            display: normalized.display,
            reason: "invalid".to_string(),
            message: format!("快捷键格式不可用：{error}"),
        };
    }

    let current_normalized = current_shortcut.and_then(|value| normalize_shortcut(value).ok());
    if current_normalized
        .as_ref()
        .is_some_and(|current| current.normalized == normalized.normalized)
    {
        return ShortcutCheck {
            ok: true,
            normalized: normalized.normalized,
            display: normalized.display,
            reason: "current".to_string(),
            message: "当前快捷键可用".to_string(),
        };
    }

    let global_shortcut = app.global_shortcut();
    if global_shortcut.is_registered(normalized.normalized.as_str()) {
        return ShortcutCheck {
            ok: false,
            normalized: normalized.normalized,
            display: normalized.display,
            reason: "cliply-conflict".to_string(),
            message: "该快捷键已被 Cliply 使用，请换一个组合键".to_string(),
        };
    }

    match global_shortcut.on_shortcut(normalized.normalized.as_str(), |_app, _shortcut, _event| {})
    {
        Ok(()) => {
            let _ = global_shortcut.unregister(normalized.normalized.as_str());
            ShortcutCheck {
                ok: true,
                normalized: normalized.normalized,
                display: normalized.display,
                reason: "available".to_string(),
                message: "快捷键可用".to_string(),
            }
        }
        Err(error) => ShortcutCheck {
            ok: false,
            normalized: normalized.normalized,
            display: normalized.display,
            reason: "system-conflict".to_string(),
            message: format!("快捷键已被系统或其他应用占用：{error}"),
        },
    }
}

pub fn register_user_shortcut(app: &AppHandle, shortcut: &str) -> Result<(), CliplyError> {
    register_shortcut(app, shortcut).map_err(|error| {
        CliplyError::PlatformUnavailable(format!(
            "快捷键已被占用或不可用，请换一个组合键（{error}）"
        ))
    })
}

fn register_shortcut(app: &AppHandle, shortcut: &str) -> tauri::Result<()> {
    let shortcut = normalize_shortcut(shortcut)
        .map_err(shortcut_validation_error)?
        .normalized;
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

fn normalize_shortcut(shortcut: &str) -> Result<NormalizedShortcut, String> {
    let raw_parts = shortcut
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if raw_parts.is_empty() {
        return Err("请先按下一个快捷键组合".to_string());
    }

    let mut has_control = false;
    let mut has_alt = false;
    let mut has_shift = false;
    let mut has_super = false;
    let mut key: Option<(String, String)> = None;

    for part in raw_parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => has_control = true,
            "shift" => has_shift = true,
            "alt" | "option" => has_alt = true,
            "win" | "cmd" | "command" | "meta" | "super" => has_super = true,
            _ => {
                if key.is_some() {
                    return Err("快捷键只能包含一个主按键".to_string());
                }
                key = Some(normalize_key(part)?);
            }
        }
    }

    if !has_control && !has_alt && !has_super {
        return Err("全局快捷键需要包含 Ctrl、Alt 或 Win，避免误触".to_string());
    }

    let Some((key_normalized, key_display)) = key else {
        return Err("请按下一个非修饰键作为主按键".to_string());
    };

    let mut normalized_parts = Vec::new();
    let mut display_parts = Vec::new();
    if has_control {
        normalized_parts.push("Control".to_string());
        display_parts.push("Ctrl".to_string());
    }
    if has_alt {
        normalized_parts.push("Alt".to_string());
        display_parts.push("Alt".to_string());
    }
    if has_shift {
        normalized_parts.push("Shift".to_string());
        display_parts.push("Shift".to_string());
    }
    if has_super {
        normalized_parts.push("Super".to_string());
        display_parts.push("Win".to_string());
    }
    normalized_parts.push(key_normalized);
    display_parts.push(key_display);

    Ok(NormalizedShortcut {
        normalized: normalized_parts.join("+"),
        display: display_parts.join("+"),
    })
}

fn normalize_key(key: &str) -> Result<(String, String), String> {
    let lower = key.to_lowercase();
    let normalized = match lower.as_str() {
        "esc" | "escape" => ("Escape".to_string(), "Escape".to_string()),
        "space" | "spacebar" => ("Space".to_string(), "Space".to_string()),
        "del" | "delete" => ("Delete".to_string(), "Delete".to_string()),
        "backspace" => ("Backspace".to_string(), "Backspace".to_string()),
        "enter" | "return" => ("Enter".to_string(), "Enter".to_string()),
        "tab" => ("Tab".to_string(), "Tab".to_string()),
        "insert" | "ins" => ("Insert".to_string(), "Insert".to_string()),
        "home" => ("Home".to_string(), "Home".to_string()),
        "end" => ("End".to_string(), "End".to_string()),
        "pageup" | "page up" => ("PageUp".to_string(), "PageUp".to_string()),
        "pagedown" | "page down" => ("PageDown".to_string(), "PageDown".to_string()),
        "arrowup" | "up" => ("ArrowUp".to_string(), "ArrowUp".to_string()),
        "arrowdown" | "down" => ("ArrowDown".to_string(), "ArrowDown".to_string()),
        "arrowleft" | "left" => ("ArrowLeft".to_string(), "ArrowLeft".to_string()),
        "arrowright" | "right" => ("ArrowRight".to_string(), "ArrowRight".to_string()),
        "`" | "backquote" => ("Backquote".to_string(), "`".to_string()),
        "-" | "minus" => ("Minus".to_string(), "-".to_string()),
        "=" | "equal" | "equals" => ("Equal".to_string(), "=".to_string()),
        "," | "comma" => ("Comma".to_string(), ",".to_string()),
        "." | "period" => ("Period".to_string(), ".".to_string()),
        "/" | "slash" => ("Slash".to_string(), "/".to_string()),
        "\\" | "backslash" => ("Backslash".to_string(), "\\".to_string()),
        ";" | "semicolon" => ("Semicolon".to_string(), ";".to_string()),
        "'" | "quote" => ("Quote".to_string(), "'".to_string()),
        "[" | "bracketleft" => ("BracketLeft".to_string(), "[".to_string()),
        "]" | "bracketright" => ("BracketRight".to_string(), "]".to_string()),
        value if is_function_key(value) => {
            let display = value.to_uppercase();
            (display.clone(), display)
        }
        value if value.len() == 1 && value.chars().all(|c| c.is_ascii_alphabetic()) => {
            let display = value.to_uppercase();
            (format!("Key{display}"), display)
        }
        value if value.len() == 1 && value.chars().all(|c| c.is_ascii_digit()) => {
            (format!("Digit{value}"), value.to_string())
        }
        value if value.starts_with("key") && value.len() == 4 => {
            let suffix = value[3..].to_uppercase();
            (format!("Key{suffix}"), suffix)
        }
        value if value.starts_with("digit") && value.len() == 6 => {
            let suffix = &value[5..];
            (format!("Digit{suffix}"), suffix.to_string())
        }
        _ => return Err(format!("无法识别主按键：{key}")),
    };

    Ok(normalized)
}

fn is_function_key(value: &str) -> bool {
    value
        .strip_prefix('f')
        .and_then(|number| number.parse::<u8>().ok())
        .is_some_and(|number| (1..=24).contains(&number))
}

fn shortcut_validation_error(message: String) -> tauri::Error {
    tauri::Error::Anyhow(std::io::Error::new(std::io::ErrorKind::InvalidInput, message).into())
}
