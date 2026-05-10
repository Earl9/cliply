use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliplySettings {
    pub max_history_items: u32,
    pub auto_delete_days: u32,
    pub pause_monitoring: bool,
    pub launch_at_startup: bool,
    pub start_minimized: bool,
    pub focus_search_on_open: bool,
    pub close_after_paste: bool,
    pub ignore_duplicate: bool,
    pub save_images: bool,
    pub save_html: bool,
    pub save_sensitive: bool,
    pub ignore_apps: Vec<String>,
    pub global_shortcut: String,
    pub theme: String,
    pub theme_name: String,
    pub accent_color: String,
    pub auto_theme: CliplyAutoThemeSettings,
    pub image_sync: CliplyImageSyncSettings,
    pub update: CliplyUpdateSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliplyAutoThemeSettings {
    pub enabled: bool,
    pub source: String,
    pub intensity: String,
    pub apply_scope: String,
}

impl Default for CliplyAutoThemeSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            source: "system-accent".to_string(),
            intensity: "normal".to_string(),
            apply_scope: "accent-only".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliplyImageSyncSettings {
    pub mode: String,
    pub max_dimension: u32,
    pub quality: u8,
    pub strip_metadata: bool,
    pub max_image_size_mb: u32,
}

impl Default for CliplyImageSyncSettings {
    fn default() -> Self {
        Self {
            mode: "metadata-only".to_string(),
            max_dimension: 1920,
            quality: 80,
            strip_metadata: true,
            max_image_size_mb: 25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliplyUpdateSettings {
    pub auto_check: bool,
    pub check_interval: String,
    pub channel: String,
    pub last_checked_at: Option<String>,
    pub ignored_version: Option<String>,
}

impl Default for CliplyUpdateSettings {
    fn default() -> Self {
        Self {
            auto_check: true,
            check_interval: "daily".to_string(),
            channel: "beta".to_string(),
            last_checked_at: None,
            ignored_version: None,
        }
    }
}

impl Default for CliplySettings {
    fn default() -> Self {
        Self {
            max_history_items: 1000,
            auto_delete_days: 30,
            pause_monitoring: false,
            launch_at_startup: false,
            start_minimized: false,
            focus_search_on_open: true,
            close_after_paste: true,
            ignore_duplicate: true,
            save_images: true,
            save_html: true,
            save_sensitive: false,
            ignore_apps: vec![
                "1Password".to_string(),
                "Bitwarden".to_string(),
                "KeePass".to_string(),
                "KeePassXC".to_string(),
                "Windows Credential Manager".to_string(),
            ],
            global_shortcut: "Ctrl+Shift+V".to_string(),
            theme: "light".to_string(),
            theme_name: "purple-default".to_string(),
            accent_color: "#6D4CFF".to_string(),
            auto_theme: CliplyAutoThemeSettings::default(),
            image_sync: CliplyImageSyncSettings::default(),
            update: CliplyUpdateSettings::default(),
        }
    }
}
