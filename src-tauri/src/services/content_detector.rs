use crate::models::clipboard_item::ClipboardItemType;

pub fn detect_text_type(value: &str) -> ClipboardItemType {
    let trimmed = value.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return ClipboardItemType::Link;
    }

    let code_markers = ["const ", "let ", "fn ", "class ", "interface ", "SELECT ", "=>"];
    if code_markers.iter().any(|marker| trimmed.contains(marker)) {
        return ClipboardItemType::Code;
    }

    ClipboardItemType::Text
}
