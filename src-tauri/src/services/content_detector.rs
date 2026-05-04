use crate::models::clipboard_item::ClipboardItemType;

pub fn detect_text_type(value: &str) -> ClipboardItemType {
    let trimmed = value.trim();
    let lower = trimmed.to_lowercase();

    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("file://")
    {
        return ClipboardItemType::Link;
    }

    let code_markers = [
        "import ",
        "export ",
        "function ",
        "const ",
        "let ",
        "fn ",
        "class ",
        "interface ",
        "=>",
        "SELECT ",
        " FROM ",
        " WHERE ",
    ];
    let has_code_marker = code_markers.iter().any(|marker| trimmed.contains(marker));
    let has_code_shape = trimmed.contains('{') && trimmed.contains('}')
        || trimmed.contains(';')
        || looks_like_json(trimmed);
    let has_indented_lines = trimmed
        .lines()
        .filter(|line| line.starts_with("  "))
        .count()
        >= 2;

    if has_code_marker || has_code_shape || has_indented_lines {
        return ClipboardItemType::Code;
    }

    ClipboardItemType::Text
}

fn looks_like_json(value: &str) -> bool {
    (value.starts_with('{') && value.ends_with('}'))
        || (value.starts_with('[') && value.ends_with(']'))
}
