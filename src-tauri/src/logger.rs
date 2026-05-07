use crate::error::CliplyError;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use time::OffsetDateTime;

const LOG_FILE_NAME: &str = "cliply.log";

pub fn log_path(app: &AppHandle) -> Result<PathBuf, CliplyError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;
    fs::create_dir_all(&app_data_dir)?;
    Ok(app_data_dir.join(LOG_FILE_NAME))
}

pub fn info(app: &AppHandle, event: &str, message: impl AsRef<str>) {
    write(app, "INFO", event, message.as_ref());
}

pub fn error(app: &AppHandle, event: &str, error: impl std::fmt::Display) {
    write(app, "ERROR", event, &error.to_string());
}

fn write(app: &AppHandle, level: &str, event: &str, message: &str) {
    let Ok(path) = log_path(app) else {
        return;
    };

    let timestamp = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown-time".to_string());
    let message = sanitize_message(message);
    let line = format!("{timestamp} {level} {event} {message}\n");

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}

fn sanitize_message(message: &str) -> String {
    let compact = message
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    let compact = if contains_sensitive_field(&compact) {
        "[redacted sensitive fields]".to_string()
    } else {
        redact_large_secret_like_tokens(&compact)
    };

    compact.chars().take(600).collect()
}

fn contains_sensitive_field(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    [
        "authorization",
        "bearer ",
        "password",
        "passwd",
        "private key",
        "private_key",
        "privatekey",
        "secret_access_key",
        "secretaccesskey",
        "access_token",
        "refresh_token",
        "\"data_text\"",
        "\"normalized_text\"",
        "encrypted_payload",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn redact_large_secret_like_tokens(message: &str) -> String {
    message
        .split_whitespace()
        .map(|token| {
            if is_probable_secret_blob(token) {
                "[redacted-large-token]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_probable_secret_blob(token: &str) -> bool {
    token.len() >= 96
        && token.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '/' | '=' | '-' | '_')
        })
}
