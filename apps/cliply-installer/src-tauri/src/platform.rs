use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[cfg_attr(windows, allow(dead_code))]
    #[error("当前安装器 MVP 仅支持 Windows")]
    UnsupportedPlatform,
    #[error("Windows API 调用失败：{0}")]
    WindowsApi(String),
    #[error("无法创建目录 {path}: {source}")]
    CreateDir {
        path: String,
        source: std::io::Error,
    },
    #[error("无法创建快捷方式 {path}: {message}")]
    Shortcut { path: String, message: String },
    #[error("注册表写入失败：{0}")]
    Registry(String),
}

pub type PlatformResult<T> = Result<T, PlatformError>;

#[cfg(windows)]
mod windows_impl {
    use super::{PlatformError, PlatformResult};
    use std::{
        ffi::OsStr,
        io,
        os::windows::ffi::{OsStrExt, OsStringExt},
        path::{Path, PathBuf},
        ptr,
    };
    use windows::{
        core::{Interface, PCWSTR, PWSTR},
        Win32::{
            Foundation::{HWND, LPARAM},
            System::Com::{
                CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IPersistFile,
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
            },
            UI::{
                Shell::{
                    Common::ITEMIDLIST, FOLDERID_CommonPrograms, FOLDERID_Desktop, ILFree,
                    IShellLinkW, SHBrowseForFolderW, SHGetKnownFolderPath, SHGetPathFromIDListW,
                    ShellLink, BIF_EDITBOX, BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW,
                    KNOWN_FOLDER_FLAG,
                },
                WindowsAndMessaging::SW_SHOWNORMAL,
            },
        },
    };
    use winreg::{enums::*, RegKey};

    const PRODUCT_ICON: &str = "cliply.ico";
    const PRODUCT_UNINSTALLER: &str = "uninstall.exe";
    const CLIPLY_SHORTCUT_NAME: &str = "Cliply.lnk";
    const UNINSTALL_SHORTCUT_NAME: &str = "卸载 Cliply.lnk";
    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const MAX_PATH_LEN: usize = 260;

    pub fn read_install_dir_from_registry() -> Option<String> {
        let candidates = [
            (
                HKEY_LOCAL_MACHINE,
                r"Software\cliply\Cliply",
                "InstallDir",
                KEY_READ | KEY_WOW64_64KEY,
            ),
            (
                HKEY_LOCAL_MACHINE,
                r"Software\cliply\Cliply",
                "",
                KEY_READ | KEY_WOW64_64KEY,
            ),
            (
                HKEY_LOCAL_MACHINE,
                r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Cliply",
                "InstallLocation",
                KEY_READ | KEY_WOW64_64KEY,
            ),
            (
                HKEY_CURRENT_USER,
                r"Software\cliply\Cliply",
                "InstallDir",
                KEY_READ,
            ),
            (HKEY_CURRENT_USER, r"Software\cliply\Cliply", "", KEY_READ),
            (
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Cliply",
                "InstallLocation",
                KEY_READ,
            ),
        ];

        candidates
            .into_iter()
            .find_map(|(root, path, name, access)| {
                RegKey::predef(root)
                    .open_subkey_with_flags(path, access)
                    .ok()
                    .and_then(|key| key.get_value::<String, _>(name).ok())
                    .filter(|value| !value.trim().is_empty())
            })
    }

    pub fn browse_install_dir(current_dir: &str) -> PlatformResult<Option<String>> {
        let title = wide_null("选择 Cliply 安装位置");
        let mut display_name = [0u16; MAX_PATH_LEN];
        let info = BROWSEINFOW {
            hwndOwner: HWND(std::ptr::null_mut()),
            pidlRoot: ptr::null_mut(),
            pszDisplayName: PWSTR(display_name.as_mut_ptr()),
            lpszTitle: PCWSTR(title.as_ptr()),
            ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE | BIF_EDITBOX,
            lpfn: None,
            lParam: LPARAM(0),
            iImage: 0,
        };

        let pidl: *mut ITEMIDLIST = unsafe { SHBrowseForFolderW(&info) };
        if pidl.is_null() {
            return Ok(None);
        }

        let mut selected = [0u16; MAX_PATH_LEN];
        let ok = unsafe { SHGetPathFromIDListW(pidl, &mut selected) };
        unsafe { ILFree(Some(pidl)) };

        if ok.as_bool() {
            Ok(Some(from_wide_null(&selected)))
        } else {
            Ok(Some(current_dir.to_string()))
        }
    }

    pub fn write_install_registry(
        product_name: &str,
        product_reg_key: &str,
        uninstall_key: &str,
        install_dir: &Path,
    ) -> PlatformResult<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (product_key, _) = hklm
            .create_subkey_with_flags(product_reg_key, KEY_WRITE | KEY_WOW64_64KEY)
            .map_err(|error| PlatformError::Registry(error.to_string()))?;
        product_key
            .set_value("InstallDir", &install_dir.to_string_lossy().to_string())
            .map_err(|error| PlatformError::Registry(error.to_string()))?;
        product_key
            .set_value("", &install_dir.to_string_lossy().to_string())
            .map_err(|error| PlatformError::Registry(error.to_string()))?;

        let uninstall_exe = install_dir.join(PRODUCT_UNINSTALLER);
        if !uninstall_exe.exists() {
            return Ok(());
        }

        let (uninstall, _) = hklm
            .create_subkey_with_flags(uninstall_key, KEY_WRITE | KEY_WOW64_64KEY)
            .map_err(|error| PlatformError::Registry(error.to_string()))?;
        let display_icon = install_dir.join(PRODUCT_ICON);
        uninstall
            .set_value("DisplayName", &product_name)
            .map_err(|error| PlatformError::Registry(error.to_string()))?;
        uninstall
            .set_value("DisplayVersion", &"0.1.0")
            .map_err(|error| PlatformError::Registry(error.to_string()))?;
        uninstall
            .set_value("Publisher", &"Cliply")
            .map_err(|error| PlatformError::Registry(error.to_string()))?;
        uninstall
            .set_value(
                "InstallLocation",
                &install_dir.to_string_lossy().to_string(),
            )
            .map_err(|error| PlatformError::Registry(error.to_string()))?;
        uninstall
            .set_value("DisplayIcon", &display_icon.to_string_lossy().to_string())
            .map_err(|error| PlatformError::Registry(error.to_string()))?;
        let uninstall_command = format!("\"{}\" --uninstall", uninstall_exe.to_string_lossy());
        uninstall
            .set_value("UninstallString", &uninstall_command)
            .map_err(|error| PlatformError::Registry(error.to_string()))?;
        uninstall
            .set_value("QuietUninstallString", &uninstall_command)
            .map_err(|error| PlatformError::Registry(error.to_string()))?;
        uninstall
            .set_value("NoModify", &1u32)
            .map_err(|error| PlatformError::Registry(error.to_string()))?;
        uninstall
            .set_value("NoRepair", &1u32)
            .map_err(|error| PlatformError::Registry(error.to_string()))?;
        Ok(())
    }

    pub fn create_start_menu_shortcuts(
        folder_name: &str,
        exe_path: &Path,
        icon_path: &Path,
    ) -> PlatformResult<()> {
        let programs = known_folder_path(&FOLDERID_CommonPrograms)?;
        let folder = programs.join(folder_name);
        std::fs::create_dir_all(&folder).map_err(|source| PlatformError::CreateDir {
            path: folder.to_string_lossy().to_string(),
            source,
        })?;

        create_shortcut(
            &folder.join(CLIPLY_SHORTCUT_NAME),
            exe_path,
            None,
            icon_path,
            "Cliply",
        )?;

        let uninstall_path = exe_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(PRODUCT_UNINSTALLER);
        if uninstall_path.exists() {
            create_shortcut(
                &folder.join(UNINSTALL_SHORTCUT_NAME),
                &uninstall_path,
                Some("--uninstall"),
                &uninstall_path,
                "卸载 Cliply",
            )?;
        }

        Ok(())
    }

    pub fn create_desktop_shortcut(exe_path: &Path, icon_path: &Path) -> PlatformResult<()> {
        let desktop = known_folder_path(&FOLDERID_Desktop)?;
        create_shortcut(
            &desktop.join(CLIPLY_SHORTCUT_NAME),
            exe_path,
            None,
            icon_path,
            "Cliply",
        )
    }

    pub fn remove_desktop_shortcut() -> PlatformResult<()> {
        let desktop = known_folder_path(&FOLDERID_Desktop)?;
        let shortcut = desktop.join(CLIPLY_SHORTCUT_NAME);
        if shortcut.exists() {
            std::fs::remove_file(&shortcut).map_err(|error| PlatformError::Shortcut {
                path: shortcut.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        }
        Ok(())
    }

    pub fn remove_start_menu_shortcuts(folder_name: &str) -> PlatformResult<()> {
        let programs = known_folder_path(&FOLDERID_CommonPrograms)?;
        let folder = programs.join(folder_name);

        remove_file_if_exists(&folder.join(CLIPLY_SHORTCUT_NAME))?;
        remove_file_if_exists(&folder.join(UNINSTALL_SHORTCUT_NAME))?;

        match std::fs::remove_dir(&folder) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::DirectoryNotEmpty => Ok(()),
            Err(error) => Err(PlatformError::Shortcut {
                path: folder.to_string_lossy().to_string(),
                message: error.to_string(),
            }),
        }
    }

    pub fn set_start_on_login(
        product_name: &str,
        exe_path: &Path,
        enabled: bool,
    ) -> PlatformResult<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (run_key, _) = hkcu
            .create_subkey_with_flags(RUN_KEY, KEY_WRITE)
            .map_err(|error| PlatformError::Registry(error.to_string()))?;

        if enabled {
            let command = format!("\"{}\" --minimized", exe_path.to_string_lossy());
            run_key
                .set_value(product_name, &command)
                .map_err(|error| PlatformError::Registry(error.to_string()))?;
        } else {
            let _ = run_key.delete_value(product_name);
        }

        Ok(())
    }

    pub fn remove_install_registry(
        product_reg_key: &str,
        uninstall_key: &str,
    ) -> PlatformResult<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        delete_registry_tree(&hklm, product_reg_key, KEY_WOW64_64KEY)?;
        delete_registry_tree(&hklm, uninstall_key, KEY_WOW64_64KEY)?;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        delete_registry_tree(&hkcu, product_reg_key, KEY_READ)?;
        delete_registry_tree(&hkcu, uninstall_key, KEY_READ)?;
        Ok(())
    }

    fn known_folder_path(folder_id: &windows::core::GUID) -> PlatformResult<PathBuf> {
        unsafe {
            let raw = SHGetKnownFolderPath(folder_id, KNOWN_FOLDER_FLAG(0), None)
                .map_err(|error| PlatformError::WindowsApi(error.to_string()))?;
            let path = raw
                .to_string()
                .map_err(|error| PlatformError::WindowsApi(error.to_string()))?;
            CoTaskMemFree(Some(raw.as_ptr().cast()));
            Ok(PathBuf::from(path))
        }
    }

    fn create_shortcut(
        shortcut_path: &Path,
        target_path: &Path,
        arguments: Option<&str>,
        icon_path: &Path,
        description: &str,
    ) -> PlatformResult<()> {
        let _com = ComApartment::new()?;

        let link: IShellLinkW = unsafe {
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).map_err(|error| {
                PlatformError::Shortcut {
                    path: shortcut_path.to_string_lossy().to_string(),
                    message: error.to_string(),
                }
            })?
        };

        unsafe {
            link.SetPath(PCWSTR(wide_null_path(target_path).as_ptr()))
                .map_err(|error| PlatformError::Shortcut {
                    path: shortcut_path.to_string_lossy().to_string(),
                    message: error.to_string(),
                })?;
            if let Some(parent) = target_path.parent() {
                link.SetWorkingDirectory(PCWSTR(wide_null_path(parent).as_ptr()))
                    .map_err(|error| PlatformError::Shortcut {
                        path: shortcut_path.to_string_lossy().to_string(),
                        message: error.to_string(),
                    })?;
            }
            if let Some(args) = arguments {
                link.SetArguments(PCWSTR(wide_null(args).as_ptr()))
                    .map_err(|error| PlatformError::Shortcut {
                        path: shortcut_path.to_string_lossy().to_string(),
                        message: error.to_string(),
                    })?;
            }
            link.SetDescription(PCWSTR(wide_null(description).as_ptr()))
                .map_err(|error| PlatformError::Shortcut {
                    path: shortcut_path.to_string_lossy().to_string(),
                    message: error.to_string(),
                })?;
            link.SetIconLocation(PCWSTR(wide_null_path(icon_path).as_ptr()), 0)
                .map_err(|error| PlatformError::Shortcut {
                    path: shortcut_path.to_string_lossy().to_string(),
                    message: error.to_string(),
                })?;
            link.SetShowCmd(SW_SHOWNORMAL)
                .map_err(|error| PlatformError::Shortcut {
                    path: shortcut_path.to_string_lossy().to_string(),
                    message: error.to_string(),
                })?;
        }

        if let Some(parent) = shortcut_path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| PlatformError::CreateDir {
                path: parent.to_string_lossy().to_string(),
                source,
            })?;
        }

        let persist: IPersistFile = link.cast().map_err(|error| PlatformError::Shortcut {
            path: shortcut_path.to_string_lossy().to_string(),
            message: error.to_string(),
        })?;
        unsafe {
            persist
                .Save(PCWSTR(wide_null_path(shortcut_path).as_ptr()), true)
                .map_err(|error| PlatformError::Shortcut {
                    path: shortcut_path.to_string_lossy().to_string(),
                    message: error.to_string(),
                })
        }
    }

    fn remove_file_if_exists(path: &Path) -> PlatformResult<()> {
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(PlatformError::Shortcut {
                path: path.to_string_lossy().to_string(),
                message: error.to_string(),
            }),
        }
    }

    fn delete_registry_tree(root: &RegKey, path: &str, flags: u32) -> PlatformResult<()> {
        match root.delete_subkey_with_flags(path, flags) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => root
                .delete_subkey_all(path)
                .or_else(|tree_error| {
                    if tree_error.kind() == io::ErrorKind::NotFound {
                        Ok(())
                    } else {
                        Err(error)
                    }
                })
                .map_err(|error| PlatformError::Registry(error.to_string())),
        }
    }

    struct ComApartment;

    impl ComApartment {
        fn new() -> PlatformResult<Self> {
            unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                    .ok()
                    .map_err(|error| PlatformError::WindowsApi(error.to_string()))?;
            }
            Ok(Self)
        }
    }

    impl Drop for ComApartment {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }

    fn wide_null(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(Some(0)).collect()
    }

    fn wide_null_path(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    fn from_wide_null(value: &[u16]) -> String {
        let len = value.iter().position(|c| *c == 0).unwrap_or(value.len());
        std::ffi::OsString::from_wide(&value[..len])
            .to_string_lossy()
            .to_string()
    }
}

#[cfg(windows)]
pub use windows_impl::{
    browse_install_dir, create_desktop_shortcut, create_start_menu_shortcuts,
    read_install_dir_from_registry, remove_desktop_shortcut, remove_install_registry,
    remove_start_menu_shortcuts, set_start_on_login, write_install_registry,
};

#[cfg(not(windows))]
pub fn read_install_dir_from_registry() -> Option<String> {
    None
}

#[cfg(not(windows))]
pub fn browse_install_dir(_current_dir: &str) -> PlatformResult<Option<String>> {
    Err(PlatformError::UnsupportedPlatform)
}

#[cfg(not(windows))]
pub fn write_install_registry(
    _product_name: &str,
    _product_reg_key: &str,
    _uninstall_key: &str,
    _install_dir: &Path,
) -> PlatformResult<()> {
    Err(PlatformError::UnsupportedPlatform)
}

#[cfg(not(windows))]
pub fn create_start_menu_shortcuts(
    _folder_name: &str,
    _exe_path: &Path,
    _icon_path: &Path,
) -> PlatformResult<()> {
    Err(PlatformError::UnsupportedPlatform)
}

#[cfg(not(windows))]
pub fn create_desktop_shortcut(_exe_path: &Path, _icon_path: &Path) -> PlatformResult<()> {
    Err(PlatformError::UnsupportedPlatform)
}

#[cfg(not(windows))]
pub fn remove_desktop_shortcut() -> PlatformResult<()> {
    Ok(())
}

#[cfg(not(windows))]
pub fn remove_start_menu_shortcuts(_folder_name: &str) -> PlatformResult<()> {
    Ok(())
}

#[cfg(not(windows))]
pub fn set_start_on_login(
    _product_name: &str,
    _exe_path: &Path,
    _enabled: bool,
) -> PlatformResult<()> {
    Err(PlatformError::UnsupportedPlatform)
}

#[cfg(not(windows))]
pub fn remove_install_registry(_product_reg_key: &str, _uninstall_key: &str) -> PlatformResult<()> {
    Ok(())
}
