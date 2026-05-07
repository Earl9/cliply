mod commands;
mod db;
mod error;
mod logger;
mod models;
mod platform;
mod services;
mod shortcuts;
mod tray;

use std::time::Instant;
use tauri::{Emitter, Manager};

struct ClipboardListenerShutdown(tauri::AppHandle);

impl Drop for ClipboardListenerShutdown {
    fn drop(&mut self) {
        let _ = platform::stop_clipboard_listener();
        logger::info(&self.0, "app_exit", "clipboard listener stopped");
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::list_clipboard_items,
            commands::get_clipboard_item_detail,
            commands::toggle_pin_clipboard_item,
            commands::delete_clipboard_item,
            commands::clear_clipboard_history,
            commands::export_sync_package,
            commands::import_sync_package,
            commands::get_sync_package_status,
            commands::get_remote_sync_status,
            commands::set_remote_sync_provider,
            commands::update_auto_sync_config,
            commands::clear_auto_sync_password,
            commands::sync_with_remote_now,
            commands::export_to_remote_sync_folder,
            commands::import_from_remote_sync_folder,
            commands::copy_clipboard_item,
            commands::paste_clipboard_item,
            commands::paste_plain_text,
            commands::initialize_storage,
            commands::get_debug_info,
            commands::open_log_directory,
            commands::get_system_theme_colors,
            commands::get_cliply_settings,
            commands::update_cliply_settings,
            commands::check_global_shortcut,
            commands::set_monitoring_paused,
            commands::show_main_window,
            commands::hide_main_window,
            commands::minimize_main_window,
            commands::toggle_main_window_pin
        ])
        .setup(|app| {
            logger::info(app.handle(), "app_start", "Cliply setup started");
            db::initialize_storage(app.handle())?;
            logger::info(app.handle(), "storage_initialized", "SQLite storage ready");
            let cleanup = services::clipboard_service::enforce_history_retention(app.handle())?;
            if cleanup.deleted_items > 0 {
                let _ = app.handle().emit("clipboard-items-changed", ());
                logger::info(
                    app.handle(),
                    "history_retention",
                    format!("deleted_items={}", cleanup.deleted_items),
                );
            }
            tray::create_tray(app.handle())?;
            shortcuts::register_default_shortcuts(app.handle())?;
            platform::start_clipboard_listener(app.handle().clone())?;
            logger::info(app.handle(), "clipboard_listener_started", "listener ready");
            app.manage(ClipboardListenerShutdown(app.handle().clone()));
            let auto_sync_shutdown =
                services::sync_scheduler_service::start_auto_sync_scheduler(app.handle().clone())?;
            app.manage(auto_sync_shutdown);
            logger::info(
                app.handle(),
                "auto_sync_scheduler_started",
                "scheduler ready",
            );
            if let Some(window) = app.get_webview_window("main") {
                set_main_window_icon(app.handle(), &window);

                let handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        logger::info(&handle, "window_close_requested", "hide instead of exit");
                        let _ = hide_main_window(&handle);
                    }
                });

                if should_start_minimized() {
                    window.hide()?;
                    logger::info(app.handle(), "startup_window", "startup_arg_minimized=true");
                } else {
                    show_main_window(app.handle())?;
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Cliply");
}

fn should_start_minimized() -> bool {
    std::env::args().any(|arg| arg == "--minimized" || arg == "--start-minimized")
}

pub fn show_main_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        let started_at = Instant::now();
        set_main_window_icon(app, &window);
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
        logger::info(
            app,
            "window_show",
            format!("duration_ms={}", started_at.elapsed().as_millis()),
        );
    }

    Ok(())
}

pub fn hide_main_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide()?;
    }

    Ok(())
}

pub fn minimize_main_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.minimize()?;
        logger::info(app, "window_minimize", "main window minimized");
    }

    Ok(())
}

pub fn toggle_main_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible()? {
            window.hide()?;
        } else {
            let started_at = Instant::now();
            set_main_window_icon(app, &window);
            window.show()?;
            window.unminimize()?;
            window.set_focus()?;
            logger::info(
                app,
                "window_show",
                format!("duration_ms={}", started_at.elapsed().as_millis()),
            );
        }
    }

    Ok(())
}

fn set_main_window_icon(app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    let Some(icon) = load_window_icon() else {
        logger::error(
            app,
            "window_icon_load_failed",
            "failed to decode bundled Cliply window icon",
        );
        return;
    };

    if let Err(error) = window.set_icon(icon) {
        logger::error(app, "window_icon_set_failed", error);
    }
}

fn load_window_icon() -> Option<tauri::image::Image<'static>> {
    let icon = image::load_from_memory(include_bytes!("../icons/128x128@2x.png"))
        .ok()?
        .into_rgba8();
    let (width, height) = icon.dimensions();

    Some(tauri::image::Image::new_owned(
        icon.into_raw(),
        width,
        height,
    ))
}

pub fn toggle_main_window_pin(app: &tauri::AppHandle, pinned: bool) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(pinned)?;
    }

    Ok(())
}
