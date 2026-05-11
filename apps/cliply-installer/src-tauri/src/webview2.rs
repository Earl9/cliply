#[cfg(windows)]
mod windows_impl {
    use std::{
        fs::File,
        io::{self, Write},
        path::PathBuf,
        process::Command,
    };

    use windows::{
        core::PCWSTR,
        Win32::UI::WindowsAndMessaging::{
            MessageBoxW, IDOK, MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MB_OKCANCEL,
        },
    };
    use winreg::{enums::*, RegKey};

    const WEBVIEW2_BOOTSTRAPPER_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
    const WEBVIEW2_CLIENT_KEY: &str =
        r"Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
    const WEBVIEW2_WOW64_CLIENT_KEY: &str =
        r"Software\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

    pub fn ensure_runtime_or_exit() {
        if is_webview2_runtime_installed() {
            return;
        }

        let should_install = message_box(
            "Cliply Installer",
            "Cliply needs Microsoft Edge WebView2 Runtime to show the modern installer.\n\nClick OK to install the runtime, then Cliply Installer will continue.",
            MB_OKCANCEL | MB_ICONINFORMATION,
        ) == IDOK;

        if !should_install {
            std::process::exit(1);
        }

        match install_webview2_runtime() {
            Ok(()) if is_webview2_runtime_installed() => {}
            Ok(()) => exit_with_error(
                "WebView2 Runtime setup finished, but Cliply still cannot find the runtime.\n\nPlease restart Windows or install Microsoft Edge WebView2 Runtime manually, then run the Cliply installer again.",
            ),
            Err(error) => exit_with_error(&format!(
                "Cliply could not prepare Microsoft Edge WebView2 Runtime.\n\n{}\n\nPlease install WebView2 Runtime manually or use the full Cliply setup package from GitHub Releases.",
                error
            )),
        }
    }

    fn is_webview2_runtime_installed() -> bool {
        [
            (HKEY_CURRENT_USER, WEBVIEW2_CLIENT_KEY, KEY_READ),
            (HKEY_CURRENT_USER, WEBVIEW2_WOW64_CLIENT_KEY, KEY_READ),
            (
                HKEY_LOCAL_MACHINE,
                WEBVIEW2_CLIENT_KEY,
                KEY_READ | KEY_WOW64_64KEY,
            ),
            (
                HKEY_LOCAL_MACHINE,
                WEBVIEW2_CLIENT_KEY,
                KEY_READ | KEY_WOW64_32KEY,
            ),
            (
                HKEY_LOCAL_MACHINE,
                WEBVIEW2_WOW64_CLIENT_KEY,
                KEY_READ | KEY_WOW64_64KEY,
            ),
            (
                HKEY_LOCAL_MACHINE,
                WEBVIEW2_WOW64_CLIENT_KEY,
                KEY_READ | KEY_WOW64_32KEY,
            ),
        ]
        .into_iter()
        .any(|(root, path, access)| {
            RegKey::predef(root)
                .open_subkey_with_flags(path, access)
                .ok()
                .and_then(|key| key.get_value::<String, _>("pv").ok())
                .is_some_and(|version| is_valid_runtime_version(&version))
        })
    }

    fn is_valid_runtime_version(version: &str) -> bool {
        let version = version.trim();
        !version.is_empty() && version != "0.0.0.0"
    }

    fn install_webview2_runtime() -> Result<(), String> {
        let path = bootstrapper_path();
        download_bootstrapper(&path).map_err(|error| format!("Download failed: {error}"))?;

        let status = Command::new(&path)
            .args(["/silent", "/install"])
            .status()
            .map_err(|error| format!("Could not start WebView2 setup: {error}"))?;

        let code = status.code().unwrap_or(-1);
        if status.success() || code == 3010 {
            let _ = std::fs::remove_file(&path);
            return Ok(());
        }

        Err(format!("WebView2 setup exited with code {code}"))
    }

    fn download_bootstrapper(path: &PathBuf) -> Result<(), String> {
        let response = ureq::get(WEBVIEW2_BOOTSTRAPPER_URL)
            .set("User-Agent", "Cliply-Installer")
            .call()
            .map_err(|error| error.to_string())?;
        let mut reader = response.into_reader();
        let mut file = File::create(path).map_err(|error| error.to_string())?;
        io::copy(&mut reader, &mut file).map_err(|error| error.to_string())?;
        file.flush().map_err(|error| error.to_string())
    }

    fn bootstrapper_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("Cliply-MicrosoftEdgeWebView2Setup.exe");
        path
    }

    fn exit_with_error(message: &str) -> ! {
        message_box("Cliply Installer", message, MB_OK | MB_ICONERROR);
        std::process::exit(1);
    }

    fn message_box(
        title: &str,
        message: &str,
        style: windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE,
    ) -> windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_RESULT {
        let title = wide_null(title);
        let message = wide_null(message);
        unsafe {
            MessageBoxW(
                None,
                PCWSTR(message.as_ptr()),
                PCWSTR(title.as_ptr()),
                style,
            )
        }
    }

    fn wide_null(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }
}

#[cfg(windows)]
pub use windows_impl::ensure_runtime_or_exit;

#[cfg(not(windows))]
pub fn ensure_runtime_or_exit() {}
