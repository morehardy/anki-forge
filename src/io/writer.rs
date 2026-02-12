use crate::domain::media::MediaRef;

#[must_use]
pub fn write_apkg(_db_bytes: &[u8], _media: &[MediaRef]) -> Vec<u8> {
    Vec::new()
}
