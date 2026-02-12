#[must_use]
pub fn encode_legacy_media_map(entries: &[(String, String)]) -> String {
    entries
        .iter()
        .map(|(k, v)| format!("{k}:{v}"))
        .collect::<Vec<String>>()
        .join("\n")
}
