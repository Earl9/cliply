use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliplySettings {
    pub max_history_items: u32,
    pub auto_delete_days: u32,
    pub pause_monitoring: bool,
    pub ignore_duplicate: bool,
    pub save_images: bool,
    pub save_html: bool,
    pub save_sensitive: bool,
    pub global_shortcut: String,
    pub theme: String,
}

impl Default for CliplySettings {
    fn default() -> Self {
        Self {
            max_history_items: 1000,
            auto_delete_days: 30,
            pause_monitoring: false,
            ignore_duplicate: true,
            save_images: true,
            save_html: true,
            save_sensitive: false,
            global_shortcut: "Ctrl+Shift+V".to_string(),
            theme: "light".to_string(),
        }
    }
}
