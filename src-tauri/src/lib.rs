mod commands;
mod db;
mod error;
mod models;
mod platform;
mod services;
mod shortcuts;
mod tray;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::list_clipboard_items,
            commands::get_clipboard_item_detail,
            commands::toggle_pin_clipboard_item,
            commands::initialize_storage
        ])
        .setup(|app| {
            db::initialize_storage(app.handle())?;
            tray::create_tray(app.handle())?;
            shortcuts::register_default_shortcuts(app.handle())?;
            platform::start_clipboard_listener(app.handle().clone())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Cliply");
}
