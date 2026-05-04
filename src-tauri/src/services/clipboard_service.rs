use crate::error::CliplyError;
use crate::models::clipboard_item::{
    ClipboardFormatDto, ClipboardItemDetailDto, ClipboardItemDto, ClipboardItemType,
};

pub fn list_mock_items(
    query: Option<String>,
    item_type: Option<String>,
    pinned_only: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ClipboardItemDto>, CliplyError> {
    let normalized_query = query.unwrap_or_default().to_lowercase();
    let pinned_only = pinned_only.unwrap_or(false);
    let offset = offset.unwrap_or(0).max(0) as usize;
    let limit = limit.unwrap_or(50).max(1) as usize;

    let mut items = mock_items();

    if !normalized_query.is_empty() {
        items.retain(|item| {
            let haystack = format!(
                "{} {} {}",
                item.title, item.preview_text, item.source_app
            )
            .to_lowercase();
            haystack.contains(&normalized_query)
        });
    }

    if let Some(item_type) = item_type {
        let item_type = item_type.to_lowercase();
        items.retain(|item| matches_item_type(item, &item_type));
    }

    if pinned_only {
        items.retain(|item| item.is_pinned);
    }

    Ok(items.into_iter().skip(offset).take(limit).collect())
}

pub fn get_mock_item_detail(id: String) -> Result<ClipboardItemDetailDto, CliplyError> {
    let item = mock_items()
        .into_iter()
        .find(|item| item.id == id)
        .unwrap_or_else(|| mock_items().remove(0));

    Ok(ClipboardItemDetailDto {
        full_text: Some(item.preview_text.clone()),
        thumbnail_path: None,
        formats: vec![ClipboardFormatDto {
            format_name: "text/plain".to_string(),
            mime_type: Some("text/plain".to_string()),
            data_kind: "text".to_string(),
            size_bytes: item.size_bytes,
        }],
        item,
    })
}

pub fn toggle_pin_mock_item(id: String) -> Result<ClipboardItemDto, CliplyError> {
    let mut item = mock_items()
        .into_iter()
        .find(|item| item.id == id)
        .unwrap_or_else(|| mock_items().remove(0));
    item.is_pinned = !item.is_pinned;
    Ok(item)
}

fn matches_item_type(item: &ClipboardItemDto, expected: &str) -> bool {
    matches!(
        (&item.item_type, expected),
        (ClipboardItemType::Text, "text")
            | (ClipboardItemType::Link, "link")
            | (ClipboardItemType::Image, "image")
            | (ClipboardItemType::Code, "code")
    )
}

fn mock_items() -> Vec<ClipboardItemDto> {
    vec![
        ClipboardItemDto {
            id: "mock-code-1".to_string(),
            item_type: ClipboardItemType::Code,
            title: "TypeScript snippet".to_string(),
            preview_text: "const user = await getProfile(session.userId);".to_string(),
            source_app: "Visual Studio Code".to_string(),
            copied_at: "2026-05-04T10:42:18Z".to_string(),
            relative_time: "Just now".to_string(),
            size_bytes: 148,
            is_pinned: true,
        },
        ClipboardItemDto {
            id: "mock-link-1".to_string(),
            item_type: ClipboardItemType::Link,
            title: "Tauri repository".to_string(),
            preview_text: "https://github.com/tauri-apps/tauri".to_string(),
            source_app: "Chrome".to_string(),
            copied_at: "2026-05-04T10:41:12Z".to_string(),
            relative_time: "1 min ago".to_string(),
            size_bytes: 37,
            is_pinned: false,
        },
        ClipboardItemDto {
            id: "mock-text-1".to_string(),
            item_type: ClipboardItemType::Text,
            title: "MVP note".to_string(),
            preview_text: "Windows MVP first, keep platform adapters clean.".to_string(),
            source_app: "Notepad".to_string(),
            copied_at: "2026-05-04T10:34:00Z".to_string(),
            relative_time: "8 min ago".to_string(),
            size_bytes: 48,
            is_pinned: false,
        },
    ]
}
