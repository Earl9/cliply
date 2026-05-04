use crate::error::CliplyError;
use crate::platform::ClipboardWritePayload;
use std::ptr::{copy_nonoverlapping, null_mut};
use std::thread;
use std::time::Duration;
use windows_sys::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE, GMEM_ZEROINIT,
};

use super::clipboard_listener;

const CF_UNICODETEXT: u32 = 13;

#[link(name = "kernel32")]
extern "system" {
    fn GlobalFree(hmem: HGLOBAL) -> HGLOBAL;
}

pub fn write_payload(
    payload: ClipboardWritePayload,
    owner_window: Option<isize>,
) -> Result<(), CliplyError> {
    let text = payload.text.ok_or_else(|| {
        CliplyError::PlatformUnavailable("only text clipboard writes are implemented".into())
    })?;

    clipboard_listener::suppress_clipboard_events_for(Duration::from_millis(700));
    write_unicode_text(&text, owner_window)
}

fn write_unicode_text(value: &str, owner_window: Option<isize>) -> Result<(), CliplyError> {
    let _clipboard = ClipboardGuard::open_with_retry(owner_window)?;
    let mut encoded = value.encode_utf16().collect::<Vec<_>>();
    encoded.push(0);

    let byte_len = encoded.len() * std::mem::size_of::<u16>();
    let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, byte_len) };
    if handle.is_null() {
        return Err(CliplyError::PlatformUnavailable(
            "failed to allocate clipboard memory".into(),
        ));
    }

    let locked = unsafe { GlobalLock(handle) } as *mut u16;
    if locked.is_null() {
        unsafe {
            GlobalFree(handle);
        }
        return Err(CliplyError::PlatformUnavailable(
            "failed to lock clipboard memory".into(),
        ));
    }

    unsafe {
        copy_nonoverlapping(encoded.as_ptr(), locked, encoded.len());
        GlobalUnlock(handle);
    }

    let emptied = unsafe { EmptyClipboard() } != 0;
    if !emptied {
        unsafe {
            GlobalFree(handle);
        }
        return Err(CliplyError::PlatformUnavailable(
            "failed to empty clipboard".into(),
        ));
    }

    let set_handle = unsafe { SetClipboardData(CF_UNICODETEXT, handle as HANDLE) };
    if set_handle.is_null() {
        unsafe {
            GlobalFree(handle);
        }
        return Err(CliplyError::PlatformUnavailable(
            "failed to set clipboard text".into(),
        ));
    }

    Ok(())
}

struct ClipboardGuard;

impl ClipboardGuard {
    fn open_with_retry(owner_window: Option<isize>) -> Result<Self, CliplyError> {
        let owner = owner_window
            .map(|handle| handle as HWND)
            .unwrap_or_else(null_mut);

        for _ in 0..6 {
            let opened = unsafe { OpenClipboard(owner) } != 0;
            if opened {
                return Ok(Self);
            }

            thread::sleep(Duration::from_millis(20));
        }

        Err(CliplyError::PlatformUnavailable(
            "windows clipboard is currently unavailable".into(),
        ))
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            CloseClipboard();
        }
    }
}
