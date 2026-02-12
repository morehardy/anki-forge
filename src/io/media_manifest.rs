use crate::domain::media::MediaRef;

#[must_use]
pub fn build_media_manifest(media: &[MediaRef]) -> Vec<(String, String)> {
    media
        .iter()
        .enumerate()
        .map(|(idx, item)| (idx.to_string(), item.logical_name.clone()))
        .collect()
}
