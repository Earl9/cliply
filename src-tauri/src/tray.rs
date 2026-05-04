use crate::services::settings_service;
use tauri::{
    image::Image,
    menu::{Menu, MenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter,
};

const TRAY_ID: &str = "cliply-main-tray";
const MENU_OPEN: &str = "open";
const MENU_TOGGLE_MONITORING: &str = "toggle_monitoring";
const MENU_CLEAR_HISTORY: &str = "clear_history";
const MENU_SETTINGS: &str = "settings";
const MENU_ABOUT: &str = "about";
const MENU_EXIT: &str = "exit";

pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_tray_menu(app)?;
    let icon = load_tray_icon();

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip(tray_tooltip(app))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = crate::show_main_window(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            match id {
                MENU_OPEN => {
                    let _ = crate::show_main_window(app);
                }
                MENU_TOGGLE_MONITORING => {
                    let paused = settings_service::is_monitoring_paused(app);
                    let _ = settings_service::set_monitoring_paused(app, !paused);
                    let _ = refresh_tray(app);
                }
                MENU_CLEAR_HISTORY => {
                    let _ = app.emit("cliply-open-clear-history", ());
                    let _ = crate::show_main_window(app);
                }
                MENU_SETTINGS => {
                    let _ = app.emit("cliply-open-settings", ());
                    let _ = crate::show_main_window(app);
                }
                MENU_ABOUT => {
                    let _ = app.emit("cliply-open-about", ());
                    let _ = crate::show_main_window(app);
                }
                MENU_EXIT => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

pub fn refresh_tray(app: &AppHandle) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(build_tray_menu(app)?))?;
        tray.set_tooltip(Some(tray_tooltip(app)))?;
    }

    Ok(())
}

fn build_tray_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let paused = settings_service::is_monitoring_paused(app);
    let monitoring_label = if paused {
        "恢复监听"
    } else {
        "暂停监听"
    };

    MenuBuilder::new(app)
        .text(MENU_OPEN, "打开 Cliply")
        .separator()
        .text(MENU_TOGGLE_MONITORING, monitoring_label)
        .text(MENU_CLEAR_HISTORY, "清空历史")
        .text(MENU_SETTINGS, "设置")
        .text(MENU_ABOUT, "关于 Cliply")
        .separator()
        .text(MENU_EXIT, "退出")
        .build()
}

fn tray_tooltip(app: &AppHandle) -> String {
    if settings_service::is_monitoring_paused(app) {
        "Cliply - 监听已暂停".to_string()
    } else {
        "Cliply - 本地保存".to_string()
    }
}

fn fallback_icon() -> Image<'static> {
    Image::new_owned(vec![115, 87, 246, 255], 1, 1)
}

fn load_tray_icon() -> Image<'static> {
    fallback_icon()
}
