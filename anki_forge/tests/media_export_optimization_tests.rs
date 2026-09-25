mod common;
use ankiforge::{BuildOptions, Media, Note, Project};
use std::path::Path;

fn media_project(root: &Path) -> Project {
    let mut project = Project::new("prepared-media").unwrap();
    for index in 0..24 {
        let name = format!("media-{index}.bin");
        let path = root.join(&name);
        let bytes = if index == 0 {
            let mut state = 17u32;
            (0..1_500_000)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 17;
                    state ^= state << 5;
                    state as u8
                })
                .collect()
        } else {
            format!("shared content {}", index % 12)
                .repeat(5000)
                .into_bytes()
        };
        std::fs::write(&path, bytes).unwrap();
        project
            .add_asset(Media::file(&path).unwrap().with_export_name(name).unwrap())
            .unwrap();
        project
            .add(
                format!("note-{index}"),
                Note::basic(format!("front {index}"), "back"),
            )
            .unwrap();
    }
    project
}

#[test]
fn temporary_and_persistent_exports_contain_identical_decoded_assets() {
    let root = tempfile::tempdir().unwrap();
    let project = media_project(root.path());
    let temporary = project.build(BuildOptions::temporary()).unwrap();
    let persistent = project
        .build(
            BuildOptions::to(root.path().join("saved.apkg"))
                .update_from(temporary.artifact().path()),
        )
        .unwrap();
    assert_eq!(
        common::entries(temporary.artifact().path()),
        common::entries(persistent.artifact().path())
    );
    assert_eq!(persistent.report().counts().media, 24);
}

#[test]
fn concurrent_builds_keep_media_and_output_independent() {
    let root = tempfile::tempdir().unwrap();
    let project = media_project(root.path());
    let initial = project.build(BuildOptions::temporary()).unwrap();
    let outputs = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..3)
            .map(|_| {
                scope.spawn(|| {
                    let output = project
                        .build(BuildOptions::temporary().update_from(initial.artifact().path()))
                        .unwrap();
                    (
                        output.artifact().path().to_owned(),
                        common::entries(output.artifact().path()),
                    )
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(outputs.windows(2).all(|pair| pair[0].1 == pair[1].1));
    assert!(outputs.iter().all(|(path, _)| !path.exists()));
}

#[test]
fn same_name_changed_snapshot_conflict_is_atomic() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("same.txt");
    std::fs::write(&source, b"original").unwrap();
    let mut project = Project::new("individual-media").unwrap();
    project
        .add_asset(
            Media::file(&source)
                .unwrap()
                .with_export_name("same.txt")
                .unwrap(),
        )
        .unwrap();
    project.add("one", Note::basic("front", "back")).unwrap();
    std::fs::write(&source, b"modified").unwrap();
    let error = project
        .add_asset(
            Media::file(&source)
                .unwrap()
                .with_export_name("same.txt")
                .unwrap(),
        )
        .unwrap_err();
    assert_eq!(error.code(), "MEDIA.DUPLICATE_FILENAME_CONFLICT");
    let output = project.build(BuildOptions::temporary()).unwrap();
    let evidence = common::evidence(output.artifact().path());
    assert_eq!(evidence["media"]["same.txt"]["size"], 8);
    assert_eq!(output.report().counts().media, 1);
}

#[test]
fn deleting_or_replacing_source_paths_after_import_does_not_change_snapshots() {
    let root = tempfile::tempdir().unwrap();
    let project = media_project(root.path());
    let first = project.build(BuildOptions::temporary()).unwrap();
    for index in 0..24 {
        let path = root.path().join(format!("media-{index}.bin"));
        if index % 2 == 0 {
            std::fs::remove_file(path).unwrap();
        } else {
            std::fs::write(path, b"changed after import").unwrap();
        }
    }
    let next = project
        .build(BuildOptions::temporary().update_from(first.artifact().path()))
        .unwrap();
    assert_eq!(
        common::entries(first.artifact().path()),
        common::entries(next.artifact().path())
    );
}

#[test]
fn byte_media_accepts_compatible_containers_through_publication() {
    let mut project = Project::new("compatible-byte-media").unwrap();
    project.add("one", Note::basic("front", "back")).unwrap();
    for (bytes, declared, extension) in [
        (b"\x1a\x45\xdf\xa3webm".as_slice(), "audio/webm", "webm"),
        (b"OggSOpusHead".as_slice(), "audio/opus", "opus"),
        (b"\0\0\0\x18ftypisom".as_slice(), "audio/mp4", "m4a"),
        (b"\0\0\0\x18ftypM4A ".as_slice(), "video/mp4", "mp4"),
    ] {
        let media = Media::bytes(bytes.to_vec(), declared).unwrap();
        assert!(media.filename().ends_with(&format!(".{extension}")));
        assert_eq!(media.media_type(), declared);
        project.add_asset(media).unwrap();
    }
    let output = project.build(BuildOptions::temporary()).unwrap();
    assert_eq!(output.report().counts().media, 4);
    let evidence = common::evidence(output.artifact().path());
    assert_eq!(evidence["media"].as_object().unwrap().len(), 4);
}

#[test]
fn byte_media_canonicalizes_type_casing_without_changing_parameters() {
    for (bytes, declared, canonical, extension) in [
        (
            b"\x89PNG\r\n\x1a\n".as_slice(),
            "IMAGE/PNG",
            "image/png",
            "png",
        ),
        (
            b"\x1a\x45\xdf\xa3webm".as_slice(),
            "AUDIO/WEBM; codecs=Opus",
            "audio/webm; codecs=Opus",
            "webm",
        ),
        (
            b"OggSOpusHead".as_slice(),
            "AUDIO/OPUS",
            "audio/opus",
            "opus",
        ),
    ] {
        let media = Media::bytes(bytes.to_vec(), declared).unwrap();
        let lower = Media::bytes(bytes.to_vec(), canonical).unwrap();
        assert_eq!(media.media_type(), canonical);
        assert_eq!(media.filename(), lower.filename());
        assert!(media.filename().ends_with(&format!(".{extension}")));
    }
    assert_eq!(
        Media::bytes(b"\x89PNG\r\n\x1a\n".to_vec(), "AUDIO/WEBM")
            .unwrap_err()
            .code(),
        "MEDIA.TYPE_MISMATCH"
    );
}

// Encoded silent MPEG-1 Layer III frames, without ID3 or Xing metadata:
// ffmpeg -f lavfi -i anullsrc=r=44100:cl=mono -t 0.06 -c:a libmp3lame
//   -b:a 32k -write_xing 0 -id3v2_version 0 -write_id3v1 0 no-id3.mp3
fn no_id3_mp3() -> Vec<u8> {
    include_bytes!("fixtures/public-api/no-id3.mp3").to_vec()
}

fn bmp_image() -> Vec<u8> {
    let pixels = image::RgbImage::from_pixel(8, 6, image::Rgb([20, 60, 100]));
    let mut encoded = std::io::Cursor::new(Vec::new());
    pixels
        .write_to(&mut encoded, image::ImageFormat::Bmp)
        .unwrap();
    encoded.into_inner()
}

fn assert_file_type_survives_export(bytes: &[u8], name: &str, mime: &str, extension: &str) {
    let root = tempfile::tempdir().unwrap();
    for source_name in [name, "no-extension", "misleading.wav"] {
        let path = root.path().join(source_name);
        std::fs::write(&path, bytes).unwrap();
        let media = Media::file(&path).unwrap();
        assert_eq!(media.media_type(), mime, "{source_name}");
        assert!(media.filename().ends_with(&format!(".{extension}")));
        assert_eq!(media, Media::bytes(bytes.to_vec(), mime).unwrap());
        let mut project = Project::new("content-detected-media").unwrap();
        let content = if mime.starts_with("audio/") {
            media.sound()
        } else {
            media.image()
        };
        project.add("media", Note::basic(content, "back")).unwrap();
        if mime == "image/bmp" {
            project
                .add(
                    "masked",
                    Note::image_occlusion(media.clone())
                        .mask(ankiforge::note::Mask::rect("pixel", 1, 1, 3, 2))
                        .build()
                        .unwrap(),
                )
                .unwrap();
        }
        std::fs::remove_file(path).unwrap();
        let output = project.build(BuildOptions::temporary()).unwrap();
        assert_eq!(output.report().counts().media, 1);
        assert_eq!(
            output.report().counts().cards,
            if mime == "image/bmp" { 2 } else { 1 }
        );
        let entries = common::entries(output.artifact().path());
        let actual = zstd::decode_all(entries["0"].as_slice()).unwrap();
        assert_eq!(actual, bytes);
        assert!(common::evidence(output.artifact().path())["media"]
            .get(media.filename())
            .is_some());
        let fields = common::fields(output.artifact().path());
        assert!(fields.iter().all(|field| field.contains(media.filename())));
        if mime == "image/bmp" {
            assert!(fields
                .iter()
                .any(|field| field.contains("{{c1::image-occlusion:rect:")));
        }
    }
}

#[test]
fn file_svg_keeps_image_type_name_and_package_bytes() {
    let bytes = "\u{feff}<?xml version=\"1.0\"?>\n<!--diagram-->\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"8\" height=\"6\"><text>细胞</text></svg>";
    assert_file_type_survives_export(bytes.as_bytes(), "diagram.SVG", "image/svg+xml", "svg");
}

#[test]
fn file_bmp_keeps_image_type_and_supports_occlusion_build() {
    assert_file_type_survives_export(&bmp_image(), "diagram.BMP", "image/bmp", "bmp");
}

#[test]
fn file_mp3_without_id3_keeps_audio_type_name_and_package_bytes() {
    let bytes = no_id3_mp3();
    assert!(!bytes.starts_with(b"ID3"));
    assert_file_type_survives_export(&bytes, "audio.MP3", "audio/mpeg", "mp3");
}

#[test]
fn file_type_hints_do_not_override_recognized_content() {
    let root = tempfile::tempdir().unwrap();
    for (name, bytes, expected) in [
        ("image.mp3", b"\x89PNG\r\n\x1a\n".as_slice(), "image/png"),
        (
            "plain.bmp",
            b"BMP is an image format".as_slice(),
            "text/plain",
        ),
        (
            "plain.mp3",
            b"this is a written transcript".as_slice(),
            "text/plain",
        ),
        (
            "plain.svg",
            b"<svg-not-an-element>text</svg-not-an-element>".as_slice(),
            "text/plain",
        ),
        (
            "page.svg",
            b"<!doctype html><html></html>".as_slice(),
            "text/html",
        ),
        (
            "fragment.CSS",
            b"body { color: red; }".as_slice(),
            "text/css",
        ),
    ] {
        let path = root.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        assert_eq!(Media::file(path).unwrap().media_type(), expected, "{name}");
    }
    // A single MP3 frame is too little for a confident content match, so a
    // supported source extension provides the hint without changing its bytes.
    let frame = &no_id3_mp3()[..104];
    let path = root.path().join("short.MP3");
    std::fs::write(&path, frame).unwrap();
    let media = Media::file(path).unwrap();
    assert_eq!(media.media_type(), "audio/mpeg");
    assert_eq!(media, Media::bytes(frame.to_vec(), "audio/mpeg").unwrap());
}

#[test]
fn recognized_bmp_svg_and_mp3_reject_conflicting_byte_mime_types() {
    for bytes in [
        bmp_image(),
        br#"<svg xmlns="http://www.w3.org/2000/svg"/>"#.to_vec(),
        no_id3_mp3(),
    ] {
        let error = Media::bytes(bytes, "image/png").unwrap_err();
        assert_eq!(
            error.kind(),
            ankiforge::media::MediaErrorKind::MediaTypeMismatch
        );
        assert_eq!(error.code(), "MEDIA.TYPE_MISMATCH");
    }
}
