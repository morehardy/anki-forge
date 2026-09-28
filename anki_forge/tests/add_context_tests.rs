use ankiforge::note::{AddErrorKind, AddTarget};
use ankiforge::{Note, Project};

#[test]
fn nested_media_misuse_is_rejected_before_registering_any_dependency() {
    use ankiforge::{media::MediaUsage, note::AddDetail};
    use ankiforge::{Content, Media};
    let audio = Media::file(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/public-api/silence.wav"
    ))
    .unwrap();
    let note = Note::basic(
        Content::sequence([Content::text("Listen"), Content::sequence([audio.image()])]),
        "answer",
    );
    let mut project = Project::new("media-context").unwrap();
    let error = project.add("one", note).unwrap_err();
    assert_eq!(error.code(), "NOTE.MEDIA_USAGE_INVALID");
    assert!(matches!(error.context().target(), AddTarget::Field {
        field_key, content_path: Some(path), byte_range: None, ..
    } if field_key.as_str() == "front" && path == &[1, 0]));
    assert!(matches!(error.detail(), Some(AddDetail::MediaUsage {
        requested: MediaUsage::Image, media_type, media_name, ..
    }) if media_type == "audio/wav" && media_name == audio.filename()));
    project
        .add("one", Note::basic(audio.sound(), "answer"))
        .unwrap();
    let output = project.build(ankiforge::BuildOptions::temporary()).unwrap();
    assert_eq!(output.report().counts().notes, 1);
    assert_eq!(output.report().counts().media, 1);
}

#[test]
fn unknown_field_identifies_the_attempted_note_model_and_field() {
    let mut project = Project::new("course").unwrap();
    let note = Note::basic("question", "answer").field("typo", "value");
    let model_key = note.note_type().key().to_owned();
    let error = project.add("record:1", note).unwrap_err();
    assert_eq!(error.kind(), AddErrorKind::UnknownField);
    assert_eq!(error.context().note_key(), Some("record:1"));
    assert_eq!(error.context().model_key(), Some(model_key.as_str()));
    assert!(matches!(error.context().target(), AddTarget::Field {
        field_key, content_path: None, byte_range: None, ..
    } if field_key.as_str() == "typo"));
    assert!(error.detail().is_none());
    project
        .add("record:1", Note::basic("fixed", "answer"))
        .unwrap();
}

use ankiforge::media::{MediaConflictKind, MediaUsage};
use ankiforge::note::{AddDetail, Mask};
use ankiforge::{BuildOptions, Content, Field, Media, NoteType, Template};

fn fixture(name: &str) -> Media {
    Media::file(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/public-api")
            .join(name),
    )
    .unwrap()
}
fn model(key: &str, name: &str) -> NoteType {
    NoteType::builder(key)
        .name(name)
        .field(Field::new("front").required())
        .template(Template::new("card").front("{{front}}"))
        .build()
        .unwrap()
}
fn populated() -> Project {
    let mut p = Project::new("contexts").unwrap();
    p.add("one", Note::basic("front", "back")).unwrap();
    p
}

#[test]
fn original_keys_and_non_field_targets_remain_machine_readable() {
    let mut p = populated();
    let error = p.add(" bad ", Note::basic("q", "a")).unwrap_err();
    assert_eq!(error.context().note_key(), Some(" bad "));
    assert!(matches!(error.context().target(), AddTarget::NoteKey));
    let duplicate = p.add("one", Note::basic("q", "a")).unwrap_err();
    assert_eq!(duplicate.kind(), AddErrorKind::DuplicateKey);
    assert!(matches!(duplicate.context().target(), AddTarget::NoteKey));
    let bad_tag = p
        .add("tagged", Note::basic("q", "a").tags(["ok", "ok"]))
        .unwrap_err();
    assert!(
        matches!(bad_tag.context().target(), AddTarget::Tag { index: 1, value, .. } if value == "ok")
    );
    let bad_deck = p
        .add("deck", Note::basic("q", "a").deck("invalid::"))
        .unwrap_err();
    assert!(
        matches!(bad_deck.context().target(), AddTarget::Deck { name, inherited: false, .. } if name == "invalid::")
    );
    let mut inherited = Project::new("inherited").unwrap().default_deck("bad::");
    let bad_deck = inherited.add("one", Note::basic("q", "a")).unwrap_err();
    assert!(matches!(
        bad_deck.context().target(),
        AddTarget::Deck {
            inherited: true,
            ..
        }
    ));
    assert_eq!(
        p.build(BuildOptions::temporary())
            .unwrap()
            .report()
            .counts()
            .notes,
        1
    );
}

#[test]
fn nested_text_ranges_refer_to_original_utf8_and_survive_error_wrapping() {
    use std::error::Error;
    let content = Content::sequence([
        Content::text("prefix"),
        Content::sequence([Content::html("中🚀\u{1f}after")]),
    ]);
    let mut p = Project::new("ranges").unwrap();
    let error = p.add("bad", Note::basic(content, "a")).unwrap_err();
    assert_eq!(error.code(), "NOTE.FIELD_CONTENT_INVALID");
    assert!(matches!(error.context().target(), AddTarget::Field {
        content_path: Some(path), byte_range: Some(range), ..
    } if path == &[1, 0] && range == &(7..8)));
    let error = anyhow::Error::new(error).context("input row");
    let source = error.downcast_ref::<ankiforge::note::AddError>().unwrap();
    assert_eq!(source.context().note_key(), Some("bad"));
    assert!(source.source().is_none());
    let error = p
        .default_deck("bad::")
        .build(BuildOptions::temporary())
        .unwrap_err();
    let source = error
        .source()
        .unwrap()
        .downcast_ref::<ankiforge::note::AddError>()
        .unwrap();
    assert_eq!(source.context().note_key(), None);
    assert_eq!(source.context().model_key(), None);
    assert!(
        matches!(source.context().target(), AddTarget::ProjectDefaultDeck { name, .. } if name == "bad::")
    );
}

#[test]
fn required_and_model_failures_have_distinct_evidence_without_registering_bad_models() {
    let mut p = Project::new("models").unwrap();
    let first = model("first", "Shared name");
    let missing = p.add("missing", first.note()).unwrap_err();
    assert_eq!(missing.kind(), AddErrorKind::RequiredField);
    assert!(
        matches!(missing.context().target(), AddTarget::Field { field_key, content_path: None, .. } if field_key.as_str() == "front")
    );
    p.add("one", first.note().field("front", "q")).unwrap();
    let other = model("second", " Shared name ");
    let conflict = p.add("two", other.note().field("front", "q")).unwrap_err();
    assert!(
        matches!(conflict.detail(), Some(AddDetail::ModelNameConflict { existing_model_key, conflicting_name, .. }) if existing_model_key == "first" && conflicting_name == "Shared name")
    );
    let different = model("first", "Changed definition");
    let conflict = p
        .add("two", different.note().field("front", "q"))
        .unwrap_err();
    assert_eq!(conflict.detail(), Some(&AddDetail::ModelDefinitionConflict));
    p.add(
        "two",
        model("second", "Distinct name").note().field("front", "q"),
    )
    .unwrap();
    assert_eq!(
        p.build(BuildOptions::temporary())
            .unwrap()
            .report()
            .counts()
            .notes,
        2
    );
}

#[test]
fn media_conflicts_locate_incoming_assets_in_both_committed_and_pending_collections() {
    let png = fixture("pixel.png").with_export_name("shared.dat").unwrap();
    let wav = fixture("silence.wav")
        .with_export_name("shared.dat")
        .unwrap();
    let mut p = populated();
    p.add_asset(png.clone()).unwrap();
    let error = p.add_asset(wav.clone()).unwrap_err();
    assert!(
        matches!(error.context().target(), AddTarget::ExplicitAsset { media_name, .. } if media_name == "shared.dat")
    );
    assert_eq!(error.context().note_key(), None);
    assert!(
        matches!(error.detail(), Some(AddDetail::MediaConflict { kind: MediaConflictKind::DifferentContent, existing_name, incoming_name, .. }) if existing_name == "shared.dat" && incoming_name == "shared.dat")
    );
    let variant = png.clone().with_export_name("SHARED.dat").unwrap();
    let error = p.add_asset(variant).unwrap_err();
    assert!(
        matches!(error.detail(), Some(AddDetail::MediaConflict { kind: MediaConflictKind::PortableNameCollision, existing_name, incoming_name, .. }) if existing_name == "shared.dat" && incoming_name == "SHARED.dat")
    );
    let mut pending = Project::new("pending").unwrap();
    let error = pending
        .add(
            "one",
            Note::basic(
                Content::sequence([png.image(), Content::sequence([wav.sound()])]),
                "a",
            ),
        )
        .unwrap_err();
    assert!(
        matches!(error.context().target(), AddTarget::Field { content_path: Some(path), .. } if path == &[1, 0])
    );
    // An entirely different model with the previously failed key/name can be registered.
    pending.add("one", Note::basic(wav.sound(), "a")).unwrap();
    assert_eq!(
        pending
            .build(BuildOptions::temporary())
            .unwrap()
            .report()
            .counts()
            .media,
        1
    );
    let asset_model = NoteType::builder("asset-model")
        .field(Field::new("q"))
        .template(Template::new("card").front("{{q}}"))
        .asset(wav)
        .build()
        .unwrap();
    let error = p
        .add("model", asset_model.note().field("q", "q"))
        .unwrap_err();
    assert!(
        matches!(error.context().target(), AddTarget::ModelAsset { media_name, .. } if media_name == "shared.dat")
    );
    p.add_asset(png).unwrap();
    assert_eq!(
        p.build(BuildOptions::temporary())
            .unwrap()
            .report()
            .counts()
            .media,
        1
    );
}

#[test]
fn occlusion_structure_reserved_fields_and_image_conflicts_have_separate_targets() {
    let image = fixture("occlusion.png")
        .with_export_name("image.dat")
        .unwrap();
    let io = Note::image_occlusion(image)
        .mask(Mask::rect("one", 0, 0, 1, 1))
        .build()
        .unwrap();
    let mut p = populated();
    let error = p.add("io", io.note_type().note()).unwrap_err();
    assert!(matches!(error.context().target(), AddTarget::Note));
    let error = p
        .add("io", io.clone().field("image", "override"))
        .unwrap_err();
    assert!(
        matches!(error.context().target(), AddTarget::Field { field_key, content_path: None, .. } if field_key.as_str() == "image")
    );
    p.add_asset(
        fixture("silence.wav")
            .with_export_name("image.dat")
            .unwrap(),
    )
    .unwrap();
    let error = p.add("io", io).unwrap_err();
    assert!(
        matches!(error.context().target(), AddTarget::OcclusionImage { media_name, .. } if media_name == "image.dat")
    );
}

#[test]
fn media_categories_and_export_names_are_checked_at_the_documented_stages() {
    let mut p = Project::new("usage").unwrap();
    let image = fixture("pixel.png");
    let error = p.add("one", Note::basic(image.sound(), "a")).unwrap_err();
    assert_eq!(error.kind(), AddErrorKind::InvalidMediaUsage);
    assert!(matches!(
        error.detail(),
        Some(AddDetail::MediaUsage {
            requested: MediaUsage::Sound,
            ..
        })
    ));
    let unknown = Media::bytes(vec![0, 1, 2, 3], "application/octet-stream")
        .unwrap()
        .with_export_name("fake.png")
        .unwrap();
    assert_eq!(
        p.add("one", Note::basic(unknown.image(), "a"))
            .unwrap_err()
            .code(),
        "NOTE.MEDIA_USAGE_INVALID"
    );
    let declared = Media::bytes(vec![0, 1, 2, 3], "IMAGE/x-example; parameter=value").unwrap();
    p.add("declared", Note::basic(declared.image(), "a"))
        .unwrap();
    let svg = Media::bytes(
        b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1\" height=\"1\"></svg>".to_vec(),
        "image/svg+xml",
    )
    .unwrap();
    p.add("svg", Note::basic(svg.image(), "a")).unwrap();
    let video = Media::bytes(vec![0, 1, 2, 3], "video/x-example").unwrap();
    p.add("video", Note::basic(video.sound(), "a")).unwrap();
    let output = p.build(BuildOptions::temporary()).unwrap();
    assert_eq!(output.report().counts().notes, 3);
    assert!(output
        .report()
        .diagnostics()
        .iter()
        .any(|d| d.code == "MEDIA.UNKNOWN_MIME"));
    let mut wrong_name = Project::new("wrong-name").unwrap();
    wrong_name
        .add(
            "one",
            Note::basic(image.with_export_name("wrong.mp3").unwrap().image(), "a"),
        )
        .unwrap();
    let error = wrong_name.build(BuildOptions::temporary()).unwrap_err();
    assert!(error
        .report()
        .diagnostics()
        .iter()
        .any(|d| d.code == "MEDIA.DECLARED_MIME_MISMATCH"));
}

mod common;

#[test]
fn real_video_sound_references_keep_payload_bytes_and_reject_image_usage() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../contracts/fixtures/phase3/manual-desktop-v1/S06_basic_video/assets");
    let mut project = Project::new("video").unwrap();
    let mut originals = Vec::new();
    for ext in ["mp4", "webm"] {
        let path = root.join(format!("demo-clip.{ext}"));
        let bytes = std::fs::read(&path).unwrap();
        let media = Media::file(path)
            .unwrap()
            .with_export_name(format!("demo.{ext}"))
            .unwrap();
        let error = project
            .add(ext, Note::basic(media.image(), "a"))
            .unwrap_err();
        assert_eq!(error.kind(), AddErrorKind::InvalidMediaUsage);
        project.add(ext, Note::basic(media.sound(), "a")).unwrap();
        originals.push(bytes);
    }
    let output = project.build(BuildOptions::temporary()).unwrap();
    let fields = common::fields(output.artifact().path());
    assert!(fields.iter().any(|f| f.contains("[sound:demo.mp4]")));
    assert!(fields.iter().any(|f| f.contains("[sound:demo.webm]")));
    let entries = common::entries(output.artifact().path());
    let payloads: Vec<_> = entries
        .iter()
        .filter(|(k, _)| k.parse::<usize>().is_ok())
        .map(|(_, v)| zstd::decode_all(v.as_slice()).unwrap())
        .collect();
    assert_eq!(payloads.len(), 2);
    for original in originals {
        assert!(payloads.contains(&original));
    }
}

#[test]
fn untyped_assets_remain_valid_while_non_media_categories_reject_typed_nodes() {
    let mut p = Project::new("untyped").unwrap();
    for mime in [
        "application/octet-stream",
        "text/plain",
        "font/woff",
        "application/pdf",
    ] {
        let media = Media::bytes(vec![1, 2, 3, 4], mime).unwrap();
        for content in [media.image(), media.sound()] {
            let error = p.add("one", Note::basic(content, "a")).unwrap_err();
            assert!(
                matches!(error.context().target(), AddTarget::Field { content_path: Some(path), .. } if path.is_empty())
            );
            assert_eq!(error.kind(), AddErrorKind::InvalidMediaUsage);
        }
        p.add_asset(media).unwrap();
    }
    p.add("one", Note::basic(Content::html("<b>untyped</b>"), "a"))
        .unwrap();
    assert_eq!(
        p.build(BuildOptions::temporary())
            .unwrap()
            .report()
            .counts()
            .media,
        4
    );
}
