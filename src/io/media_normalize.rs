#[must_use]
pub fn normalize_media_name(name: &str) -> String {
    name.trim().to_owned()
}
