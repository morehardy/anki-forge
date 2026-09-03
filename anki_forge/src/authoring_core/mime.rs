use std::path::Path;

pub const APPLICATION_OCTET_STREAM: &str = "application/octet-stream";

pub fn mime_from_filename(name: &str) -> Option<&'static str> {
    let extension = Path::new(name).extension()?.to_str()?;
    mime_from_extension(extension)
}

pub fn mime_from_filename_or_octet(name: &str) -> String {
    mime_from_filename(name)
        .unwrap_or(APPLICATION_OCTET_STREAM)
        .into()
}

pub fn mime_from_extension(extension: &str) -> Option<&'static str> {
    let extension = extension.trim_start_matches('.').to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "svg" => Some("image/svg+xml"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "mp3" => Some("audio/mpeg"),
        "wav" => Some("audio/wav"),
        "ogg" | "oga" => Some("audio/ogg"),
        "opus" => Some("audio/opus"),
        "m4a" => Some("audio/mp4"),
        "aac" => Some("audio/aac"),
        "mp4" | "m4v" => Some("video/mp4"),
        "webm" => Some("video/webm"),
        "weba" => Some("audio/webm"),
        "pdf" => Some("application/pdf"),
        "css" => Some("text/css"),
        "js" | "mjs" => Some("text/javascript"),
        "html" | "htm" => Some("text/html"),
        "ttf" => Some("font/ttf"),
        "otf" => Some("font/otf"),
        "woff" => Some("font/woff"),
        "woff2" => Some("font/woff2"),
        _ => None,
    }
}
