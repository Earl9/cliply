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
    message
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(600)
        .collect()
}
