use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardItemType {
    Text,
    Link,
    Image,
    Code,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItemDto {
    pub id: String,
    pub item_type: ClipboardItemType,
    pub title: String,
    pub preview_text: String,
    pub source_app: String,
    pub source_window: Option<String>,
    pub copied_at: String,
    pub created_at: String,
    pub relative_time: String,
    pub size_bytes: i64,
    pub is_pinned: bool,
    pub sensitive_score: i64,
    pub is_redacted: bool,
    pub tags: Vec<String>,
    pub thumbnail_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardFormatDto {
    pub format_name: String,
    pub mime_type: Option<String>,
    pub data_kind: String,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItemDetailDto {
    pub item: ClipboardItemDto,
    pub full_text: Option<String>,
    pub thumbnail_path: Option<String>,
    pub image_path: Option<String>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub formats: Vec<ClipboardFormatDto>,
}
