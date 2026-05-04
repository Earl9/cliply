mod commands;
mod db;
mod error;
mod models;
mod platform;
mod services;
mod shortcuts;
mod tray;

use tauri::{Emitter, Manager};

struct ClipboardListenerShutdown;

impl Drop for ClipboardListenerShutdown {
    fn drop(&mut self) {
        let _ = platform::stop_clipboard_listener();
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::list_clipboard_items,
            commands::get_clipboard_item_detail,
            commands::toggle_pin_clipboard_item,
            commands::delete_clipboard_item,
            commands::clear_clipboard_history,
            commands::copy_clipboard_item,
            commands::paste_clipboard_item,
            commands::paste_plain_text,
            commands::initialize_storage,
            commands::get_cliply_settings,
            commands::update_cliply_settings,
            commands::set_monitoring_paused,
            commands::show_main_window,
            commands::hide_main_window,
            commands::toggle_main_window_pin
        ])
        .setup(|app| {
            db::initialize_storage(app.handle())?;
            let cleanup = services::clipboard_service::enforce_history_retention(app.handle())?;
            if cleanup.deleted_items > 0 {
                let _ = app.handle().emit("clipboard-items-changed", ());
            }
            tray::create_tray(app.handle())?;
            shortcuts::register_default_shortcuts(app.handle())?;
            platform::start_clipboard_listener(app.handle().clone())?;
            app.manage(ClipboardListenerShutdown);
            if let Some(window) = app.get_webview_window("main") {
                let handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = hide_main_window(&handle);
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Cliply");
}

pub fn show_main_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
    }

    Ok(())
}

pub fn hide_main_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide()?;
    }

    Ok(())
}

pub fn toggle_main_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible()? {
            window.hide()?;
        } else {
            window.show()?;
            window.unminimize()?;
            window.set_focus()?;
        }
    }

    Ok(())
}

pub fn toggle_main_window_pin(app: &tauri::AppHandle, pinned: bool) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(pinned)?;
    }

    Ok(())
}
