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

fn load_tray_icon() -> Image<'static> {
    Image::new_owned(build_tray_icon_rgba(), TRAY_ICON_SIZE, TRAY_ICON_SIZE)
}

const TRAY_ICON_SIZE: u32 = 32;

fn build_tray_icon_rgba() -> Vec<u8> {
    let mut rgba = vec![0; (TRAY_ICON_SIZE * TRAY_ICON_SIZE * 4) as usize];
    let top_color = [126, 91, 255];
    let bottom_color = [70, 123, 255];
    let white = [255, 255, 255];

    for y in 0..TRAY_ICON_SIZE {
        for x in 0..TRAY_ICON_SIZE {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let panel_alpha = rounded_rect_alpha(px, py, 4.0, 4.0, 28.0, 28.0, 7.0);

            if panel_alpha > 0.0 {
                let gradient = ((px * 0.35 + py * 0.65) / TRAY_ICON_SIZE as f32).clamp(0.0, 1.0);
                blend_pixel(
                    &mut rgba,
                    x,
                    y,
                    mix_color(top_color, bottom_color, gradient),
                    panel_alpha,
                );
            }

            let mark_alpha = c_mark_alpha(px, py) * panel_alpha;
            if mark_alpha > 0.0 {
                blend_pixel(&mut rgba, x, y, white, mark_alpha);
            }
        }
    }

    rgba
}

fn c_mark_alpha(x: f32, y: f32) -> f32 {
    let center_x = 17.0;
    let center_y = 16.0;
    let outer_radius = 8.9;
    let inner_radius = 5.1;
    let dx = x - center_x;
    let dy = y - center_y;
    let radius = (dx * dx + dy * dy).sqrt();
    let ring_distance = (radius - outer_radius).max(inner_radius - radius);
    let ring_alpha = alpha_from_distance(ring_distance);
    let angle = dy.atan2(dx).abs();
    let opening_alpha = ((angle - 0.62) / 0.18).clamp(0.0, 1.0);
    let c_body = ring_alpha * opening_alpha;
    let cap_radius = 2.0;
    let cap_distance = (outer_radius + inner_radius) * 0.5;
    let upper_cap = circle_alpha(
        x,
        y,
        center_x + cap_distance * 0.62_f32.cos(),
        center_y - cap_distance * 0.62_f32.sin(),
        cap_radius,
    );
    let lower_cap = circle_alpha(
        x,
        y,
        center_x + cap_distance * 0.62_f32.cos(),
        center_y + cap_distance * 0.62_f32.sin(),
        cap_radius,
    );

    c_body.max(upper_cap).max(lower_cap)
}

fn rounded_rect_alpha(
    x: f32,
    y: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    radius: f32,
) -> f32 {
    let center_x = (left + right) * 0.5;
    let center_y = (top + bottom) * 0.5;
    let half_width = (right - left) * 0.5 - radius;
    let half_height = (bottom - top) * 0.5 - radius;
    let qx = (x - center_x).abs() - half_width;
    let qy = (y - center_y).abs() - half_height;
    let outside_x = qx.max(0.0);
    let outside_y = qy.max(0.0);
    let outside = (outside_x * outside_x + outside_y * outside_y).sqrt();
    let inside = qx.max(qy).min(0.0);

    alpha_from_distance(outside + inside - radius)
}

fn circle_alpha(x: f32, y: f32, center_x: f32, center_y: f32, radius: f32) -> f32 {
    let dx = x - center_x;
    let dy = y - center_y;
    alpha_from_distance((dx * dx + dy * dy).sqrt() - radius)
}

fn alpha_from_distance(distance: f32) -> f32 {
    (0.5 - distance).clamp(0.0, 1.0)
}

fn mix_color(start: [u8; 3], end: [u8; 3], amount: f32) -> [u8; 3] {
    [
        mix_channel(start[0], end[0], amount),
        mix_channel(start[1], end[1], amount),
        mix_channel(start[2], end[2], amount),
    ]
}

fn mix_channel(start: u8, end: u8, amount: f32) -> u8 {
    (start as f32 + (end as f32 - start as f32) * amount).round() as u8
}

fn blend_pixel(rgba: &mut [u8], x: u32, y: u32, color: [u8; 3], alpha: f32) {
    let index = ((y * TRAY_ICON_SIZE + x) * 4) as usize;
    let src_alpha = alpha.clamp(0.0, 1.0);
    let dst_alpha = rgba[index + 3] as f32 / 255.0;
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);

    if out_alpha <= 0.0 {
        return;
    }

    for channel in 0..3 {
        let src = color[channel] as f32 / 255.0;
        let dst = rgba[index + channel] as f32 / 255.0;
        let out = (src * src_alpha + dst * dst_alpha * (1.0 - src_alpha)) / out_alpha;
        rgba[index + channel] = (out * 255.0).round() as u8;
    }
    rgba[index + 3] = (out_alpha * 255.0).round() as u8;
}

#[cfg(test)]
mod tests {
    use super::{build_tray_icon_rgba, load_tray_icon, TRAY_ICON_SIZE};

    #[test]
    fn tray_icon_is_not_a_single_solid_square() {
        let icon = load_tray_icon();
        let rgba = build_tray_icon_rgba();

        assert_eq!(icon.width(), TRAY_ICON_SIZE);
        assert_eq!(icon.height(), TRAY_ICON_SIZE);
        assert_eq!(rgba[3], 0);
        assert!(rgba.chunks_exact(4).any(|pixel| pixel[3] == 0));
        assert!(rgba.chunks_exact(4).any(|pixel| pixel[3] == 255));
    }
}
