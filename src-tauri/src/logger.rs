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

pub(crate) fn sanitize_diagnostic_message(message: &str) -> String {
    sanitize_message(message)
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
        "data_text",
        "preview_text",
        "normalized_text",
        "payload_json",
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

#[cfg(test)]
mod tests {
    use super::sanitize_diagnostic_message;

    #[test]
    fn redacts_sensitive_field_messages() {
        for message in [
            "password=hunter2",
            "Authorization: Bearer secret-token",
            r#"{"data_text":"clipboard body"}"#,
            r#"{"preview_text":"clipboard preview"}"#,
            r#"{"normalized_text":"clipboard body"}"#,
            r#"{"payload_json":"event body"}"#,
            r#"{"encrypted_payload":"ciphertext"}"#,
            "-----BEGIN PRIVATE KEY-----",
        ] {
            assert_eq!(
                sanitize_diagnostic_message(message),
                "[redacted sensitive fields]"
            );
        }
    }

    #[test]
    fn redacts_large_secret_like_tokens() {
        let token = "A".repeat(120);
        assert_eq!(
            sanitize_diagnostic_message(&format!("upload failed token={token}")),
            "upload failed [redacted-large-token]"
        );
    }

    #[test]
    fn keeps_operational_messages() {
        assert_eq!(
            sanitize_diagnostic_message("exported=1 imported=2 skipped=0"),
            "exported=1 imported=2 skipped=0"
        );
    }
}
