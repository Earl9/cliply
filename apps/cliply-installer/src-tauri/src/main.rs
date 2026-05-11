#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod installer;
mod payload;
mod platform;
mod webview2;

fn main() {
    webview2::ensure_runtime_or_exit();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::detect_mode,
            commands::detect_installation,
            commands::browse_install_dir,
            commands::run_install,
            commands::run_uninstall,
            commands::launch_cliply,
            commands::open_installer_log_directory,
            commands::open_release_page
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Cliply modern installer");
}
