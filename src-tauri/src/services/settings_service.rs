use crate::models::settings::CliplySettings;
use crate::{error::CliplyError, logger, platform, services::database_service, shortcuts};
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
const KEY_THEME_NAME: &str = "theme_name";
const KEY_ACCENT_COLOR: &str = "accent_color";
const KEY_AUTO_THEME: &str = "auto_theme";
const KEY_IMAGE_SYNC: &str = "image_sync";
const KEY_UPDATE: &str = "update";

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
    if previous_settings.global_shortcut != settings.global_shortcut {
        if let Err(error) = shortcuts::register_user_shortcut(app, &settings.global_shortcut) {
            let _ = shortcuts::register_user_shortcut(app, &previous_settings.global_shortcut);
            return Err(error);
        }
    }

    if previous_settings.launch_at_startup != settings.launch_at_startup
        || previous_settings.start_minimized != settings.start_minimized
    {
        platform::set_launch_at_startup(settings.launch_at_startup, settings.start_minimized)?;
        log_startup_setting_synced(app, &settings, "settings_changed");
    } else if settings.launch_at_startup {
        match platform::set_launch_at_startup(settings.launch_at_startup, settings.start_minimized)
        {
            Ok(()) => log_startup_setting_synced(app, &settings, "settings_repaired"),
            Err(error) => logger::error(app, "startup_setting_repair_failed", error),
        }
    }
    save_settings(&connection, &settings)?;
    let _ = app.emit("cliply-settings-changed", &settings);
    Ok(settings)
}

pub fn reconcile_startup_setting(app: &AppHandle) {
    let settings = match get_settings(app) {
        Ok(settings) => settings,
        Err(error) => {
            logger::error(app, "startup_setting_reconcile_failed", error);
            return;
        }
    };

    match platform::set_launch_at_startup(settings.launch_at_startup, settings.start_minimized) {
        Ok(()) => log_startup_setting_synced(app, &settings, "app_start"),
        Err(error) => logger::error(app, "startup_setting_reconcile_failed", error),
    }
}

fn log_startup_setting_synced(app: &AppHandle, settings: &CliplySettings, source: &str) {
    logger::info(
        app,
        "startup_setting",
        format!(
            "source={} launch_at_startup={} start_minimized={}",
            source, settings.launch_at_startup, settings.start_minimized
        ),
    );
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
    let stored_launch_at_startup = get_value(connection, KEY_LAUNCH_AT_STARTUP)?;
    let stored_start_minimized = get_value(connection, KEY_START_MINIMIZED)?;
    let registry_start_minimized = if stored_launch_at_startup.is_none() {
        platform::read_launch_at_startup().ok().flatten()
    } else {
        None
    };

    Ok(CliplySettings {
        max_history_items: get_value(connection, KEY_MAX_HISTORY_ITEMS)?
            .unwrap_or(default.max_history_items),
        auto_delete_days: get_value(connection, KEY_AUTO_DELETE_DAYS)?
            .unwrap_or(default.auto_delete_days),
        pause_monitoring: get_value(connection, KEY_PAUSE_MONITORING)?
            .unwrap_or(default.pause_monitoring),
        launch_at_startup: stored_launch_at_startup
            .or_else(|| registry_start_minimized.map(|_| true))
            .unwrap_or(default.launch_at_startup),
        start_minimized: stored_start_minimized
            .or(registry_start_minimized)
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
        theme_name: get_value(connection, KEY_THEME_NAME)?.unwrap_or(default.theme_name),
        accent_color: get_value(connection, KEY_ACCENT_COLOR)?.unwrap_or(default.accent_color),
        auto_theme: get_value(connection, KEY_AUTO_THEME)?.unwrap_or(default.auto_theme),
        image_sync: get_value(connection, KEY_IMAGE_SYNC)?.unwrap_or(default.image_sync),
        update: get_value(connection, KEY_UPDATE)?.unwrap_or(default.update),
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
    set_value(connection, KEY_THEME_NAME, &settings.theme_name)?;
    set_value(connection, KEY_ACCENT_COLOR, &settings.accent_color)?;
    set_value(connection, KEY_AUTO_THEME, &settings.auto_theme)?;
    set_value(connection, KEY_IMAGE_SYNC, &settings.image_sync)?;
    set_value(connection, KEY_UPDATE, &settings.update)?;
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
