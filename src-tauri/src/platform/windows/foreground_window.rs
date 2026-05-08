use crate::platform::ForegroundAppInfo;
use std::sync::atomic::{AtomicIsize, Ordering};
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
    IsWindow, IsWindowVisible, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

static LAST_PASTE_TARGET: AtomicIsize = AtomicIsize::new(0);

pub fn current_foreground_app() -> Option<ForegroundAppInfo> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return None;
    }

    let mut process_id = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut process_id);
    }

    let app_name = process_name(process_id).unwrap_or_else(|| "Windows Clipboard".to_string());
    let window_title = window_title(hwnd);

    Some(ForegroundAppInfo {
        app_name,
        window_title,
    })
}

pub fn remember_paste_target(excluded_window: Option<isize>) {
    let hwnd = unsafe { GetForegroundWindow() };
    if is_paste_target_candidate(hwnd, excluded_window) {
        LAST_PASTE_TARGET.store(hwnd as isize, Ordering::Relaxed);
    }
}

pub fn restore_paste_target() -> bool {
    let hwnd_value = LAST_PASTE_TARGET.load(Ordering::Relaxed);
    if hwnd_value == 0 {
        return false;
    }

    let hwnd = hwnd_value as windows_sys::Win32::Foundation::HWND;
    if !is_paste_target_candidate(hwnd, None) {
        LAST_PASTE_TARGET.store(0, Ordering::Relaxed);
        return false;
    }

    unsafe {
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        }
    }

    focus_window(hwnd)
}

fn is_paste_target_candidate(
    hwnd: windows_sys::Win32::Foundation::HWND,
    excluded_window: Option<isize>,
) -> bool {
    if hwnd.is_null() {
        return false;
    }

    if excluded_window.is_some_and(|excluded| excluded == hwnd as isize) {
        return false;
    }

    let mut process_id = 0u32;
    unsafe {
        if IsWindow(hwnd) == 0 || IsWindowVisible(hwnd) == 0 {
            return false;
        }
        GetWindowThreadProcessId(hwnd, &mut process_id);
    }

    process_id != std::process::id()
}

fn focus_window(hwnd: windows_sys::Win32::Foundation::HWND) -> bool {
    let foreground = unsafe { GetForegroundWindow() };
    if foreground == hwnd {
        return true;
    }

    let focused = unsafe {
        BringWindowToTop(hwnd);
        SetForegroundWindow(hwnd) != 0
    };

    focused || unsafe { GetForegroundWindow() == hwnd }
}

fn window_title(hwnd: windows_sys::Win32::Foundation::HWND) -> Option<String> {
    let mut buffer = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };

    if len <= 0 {
        return None;
    }

    Some(String::from_utf16_lossy(&buffer[..len as usize]))
}

fn process_name(process_id: u32) -> Option<String> {
    if process_id == 0 {
        return None;
    }

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id) };
    if handle.is_null() {
        return None;
    }

    let mut buffer = [0u16; 1024];
    let mut len = buffer.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut len) } != 0;

    unsafe {
        CloseHandle(handle);
    }

    if !ok || len == 0 {
        return None;
    }

    let path = String::from_utf16_lossy(&buffer[..len as usize]);
    Some(
        path.rsplit(['\\', '/'])
            .next()
            .unwrap_or(path.as_str())
            .trim_end_matches(".exe")
            .to_string(),
    )
}
