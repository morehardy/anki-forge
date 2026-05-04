use authoring_core::{
    sort_media_bindings, sort_media_objects, sort_media_references, AuthoringMedia,
    AuthoringMediaSource, DiagnosticBehavior, MediaBinding, MediaObject, MediaPolicy,
    MediaReference, MediaReferenceResolution, NormalizeOptions, NormalizedIr,
};
use std::path::PathBuf;

#[test]
fn authoring_media_path_source_serializes_without_payload() {
    let media = AuthoringMedia {
        id: "media:heart".into(),
        desired_filename: "heart.png".into(),
        source: AuthoringMediaSource::Path {
            path: "assets/heart.png".into(),
        },
        declared_mime: Some("image/png".into()),
    };

    let json = serde_json::to_value(media).unwrap();

    assert_eq!(json["id"], "media:heart");
    assert_eq!(json["desired_filename"], "heart.png");
    assert_eq!(json["source"]["kind"], "path");
    assert_eq!(json["source"]["path"], "assets/heart.png");
    assert!(json.get("data_base64").is_none());
}

#[test]
fn normalized_ir_serializes_media_objects_bindings_and_references() {
    let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let normalized = NormalizedIr {
        kind: "normalized-ir".into(),
        schema_version: "0.1.0".into(),
        document_id: "doc".into(),
        resolved_identity: "det:doc".into(),
        notetypes: vec![],
        notes: vec![],
        media_objects: vec![MediaObject {
            id: format!("obj:blake3:{hash}"),
            object_ref: format!("media://blake3/{hash}"),
            blake3: hash.into(),
            sha1: "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d".into(),
            size_bytes: 5,
            mime: "text/plain".into(),
        }],
        media_bindings: vec![MediaBinding {
            id: "media:hello".into(),
            export_filename: "hello.txt".into(),
            object_id: format!("obj:blake3:{hash}"),
        }],
        media_references: vec![MediaReference {
            owner_kind: "note".into(),
            owner_id: "note-1".into(),
            location_kind: "field".into(),
            location_name: "Front".into(),
            raw_ref: "hello.txt".into(),
            ref_kind: "html_src".into(),
            resolution: MediaReferenceResolution::Resolved {
                media_id: "media:hello".into(),
            },
        }],
    };

    let json = serde_json::to_value(normalized).unwrap();

    assert!(json.get("media").is_none());
    assert!(json.get("media_objects").is_some());
    assert!(json.get("media_bindings").is_some());
    assert!(json.get("media_references").is_some());
}

#[test]
fn normalize_options_default_policy_is_explicit() {
    let options = NormalizeOptions {
        base_dir: PathBuf::from("/tmp/input"),
        media_store_dir: PathBuf::from("/tmp/store"),
        media_policy: MediaPolicy {
            inline_bytes_max: 64 * 1024,
            max_media_object_bytes: None,
            max_total_media_bytes: None,
            declared_mime_mismatch_behavior: DiagnosticBehavior::Error,
            unknown_mime_behavior: DiagnosticBehavior::Warning,
            unused_binding_behavior: DiagnosticBehavior::Warning,
        },
    };

    assert_eq!(options.media_policy.inline_bytes_max, 65536);
}

#[test]
fn sort_media_objects_orders_by_id() {
    let mut objects = vec![
        media_object("obj:blake3:cccc"),
        media_object("obj:blake3:aaaa"),
        media_object("obj:blake3:bbbb"),
    ];

    sort_media_objects(&mut objects);

    assert_eq!(
        object_ids(&objects),
        vec!["obj:blake3:aaaa", "obj:blake3:bbbb", "obj:blake3:cccc"],
    );
}

#[test]
fn sort_media_bindings_orders_by_export_filename_then_id() {
    let mut bindings = vec![
        media_binding("media:beta", "b.txt"),
        media_binding("media:zeta", "a.txt"),
        media_binding("media:alpha", "b.txt"),
        media_binding("media:alpha-2", "a.txt"),
    ];

    sort_media_bindings(&mut bindings);

    assert_eq!(
        binding_keys(&bindings),
        vec![
            ("a.txt", "media:alpha-2"),
            ("a.txt", "media:zeta"),
            ("b.txt", "media:alpha"),
            ("b.txt", "media:beta"),
        ],
    );
}

#[test]
fn sort_media_references_orders_by_owner_location_raw_kind_and_resolution() {
    let mut references = vec![
        media_reference(
            "note-1",
            "Front",
            "hero.png",
            MediaReferenceResolution::Skipped {
                skip_reason: "z-reason".into(),
            },
        ),
        media_reference(
            "note-2",
            "Front",
            "hero.png",
            MediaReferenceResolution::Missing,
        ),
        media_reference(
            "note-1",
            "Front",
            "hero.png",
            MediaReferenceResolution::Resolved {
                media_id: "media:z".into(),
            },
        ),
        media_reference(
            "note-1",
            "Back",
            "hero.png",
            MediaReferenceResolution::Missing,
        ),
        media_reference(
            "note-1",
            "Front",
            "alpha.png",
            MediaReferenceResolution::Skipped {
                skip_reason: "a-reason".into(),
            },
        ),
        media_reference(
            "note-1",
            "Front",
            "hero.png",
            MediaReferenceResolution::Missing,
        ),
        media_reference(
            "note-1",
            "Front",
            "hero.png",
            MediaReferenceResolution::Resolved {
                media_id: "media:a".into(),
            },
        ),
        media_reference(
            "note-1",
            "Front",
            "hero.png",
            MediaReferenceResolution::Skipped {
                skip_reason: "a-reason".into(),
            },
        ),
    ];

    sort_media_references(&mut references);

    assert_eq!(
        reference_keys(&references),
        vec![
            "note-1|Back|hero.png|missing",
            "note-1|Front|alpha.png|skipped:a-reason",
            "note-1|Front|hero.png|missing",
            "note-1|Front|hero.png|resolved:media:a",
            "note-1|Front|hero.png|resolved:media:z",
            "note-1|Front|hero.png|skipped:a-reason",
            "note-1|Front|hero.png|skipped:z-reason",
            "note-2|Front|hero.png|missing",
        ],
    );
}

fn media_object(id: &str) -> MediaObject {
    MediaObject {
        id: id.into(),
        object_ref: format!("media://test/{id}"),
        blake3: id.trim_start_matches("obj:blake3:").into(),
        sha1: "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d".into(),
        size_bytes: 5,
        mime: "text/plain".into(),
    }
}

fn media_binding(id: &str, export_filename: &str) -> MediaBinding {
    MediaBinding {
        id: id.into(),
        export_filename: export_filename.into(),
        object_id: format!("obj:{id}"),
    }
}

fn media_reference(
    owner_id: &str,
    location_name: &str,
    raw_ref: &str,
    resolution: MediaReferenceResolution,
) -> MediaReference {
    MediaReference {
        owner_kind: "note".into(),
        owner_id: owner_id.into(),
        location_kind: "field".into(),
        location_name: location_name.into(),
        raw_ref: raw_ref.into(),
        ref_kind: "html_src".into(),
        resolution,
    }
}

fn object_ids(objects: &[MediaObject]) -> Vec<&str> {
    objects.iter().map(|object| object.id.as_str()).collect()
}

fn binding_keys(bindings: &[MediaBinding]) -> Vec<(&str, &str)> {
    bindings
        .iter()
        .map(|binding| (binding.export_filename.as_str(), binding.id.as_str()))
        .collect()
}

fn reference_keys(references: &[MediaReference]) -> Vec<String> {
    references
        .iter()
        .map(|reference| {
            format!(
                "{}|{}|{}|{}",
                reference.owner_id,
                reference.location_name,
                reference.raw_ref,
                reference_resolution_key(&reference.resolution),
            )
        })
        .collect()
}

fn reference_resolution_key(resolution: &MediaReferenceResolution) -> String {
    match resolution {
        MediaReferenceResolution::Resolved { media_id } => format!("resolved:{media_id}"),
        MediaReferenceResolution::Missing => "missing".into(),
        MediaReferenceResolution::Skipped { skip_reason } => format!("skipped:{skip_reason}"),
    }
}
