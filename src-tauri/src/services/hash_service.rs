pub fn normalize_text_for_hash(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
