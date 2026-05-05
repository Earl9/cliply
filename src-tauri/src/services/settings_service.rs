use crate::models::settings::CliplySettings;
use crate::{error::CliplyError, logger, platform, services::database_service};
use rusqlite::{params, Connection};
use serde::{de::DeserializeOwned, Serialize};
use tauri::{AppHandle, Emitter};
use time::OffsetDateTime;

const KEY_MAX_HISTORY_ITEMS: &str = "max_history_items";
const KEY_AUTO_DELETE_DAYS: &str = "auto_delete_days";
const KEY_PAUSE_MONITORING: &str = "pause_monitoring";
const KEY_LAUNCH_AT_STARTUP: &str = "launch_at_startup";
const KEY_START_MINIMIZED: &str = "start_minimized";
const KEY_FOCUS_SEARCH_ON_OPEN: &str = "focus_search_on_open";
const KEY_CLOSE_AFTER_PASTE: &str = "close_after_paste";
const KEY_IGNORE_DUPLICATE: &str = "ignore_duplicate";
const KEY_SAVE_IMAGES: &str = "save_images";
const KEY_SAVE_HTML: &str = "save_html";
const KEY_SAVE_SENSITIVE: &str = "save_sensitive";
const KEY_IGNORE_APPS: &str = "ignore_apps";
const KEY_GLOBAL_SHORTCUT: &str = "global_shortcut";
const KEY_THEME: &str = "theme";

pub fn default_settings() -> CliplySettings {
    CliplySettings::default()
}

pub fn get_settings(app: &AppHandle) -> Result<CliplySettings, CliplyError> {
    let connection = database_service::connect(app)?;
    Ok(load_settings(&connection)?)
}

pub fn update_settings(
    app: &AppHandle,
    settings: CliplySettings,
) -> Result<CliplySettings, CliplyError> {
    let connection = database_service::connect(app)?;
    let previous_settings = load_settings(&connection).unwrap_or_else(|_| default_settings());
    if previous_settings.launch_at_startup != settings.launch_at_startup {
        platform::set_launch_at_startup(settings.launch_at_startup)?;
        logger::info(
            app,
            "startup_setting",
            format!("launch_at_startup={}", settings.launch_at_startup),
        );
    }
    save_settings(&connection, &settings)?;
    let _ = app.emit("cliply-settings-changed", &settings);
    Ok(settings)
}

pub fn set_monitoring_paused(app: &AppHandle, paused: bool) -> Result<CliplySettings, CliplyError> {
    let mut settings = get_settings(app)?;
    settings.pause_monitoring = paused;
    update_settings(app, settings)
}

pub fn is_monitoring_paused(app: &AppHandle) -> bool {
    get_settings(app)
        .map(|settings| settings.pause_monitoring)
        .unwrap_or(false)
}

fn load_settings(connection: &Connection) -> Result<CliplySettings, CliplyError> {
    let default = default_settings();
    Ok(CliplySettings {
        max_history_items: get_value(connection, KEY_MAX_HISTORY_ITEMS)?
            .unwrap_or(default.max_history_items),
        auto_delete_days: get_value(connection, KEY_AUTO_DELETE_DAYS)?
            .unwrap_or(default.auto_delete_days),
        pause_monitoring: get_value(connection, KEY_PAUSE_MONITORING)?
            .unwrap_or(default.pause_monitoring),
        launch_at_startup: get_value(connection, KEY_LAUNCH_AT_STARTUP)?
            .unwrap_or(default.launch_at_startup),
        start_minimized: get_value(connection, KEY_START_MINIMIZED)?
            .unwrap_or(default.start_minimized),
        focus_search_on_open: get_value(connection, KEY_FOCUS_SEARCH_ON_OPEN)?
            .unwrap_or(default.focus_search_on_open),
        close_after_paste: get_value(connection, KEY_CLOSE_AFTER_PASTE)?
            .unwrap_or(default.close_after_paste),
        ignore_duplicate: get_value(connection, KEY_IGNORE_DUPLICATE)?
            .unwrap_or(default.ignore_duplicate),
        save_images: get_value(connection, KEY_SAVE_IMAGES)?.unwrap_or(default.save_images),
        save_html: get_value(connection, KEY_SAVE_HTML)?.unwrap_or(default.save_html),
        save_sensitive: get_value(connection, KEY_SAVE_SENSITIVE)?
            .unwrap_or(default.save_sensitive),
        ignore_apps: get_value(connection, KEY_IGNORE_APPS)?.unwrap_or(default.ignore_apps),
        global_shortcut: get_value(connection, KEY_GLOBAL_SHORTCUT)?
            .unwrap_or(default.global_shortcut),
        theme: get_value(connection, KEY_THEME)?.unwrap_or(default.theme),
    })
}

fn save_settings(connection: &Connection, settings: &CliplySettings) -> Result<(), CliplyError> {
    set_value(
        connection,
        KEY_MAX_HISTORY_ITEMS,
        settings.max_history_items,
    )?;
    set_value(connection, KEY_AUTO_DELETE_DAYS, settings.auto_delete_days)?;
    set_value(connection, KEY_PAUSE_MONITORING, settings.pause_monitoring)?;
    set_value(
        connection,
        KEY_LAUNCH_AT_STARTUP,
        settings.launch_at_startup,
    )?;
    set_value(connection, KEY_START_MINIMIZED, settings.start_minimized)?;
    set_value(
        connection,
        KEY_FOCUS_SEARCH_ON_OPEN,
        settings.focus_search_on_open,
    )?;
    set_value(
        connection,
        KEY_CLOSE_AFTER_PASTE,
        settings.close_after_paste,
    )?;
    set_value(connection, KEY_IGNORE_DUPLICATE, settings.ignore_duplicate)?;
    set_value(connection, KEY_SAVE_IMAGES, settings.save_images)?;
    set_value(connection, KEY_SAVE_HTML, settings.save_html)?;
    set_value(connection, KEY_SAVE_SENSITIVE, settings.save_sensitive)?;
    set_value(connection, KEY_IGNORE_APPS, &settings.ignore_apps)?;
    set_value(connection, KEY_GLOBAL_SHORTCUT, &settings.global_shortcut)?;
    set_value(connection, KEY_THEME, &settings.theme)?;
    Ok(())
}

fn get_value<T: DeserializeOwned>(
    connection: &Connection,
    key: &str,
) -> Result<Option<T>, CliplyError> {
    let result = connection.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(value) => serde_json::from_str(&value)
            .map(Some)
            .map_err(|error| CliplyError::StorageUnavailable(error.to_string())),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn set_value<T: Serialize>(
    connection: &Connection,
    key: &str,
    value: T,
) -> Result<(), CliplyError> {
    let now = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;
    let value = serde_json::to_string(&value)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;

    connection.execute(
        "INSERT INTO settings (key, value, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET
           value = excluded.value,
           updated_at = excluded.updated_at",
        params![key, value, now],
    )?;

    Ok(())
}
