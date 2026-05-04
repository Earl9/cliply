pub fn looks_sensitive(value: &str) -> bool {
    let lower = value.to_lowercase();
    lower.contains("password")
        || lower.contains("api_key")
        || lower.contains("secret")
        || value.contains("-----BEGIN PRIVATE KEY-----")
}
