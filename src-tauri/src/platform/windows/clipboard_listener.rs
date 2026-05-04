use crate::error::CliplyError;
use crate::services::clipboard_service;
use std::ptr::null;
use std::sync::{
    mpsc::{self, Sender},
    Mutex, OnceLock,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::DataExchange::{
    AddClipboardFormatListener, RemoveClipboardFormatListener,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostMessageW, PostQuitMessage,
    RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, HWND_MESSAGE, MSG, WM_APP,
    WM_CLIPBOARDUPDATE, WM_DESTROY, WNDCLASSW, WS_OVERLAPPED,
};

const CLASS_NAME: &[u16] = &[
    b'C' as u16,
    b'l' as u16,
    b'i' as u16,
    b'p' as u16,
    b'l' as u16,
    b'y' as u16,
    b'C' as u16,
    b'l' as u16,
    b'i' as u16,
    b'p' as u16,
    b'b' as u16,
    b'o' as u16,
    b'a' as u16,
    b'r' as u16,
    b'd' as u16,
    b'L' as u16,
    b'i' as u16,
    b's' as u16,
    b't' as u16,
    b'e' as u16,
    b'n' as u16,
    b'e' as u16,
    b'r' as u16,
    0,
];
const WM_CLIPLY_STOP: u32 = WM_APP + 1;

static LISTENER: OnceLock<Mutex<Option<ListenerHandle>>> = OnceLock::new();
static CHANGE_SENDER: OnceLock<Mutex<Option<Sender<()>>>> = OnceLock::new();
static SUPPRESS_UNTIL: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

pub fn start_listener(app: AppHandle) -> Result<(), CliplyError> {
    let listener_slot = LISTENER.get_or_init(|| Mutex::new(None));
    let mut listener = listener_slot
        .lock()
        .map_err(|_| CliplyError::PlatformUnavailable("clipboard listener lock poisoned".into()))?;

    if listener.is_some() {
        return Ok(());
    }

    let (message_tx, message_rx) = mpsc::channel::<ListenerMessage>();
    let (change_tx, change_rx) = mpsc::channel::<()>();
    set_change_sender(Some(change_tx))?;

    let worker_app = app.clone();
    let worker_thread = thread::Builder::new()
        .name("cliply-clipboard-worker".into())
        .spawn(move || {
            let mut last_handled_at = Instant::now() - Duration::from_secs(1);
            while change_rx.recv().is_ok() {
                if last_handled_at.elapsed() < Duration::from_millis(140) {
                    thread::sleep(Duration::from_millis(140));
                }

                last_handled_at = Instant::now();
                match clipboard_service::ingest_current_clipboard(&worker_app) {
                    Ok(Some(_item)) => {
                        let _ = worker_app.emit("clipboard-items-changed", ());
                    }
                    Ok(None) => {}
                    Err(error) => eprintln!("[cliply] clipboard ingest failed: {error}"),
                }
            }
        })
        .map_err(|error| CliplyError::PlatformUnavailable(error.to_string()))?;

    let listener_thread = thread::Builder::new()
        .name("cliply-clipboard-listener".into())
        .spawn(move || run_message_window(message_tx))
        .map_err(|error| CliplyError::PlatformUnavailable(error.to_string()))?;

    let hwnd = match message_rx.recv_timeout(Duration::from_secs(3)) {
        Ok(ListenerMessage::Ready(hwnd)) => hwnd as HWND,
        Ok(ListenerMessage::Failed(message)) => {
            set_change_sender(None)?;
            let _ = worker_thread.join();
            let _ = listener_thread.join();
            return Err(CliplyError::PlatformUnavailable(message));
        }
        Err(error) => {
            set_change_sender(None)?;
            let _ = worker_thread.join();
            let _ = listener_thread.join();
            return Err(CliplyError::PlatformUnavailable(error.to_string()));
        }
    };

    *listener = Some(ListenerHandle {
        hwnd,
        listener_thread,
        worker_thread,
    });

    Ok(())
}

pub fn stop_listener() -> Result<(), CliplyError> {
    let listener_slot = LISTENER.get_or_init(|| Mutex::new(None));
    let mut listener = listener_slot
        .lock()
        .map_err(|_| CliplyError::PlatformUnavailable("clipboard listener lock poisoned".into()))?;

    if let Some(handle) = listener.take() {
        unsafe {
            PostMessageW(handle.hwnd, WM_CLIPLY_STOP, 0, 0);
        }
        set_change_sender(None)?;
        let _ = handle.listener_thread.join();
        let _ = handle.worker_thread.join();
    }

    Ok(())
}

pub fn suppress_clipboard_events_for(duration: Duration) {
    let suppress_slot = SUPPRESS_UNTIL.get_or_init(|| Mutex::new(None));
    if let Ok(mut suppress_until) = suppress_slot.lock() {
        *suppress_until = Some(Instant::now() + duration);
    }
}

fn run_message_window(message_tx: Sender<ListenerMessage>) {
    let hwnd = unsafe { create_message_window() };
    if hwnd.is_null() {
        let _ = message_tx.send(ListenerMessage::Failed(
            "failed to create clipboard listener window".into(),
        ));
        return;
    }

    let registered = unsafe { AddClipboardFormatListener(hwnd) } != 0;
    if !registered {
        let _ = message_tx.send(ListenerMessage::Failed(
            "failed to register clipboard listener".into(),
        ));
        return;
    }

    let _ = message_tx.send(ListenerMessage::Ready(hwnd as isize));

    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) } > 0 {
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

unsafe fn create_message_window() -> HWND {
    let instance = GetModuleHandleW(null());
    let window_class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: std::ptr::null_mut(),
        hCursor: std::ptr::null_mut(),
        hbrBackground: std::ptr::null_mut(),
        lpszMenuName: null(),
        lpszClassName: CLASS_NAME.as_ptr(),
    };

    RegisterClassW(&window_class);

    CreateWindowExW(
        0,
        CLASS_NAME.as_ptr(),
        CLASS_NAME.as_ptr(),
        WS_OVERLAPPED,
        0,
        0,
        0,
        0,
        HWND_MESSAGE,
        std::ptr::null_mut(),
        instance,
        std::ptr::null(),
    )
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CLIPBOARDUPDATE => {
            notify_clipboard_changed();
            0
        }
        WM_CLIPLY_STOP => {
            RemoveClipboardFormatListener(hwnd);
            PostQuitMessage(0);
            0
        }
        WM_DESTROY => {
            RemoveClipboardFormatListener(hwnd);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

fn notify_clipboard_changed() {
    if should_suppress_clipboard_event() {
        return;
    }

    if let Some(sender_slot) = CHANGE_SENDER.get() {
        if let Ok(sender) = sender_slot.lock() {
            if let Some(sender) = sender.as_ref() {
                let _ = sender.send(());
            }
        }
    }
}

fn should_suppress_clipboard_event() -> bool {
    let suppress_slot = SUPPRESS_UNTIL.get_or_init(|| Mutex::new(None));
    let Ok(mut suppress_until) = suppress_slot.lock() else {
        return false;
    };

    match *suppress_until {
        Some(until) if Instant::now() <= until => true,
        Some(_) => {
            *suppress_until = None;
            false
        }
        None => false,
    }
}

fn set_change_sender(sender: Option<Sender<()>>) -> Result<(), CliplyError> {
    let sender_slot = CHANGE_SENDER.get_or_init(|| Mutex::new(None));
    let mut current_sender = sender_slot
        .lock()
        .map_err(|_| CliplyError::PlatformUnavailable("clipboard sender lock poisoned".into()))?;
    *current_sender = sender;
    Ok(())
}

#[allow(dead_code)]
struct ListenerHandle {
    hwnd: HWND,
    listener_thread: JoinHandle<()>,
    worker_thread: JoinHandle<()>,
}

unsafe impl Send for ListenerHandle {}

enum ListenerMessage {
    Ready(isize),
    Failed(String),
}
