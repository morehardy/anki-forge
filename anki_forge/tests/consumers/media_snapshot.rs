use std::error::Error;

use ankiforge::{media::{MediaError, MediaErrorKind, MediaLimits}, Media};

fn main() -> Result<(), Box<dyn Error>> {
    fn standard_error<T: Error + Send + Sync + 'static>() {}
    standard_error::<MediaError>();
    let content = vec![b'a'; 100_000];
    let bytes = Media::bytes(content.clone(), "text/plain")?;
    assert_eq!(bytes.len(), 100_000);
    assert!(bytes.filename().ends_with(".txt"));
    let path = std::env::current_dir()?.join("source.unrelated-extension");
    std::fs::write(&path, &content)?;
    let file = Media::file(&path)?;
    assert_eq!(file, bytes);
    std::fs::write(&path, b"changed after successful import")?;
    std::fs::remove_file(&path)?;
    assert_eq!(file, bytes);
    let cloned = file.clone();
    let named = file.with_export_name("labels.txt")?;
    assert_eq!(named.filename(), "labels.txt");
    assert_eq!(cloned.filename(), bytes.filename());
    let limit = MediaLimits { max_bytes: 10 };
    let error = Media::bytes_with_limits(content, "text/plain", limit).unwrap_err();
    assert_eq!(error.kind(), MediaErrorKind::ResourceLimit);
    assert_eq!(error.code(), "MEDIA.RESOURCE_LIMIT_EXCEEDED");
    let exceeded = error.limit_exceeded().unwrap();
    assert_eq!(exceeded.limit, 10);
    assert_eq!(exceeded.observed, 100_000);
    assert_eq!(exceeded.resource, "media_bytes");
    let error = Media::file(&path).unwrap_err();
    assert_eq!(error.kind(), MediaErrorKind::Io);
    assert!(error.source().unwrap().downcast_ref::<std::io::Error>().is_some());
    let error = Media::bytes(b"text".to_vec(), "not a MIME").unwrap_err();
    assert_eq!(error.kind(), MediaErrorKind::InvalidMediaType);
    for name in ["../escape", "a/b.png", "a\\b.png", "nul.txt", "CON", "LPT9.png", "trailing.", "a\n.png", "sound].mp3", "a\"b.png"] {
        let error = bytes.clone().with_export_name(name).unwrap_err();
        assert_eq!(error.kind(), MediaErrorKind::InvalidName, "{name}");
    }
    let unicode = bytes.clone().with_export_name("细胞.txt")?;
    assert_eq!(unicode.filename(), "细胞.txt");
    let png = std::fs::read("assets/pixel.png")?;
    let mismatch = Media::bytes(png.clone(), "audio/wav").unwrap_err();
    assert_eq!(mismatch.kind(), MediaErrorKind::MediaTypeMismatch);
    assert_eq!(mismatch.code(), "MEDIA.TYPE_MISMATCH");
    assert_eq!(Media::bytes(png, "image/png")?, Media::file("assets/pixel.png")?);
    Ok(())
}
