use crate::error::CliplyError;
use crate::models::clipboard_item::ClipboardItemType;
use crate::platform::{ClipboardFormatSnapshot, ClipboardSnapshot, ImageSnapshot};
use crate::services::content_detector;
use std::ffi::c_void;
use std::slice;
use windows_sys::Win32::Foundation::HGLOBAL;
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EnumClipboardFormats, GetClipboardData, GetClipboardFormatNameW,
    IsClipboardFormatAvailable, OpenClipboard,
};
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

const CF_TEXT: u32 = 1;
const CF_BITMAP: u32 = 2;
const CF_DIB: u32 = 8;
const CF_DIBV5: u32 = 17;
const CF_UNICODETEXT: u32 = 13;
const BI_BITFIELDS: u32 = 3;

pub fn read_current_snapshot() -> Result<Option<ClipboardSnapshot>, CliplyError> {
    let _clipboard = ClipboardGuard::open()?;
    let formats = read_available_formats();
    let text = read_unicode_text()?;
    let foreground_app = crate::platform::get_foreground_app();

    if let Some(text) = text {
        if text.trim().is_empty() {
            return Ok(None);
        }

        let primary_type = content_detector::detect_text_type(&text);
        return Ok(Some(ClipboardSnapshot {
            primary_type,
            text: Some(text),
            html: None,
            image: None,
            formats,
            source_app: foreground_app
                .as_ref()
                .map(|app| app.app_name.clone())
                .or_else(|| Some("Windows Clipboard".into())),
            source_window: foreground_app.and_then(|app| app.window_title),
        }));
    }

    if is_format_available(CF_BITMAP)
        || is_format_available(CF_DIB)
        || is_format_available(CF_DIBV5)
    {
        let image = read_dib_image()?;
        let foreground_app = crate::platform::get_foreground_app();
        return Ok(Some(ClipboardSnapshot {
            primary_type: ClipboardItemType::Image,
            text: None,
            html: None,
            image,
            formats,
            source_app: foreground_app
                .as_ref()
                .map(|app| app.app_name.clone())
                .or_else(|| Some("Windows Clipboard".into())),
            source_window: foreground_app.and_then(|app| app.window_title),
        }));
    }

    Ok(None)
}

fn read_unicode_text() -> Result<Option<String>, CliplyError> {
    if !is_format_available(CF_UNICODETEXT) {
        return Ok(None);
    }

    let handle = unsafe { GetClipboardData(CF_UNICODETEXT) };
    if handle.is_null() {
        return Ok(None);
    }

    let global = handle as HGLOBAL;
    let locked = unsafe { GlobalLock(global) } as *const u16;
    if locked.is_null() {
        return Ok(None);
    }

    let byte_len = unsafe { GlobalSize(global) };
    let max_units = byte_len / std::mem::size_of::<u16>();
    let units = unsafe { slice::from_raw_parts(locked, max_units) };
    let len = units
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(max_units);
    let value = String::from_utf16_lossy(&units[..len]);

    unsafe {
        GlobalUnlock(global);
    }

    Ok(Some(value))
}

fn read_dib_image() -> Result<Option<ImageSnapshot>, CliplyError> {
    let format = if is_format_available(CF_DIBV5) {
        CF_DIBV5
    } else if is_format_available(CF_DIB) {
        CF_DIB
    } else {
        return Ok(None);
    };

    let handle = unsafe { GetClipboardData(format) };
    if handle.is_null() {
        return Ok(None);
    }

    let global = handle as HGLOBAL;
    let locked = unsafe { GlobalLock(global) } as *const u8;
    if locked.is_null() {
        return Ok(None);
    }

    let byte_len = unsafe { GlobalSize(global) };
    let bytes = unsafe { slice::from_raw_parts(locked, byte_len) };
    let image = dib_to_bmp_snapshot(bytes);

    unsafe {
        GlobalUnlock(global);
    }

    image
}

fn dib_to_bmp_snapshot(dib: &[u8]) -> Result<Option<ImageSnapshot>, CliplyError> {
    if dib.len() < 40 {
        return Ok(None);
    }

    let width = read_i32_le(dib, 4).unsigned_abs();
    let height = read_i32_le(dib, 8).unsigned_abs();
    let pixel_offset = dib_pixel_offset(dib)?;
    let file_size = 14usize
        .checked_add(dib.len())
        .ok_or_else(|| CliplyError::PlatformUnavailable("clipboard image is too large".into()))?;
    let pixel_data_offset = 14usize.checked_add(pixel_offset).ok_or_else(|| {
        CliplyError::PlatformUnavailable("clipboard image header is invalid".into())
    })?;

    let mut bmp = Vec::with_capacity(file_size);
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(file_size as u32).to_le_bytes());
    bmp.extend_from_slice(&[0, 0, 0, 0]);
    bmp.extend_from_slice(&(pixel_data_offset as u32).to_le_bytes());
    bmp.extend_from_slice(dib);

    Ok(Some(ImageSnapshot {
        width,
        height,
        bytes: bmp,
        mime_type: "image/bmp".into(),
        extension: "bmp".into(),
    }))
}

fn dib_pixel_offset(dib: &[u8]) -> Result<usize, CliplyError> {
    let header_size = read_u32_le(dib, 0) as usize;
    if header_size == 0 || header_size > dib.len() {
        return Err(CliplyError::PlatformUnavailable(
            "clipboard image header is invalid".into(),
        ));
    }

    let bit_count = read_u16_le(dib, 14);
    let compression = read_u32_le(dib, 16);
    let color_used = read_u32_le(dib, 32) as usize;
    let mask_bytes = if compression == BI_BITFIELDS && header_size == 40 {
        12
    } else {
        0
    };
    let colors = if color_used > 0 {
        color_used
    } else if bit_count <= 8 {
        1usize << bit_count
    } else {
        0
    };

    header_size
        .checked_add(mask_bytes)
        .and_then(|offset| offset.checked_add(colors.saturating_mul(4)))
        .ok_or_else(|| CliplyError::PlatformUnavailable("clipboard image header is invalid".into()))
}

fn read_u16_le(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32_le(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_i32_le(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_available_formats() -> Vec<ClipboardFormatSnapshot> {
    let mut formats = Vec::new();
    let mut current = 0;

    loop {
        current = unsafe { EnumClipboardFormats(current) };
        if current == 0 {
            break;
        }

        formats.push(ClipboardFormatSnapshot {
            format_name: format_name(current),
            mime_type: mime_type(current),
            data_kind: data_kind(current).into(),
        });
    }

    formats
}

fn format_name(format: u32) -> String {
    match format {
        CF_TEXT => "CF_TEXT".into(),
        CF_BITMAP => "CF_BITMAP".into(),
        CF_DIB => "CF_DIB".into(),
        CF_DIBV5 => "CF_DIBV5".into(),
        CF_UNICODETEXT => "CF_UNICODETEXT".into(),
        _ => registered_format_name(format).unwrap_or_else(|| format!("Windows format {format}")),
    }
}

fn registered_format_name(format: u32) -> Option<String> {
    let mut buffer = [0u16; 128];
    let len = unsafe { GetClipboardFormatNameW(format, buffer.as_mut_ptr(), buffer.len() as i32) };
    if len <= 0 {
        return None;
    }

    Some(String::from_utf16_lossy(&buffer[..len as usize]))
}

fn mime_type(format: u32) -> Option<String> {
    match format {
        CF_TEXT | CF_UNICODETEXT => Some("text/plain".into()),
        CF_BITMAP | CF_DIB | CF_DIBV5 => Some("image/bmp".into()),
        _ => None,
    }
}

fn data_kind(format: u32) -> &'static str {
    match format {
        CF_TEXT | CF_UNICODETEXT => "text",
        CF_BITMAP | CF_DIB | CF_DIBV5 => "image_file",
        _ => "external_ref",
    }
}

fn is_format_available(format: u32) -> bool {
    (unsafe { IsClipboardFormatAvailable(format) }) != 0
}

struct ClipboardGuard;

impl ClipboardGuard {
    fn open() -> Result<Self, CliplyError> {
        let opened = unsafe { OpenClipboard(std::ptr::null_mut::<c_void>()) } != 0;
        if !opened {
            return Err(CliplyError::PlatformUnavailable(
                "windows clipboard is currently unavailable".into(),
            ));
        }

        Ok(Self)
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            CloseClipboard();
        }
    }
}
