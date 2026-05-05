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
        }
    }
}
