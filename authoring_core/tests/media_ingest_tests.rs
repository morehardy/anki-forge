use authoring_core::{
    ingest_authoring_media, normalize, normalize_with_options, object_store_path,
    sort_media_bindings, sort_media_objects, sort_media_references, AuthoringDocument,
    AuthoringMedia, AuthoringMediaSource, AuthoringNote, AuthoringNotetype, ComparisonContext,
    DiagnosticBehavior, MediaBinding, MediaObject, MediaPolicy, MediaReference,
    MediaReferenceResolution, NormalizationRequest, NormalizeOptions, NormalizedIr,
};
use std::collections::BTreeMap;
use std::fs;
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

#[test]
fn ingest_path_source_writes_original_bytes_to_cas() {
    let root = unique_test_root("path-source");
    let base_dir = root.join("input");
    let store_dir = root.join("store");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/hello.txt"), b"hello").unwrap();
    let options = test_options(&base_dir, &store_dir);
    let media = vec![AuthoringMedia {
        id: "media:hello".into(),
        desired_filename: "hello.txt".into(),
        source: AuthoringMediaSource::Path {
            path: "assets/hello.txt".into(),
        },
        declared_mime: Some("text/plain".into()),
    }];

    let result = ingest_authoring_media(&media, &options).unwrap();

    assert!(result.diagnostics.is_empty());
    assert_eq!(result.objects.len(), 1);
    assert_eq!(result.bindings[0].export_filename, "hello.txt");
    assert_eq!(
        fs::read(result.object_path(&result.objects[0]).unwrap()).unwrap(),
        b"hello"
    );
}

#[cfg(unix)]
#[test]
fn ingest_rejects_symlink_escape() {
    use std::os::unix::fs as unix_fs;

    let root = unique_test_root("symlink-escape");
    let base_dir = root.join("input");
    let outside_dir = root.join("outside");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::create_dir_all(&outside_dir).unwrap();
    fs::write(outside_dir.join("secret.txt"), b"secret").unwrap();
    unix_fs::symlink(
        outside_dir.join("secret.txt"),
        base_dir.join("assets/link.txt"),
    )
    .unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let media = vec![AuthoringMedia {
        id: "media:bad".into(),
        desired_filename: "bad.txt".into(),
        source: AuthoringMediaSource::Path {
            path: "assets/link.txt".into(),
        },
        declared_mime: None,
    }];

    let err = ingest_authoring_media(&media, &options).unwrap_err();

    assert!(err
        .diagnostics
        .iter()
        .any(|item| item.code == "MEDIA.UNSAFE_SOURCE_PATH"));
}

#[test]
fn ingest_rejects_empty_path_source_as_unsafe() {
    let root = unique_test_root("empty-source-path");
    let base_dir = root.join("input");
    fs::create_dir_all(&base_dir).unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let media = vec![AuthoringMedia {
        id: "media:empty".into(),
        desired_filename: "empty.txt".into(),
        source: AuthoringMediaSource::Path { path: "".into() },
        declared_mime: None,
    }];

    let err = ingest_authoring_media(&media, &options).unwrap_err();

    assert!(err
        .diagnostics
        .iter()
        .any(|item| item.code == "MEDIA.UNSAFE_SOURCE_PATH"));
}

#[test]
fn ingest_rejects_dot_component_path_source_as_unsafe() {
    let root = unique_test_root("dot-source-path");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/hello.txt"), b"hello").unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let media = vec![AuthoringMedia {
        id: "media:dot".into(),
        desired_filename: "dot.txt".into(),
        source: AuthoringMediaSource::Path {
            path: "./assets/hello.txt".into(),
        },
        declared_mime: None,
    }];

    let err = ingest_authoring_media(&media, &options).unwrap_err();

    assert!(err
        .diagnostics
        .iter()
        .any(|item| item.code == "MEDIA.UNSAFE_SOURCE_PATH"));
}

#[test]
fn ingest_rejects_inline_base64_decode_failure() {
    let root = unique_test_root("inline-decode");
    let options = test_options(&root, &root.join("store"));
    let media = vec![AuthoringMedia {
        id: "media:inline".into(),
        desired_filename: "inline.txt".into(),
        source: AuthoringMediaSource::InlineBytes {
            data_base64: "%%%".into(),
        },
        declared_mime: None,
    }];

    let err = ingest_authoring_media(&media, &options).unwrap_err();

    assert!(err
        .diagnostics
        .iter()
        .any(|item| item.code == "MEDIA.INLINE_BASE64_DECODE_FAILED"));
}

#[test]
fn inline_base64_decode_diagnostic_has_readable_summary() {
    let root = unique_test_root("inline-decode-summary");
    let options = test_options(&root, &root.join("store"));
    let media = vec![AuthoringMedia {
        id: "media:inline".into(),
        desired_filename: "inline.txt".into(),
        source: AuthoringMediaSource::InlineBytes {
            data_base64: "%%%".into(),
        },
        declared_mime: None,
    }];

    let err = ingest_authoring_media(&media, &options).unwrap_err();
    let diagnostic = err
        .diagnostics
        .iter()
        .find(|item| item.code == "MEDIA.INLINE_BASE64_DECODE_FAILED")
        .unwrap();

    assert!(diagnostic.summary.contains("decode inline bytes"));
    assert!(diagnostic.summary.contains("media:inline"));
    assert!(!diagnostic.summary.contains("InlineBase64Decode"));
}

#[test]
fn ingest_rejects_path_source_that_is_not_regular_file() {
    let root = unique_test_root("not-regular");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets/dir")).unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let media = vec![AuthoringMedia {
        id: "media:dir".into(),
        desired_filename: "dir.txt".into(),
        source: AuthoringMediaSource::Path {
            path: "assets/dir".into(),
        },
        declared_mime: None,
    }];

    let err = ingest_authoring_media(&media, &options).unwrap_err();

    assert!(err
        .diagnostics
        .iter()
        .any(|item| item.code == "MEDIA.SOURCE_NOT_REGULAR_FILE"));
}

#[test]
fn oversized_path_source_is_rejected_before_final_cas_object_is_written() {
    let root = unique_test_root("oversized-path-no-cas");
    let base_dir = root.join("input");
    let store_dir = root.join("store");
    let bytes = b"oversized";
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/blob.txt"), bytes).unwrap();
    let mut options = test_options(&base_dir, &store_dir);
    options.media_policy.max_media_object_bytes = Some((bytes.len() - 1) as u64);
    let media = vec![AuthoringMedia {
        id: "media:oversized".into(),
        desired_filename: "blob.txt".into(),
        source: AuthoringMediaSource::Path {
            path: "assets/blob.txt".into(),
        },
        declared_mime: Some("text/plain".into()),
    }];
    let blake3 = blake3::hash(bytes).to_hex().to_string();
    let object_path = object_store_path(&store_dir, &blake3).unwrap();

    let err = ingest_authoring_media(&media, &options).unwrap_err();

    assert!(err
        .diagnostics
        .iter()
        .any(|item| item.code == "MEDIA.SIZE_LIMIT_EXCEEDED"));
    assert!(!object_path.exists());
}

#[test]
fn same_export_filename_and_same_object_merges_binding_with_info() {
    let root = unique_test_root("same-filename-same-object");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/hello.txt"), b"hello").unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let media = vec![
        AuthoringMedia {
            id: "media:first".into(),
            desired_filename: "hello.txt".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/hello.txt".into(),
            },
            declared_mime: Some("text/plain".into()),
        },
        AuthoringMedia {
            id: "media:second".into(),
            desired_filename: "hello.txt".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/hello.txt".into(),
            },
            declared_mime: Some("text/plain".into()),
        },
    ];

    let result = ingest_authoring_media(&media, &options).unwrap();

    assert_eq!(result.bindings.len(), 1);
    assert!(result
        .diagnostics
        .iter()
        .any(|item| item.level == "info" && item.code == "MEDIA.DEDUPED_BINDING"));
}

#[test]
fn same_export_filename_and_different_object_is_conflict() {
    let root = unique_test_root("same-filename-conflict");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/left.txt"), b"left").unwrap();
    fs::write(base_dir.join("assets/right.txt"), b"right").unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let media = vec![
        AuthoringMedia {
            id: "media:left".into(),
            desired_filename: "hello.txt".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/left.txt".into(),
            },
            declared_mime: Some("text/plain".into()),
        },
        AuthoringMedia {
            id: "media:right".into(),
            desired_filename: "hello.txt".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/right.txt".into(),
            },
            declared_mime: Some("text/plain".into()),
        },
    ];

    let err = ingest_authoring_media(&media, &options).unwrap_err();

    assert!(err
        .diagnostics
        .iter()
        .any(|item| item.code == "MEDIA.DUPLICATE_FILENAME_CONFLICT"));
}

#[test]
fn different_filenames_for_same_object_are_allowed_with_deduped_object_info() {
    let root = unique_test_root("deduped-object");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/hello.txt"), b"hello").unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let media = vec![
        AuthoringMedia {
            id: "media:a".into(),
            desired_filename: "a.txt".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/hello.txt".into(),
            },
            declared_mime: Some("text/plain".into()),
        },
        AuthoringMedia {
            id: "media:b".into(),
            desired_filename: "b.txt".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/hello.txt".into(),
            },
            declared_mime: Some("text/plain".into()),
        },
    ];

    let result = ingest_authoring_media(&media, &options).unwrap();

    assert_eq!(result.objects.len(), 1);
    assert_eq!(result.bindings.len(), 2);
    assert!(result
        .diagnostics
        .iter()
        .any(|item| item.level == "info" && item.code == "MEDIA.DEDUPED_OBJECT"));
}

#[test]
fn declared_mime_high_confidence_conflict_is_error() {
    let root = unique_test_root("mime-conflict");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(
        base_dir.join("assets/image.bin"),
        b"\x89PNG\r\n\x1a\npayload",
    )
    .unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let media = vec![AuthoringMedia {
        id: "media:image".into(),
        desired_filename: "image.png".into(),
        source: AuthoringMediaSource::Path {
            path: "assets/image.bin".into(),
        },
        declared_mime: Some("image/jpeg".into()),
    }];

    let err = ingest_authoring_media(&media, &options).unwrap_err();

    assert!(err
        .diagnostics
        .iter()
        .any(|item| item.code == "MEDIA.DECLARED_MIME_MISMATCH"));
}

#[test]
fn declared_mime_comparison_is_case_insensitive_for_type_and_subtype() {
    let root = unique_test_root("mime-case");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(
        base_dir.join("assets/image.bin"),
        b"\x89PNG\r\n\x1a\npayload",
    )
    .unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let media = vec![AuthoringMedia {
        id: "media:image".into(),
        desired_filename: "image.png".into(),
        source: AuthoringMediaSource::Path {
            path: "assets/image.bin".into(),
        },
        declared_mime: Some("IMAGE/PNG".into()),
    }];

    let result = ingest_authoring_media(&media, &options).unwrap();

    assert_eq!(result.objects[0].mime, "image/png");
    assert!(result
        .diagnostics
        .iter()
        .all(|item| item.code != "MEDIA.DECLARED_MIME_MISMATCH"));
}

#[test]
fn rejected_object_does_not_count_toward_total_unique_size_limit() {
    let root = unique_test_root("rejected-total-size-accounting");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/rejected.bin"), b"\x89PNG\r\n\x1a\nx").unwrap();
    fs::write(base_dir.join("assets/accepted.txt"), b"ok\n").unwrap();
    let mut options = test_options(&base_dir, &root.join("store"));
    options.media_policy.max_total_media_bytes = Some(10);
    let media = vec![
        AuthoringMedia {
            id: "media:rejected".into(),
            desired_filename: "rejected.png".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/rejected.bin".into(),
            },
            declared_mime: Some("image/jpeg".into()),
        },
        AuthoringMedia {
            id: "media:accepted".into(),
            desired_filename: "accepted.txt".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/accepted.txt".into(),
            },
            declared_mime: Some("text/plain".into()),
        },
    ];

    let err = ingest_authoring_media(&media, &options).unwrap_err();

    assert!(err
        .diagnostics
        .iter()
        .any(|item| item.code == "MEDIA.DECLARED_MIME_MISMATCH"));
    assert!(err
        .diagnostics
        .iter()
        .all(|item| item.code != "MEDIA.SIZE_LIMIT_EXCEEDED"));
}

#[test]
fn unknown_mime_and_size_limits_follow_policy() {
    let root = unique_test_root("policy");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/blob.bin"), [0_u8, 159, 146, 150]).unwrap();
    fs::write(base_dir.join("assets/large.txt"), b"large").unwrap();
    let mut options = test_options(&base_dir, &root.join("store"));
    options.media_policy.unknown_mime_behavior = DiagnosticBehavior::Error;
    options.media_policy.max_media_object_bytes = Some(4);
    let media = vec![
        AuthoringMedia {
            id: "media:blob".into(),
            desired_filename: "blob.bin".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/blob.bin".into(),
            },
            declared_mime: None,
        },
        AuthoringMedia {
            id: "media:large".into(),
            desired_filename: "large.txt".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/large.txt".into(),
            },
            declared_mime: Some("text/plain".into()),
        },
    ];

    let err = ingest_authoring_media(&media, &options).unwrap_err();
    let codes = err
        .diagnostics
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<_>>();

    assert!(codes.contains(&"MEDIA.UNKNOWN_MIME"));
    assert!(codes.contains(&"MEDIA.SIZE_LIMIT_EXCEEDED"));
}

#[test]
fn total_unique_media_size_limit_is_enforced() {
    let root = unique_test_root("total-size-limit");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/a.txt"), b"aaa").unwrap();
    fs::write(base_dir.join("assets/b.txt"), b"bbb").unwrap();
    let mut options = test_options(&base_dir, &root.join("store"));
    options.media_policy.max_total_media_bytes = Some(5);
    let media = vec![
        AuthoringMedia {
            id: "media:a".into(),
            desired_filename: "a.txt".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/a.txt".into(),
            },
            declared_mime: Some("text/plain".into()),
        },
        AuthoringMedia {
            id: "media:b".into(),
            desired_filename: "b.txt".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/b.txt".into(),
            },
            declared_mime: Some("text/plain".into()),
        },
    ];

    let err = ingest_authoring_media(&media, &options).unwrap_err();

    assert!(err
        .diagnostics
        .iter()
        .any(|item| item.code == "MEDIA.SIZE_LIMIT_EXCEEDED"));
}

#[test]
fn normalize_outputs_media_objects_bindings_and_resolved_references() {
    let root = unique_test_root("normalize-media");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/hello.txt"), b"hello").unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let request = NormalizationRequest::new(authoring_doc_with_media("<img src=\"hello.txt\">"));

    let result = normalize_with_options(request, options);

    assert_eq!(result.result_status, "success");
    let normalized = result.normalized_ir.unwrap();
    assert_eq!(normalized.media_objects.len(), 1);
    assert_eq!(normalized.media_bindings[0].export_filename, "hello.txt");
    assert_eq!(
        normalized.media_references[0].resolution_status(),
        "resolved"
    );
}

#[test]
fn normalize_reports_missing_and_unsafe_references() {
    let root = unique_test_root("normalize-missing");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/hello.txt"), b"hello").unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let request = NormalizationRequest::new(authoring_doc_with_media(
        "<img src=\"missing.png\"><img src=\"bad%2Fname.png\">",
    ));

    let result = normalize_with_options(request, options);

    assert_eq!(result.result_status, "invalid");
    let codes = result
        .diagnostics
        .items
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"MEDIA.MISSING_REFERENCE"));
    assert!(codes.contains(&"MEDIA.UNSAFE_REFERENCE"));
    let summaries = result
        .diagnostics
        .items
        .iter()
        .map(|item| item.summary.as_str())
        .collect::<Vec<_>>();
    assert!(summaries
        .iter()
        .any(|summary| summary.contains("note note-1 field Back")));
}

#[test]
fn normalize_preserves_successful_media_ingest_diagnostics() {
    let root = unique_test_root("normalize-ingest-diagnostics");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/blob.bin"), [0_u8, 159, 146, 150]).unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let media = vec![AuthoringMedia {
        id: "media:blob".into(),
        desired_filename: "blob.bin".into(),
        source: AuthoringMediaSource::Path {
            path: "assets/blob.bin".into(),
        },
        declared_mime: None,
    }];
    let request = NormalizationRequest::new(authoring_doc("<img src=\"blob.bin\">", media));

    let result = normalize_with_options(request, options);

    assert_eq!(result.result_status, "success");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.level == "warning" && item.code == "MEDIA.UNKNOWN_MIME"));
}

#[test]
fn normalize_warns_about_unused_media_bindings_by_default() {
    let root = unique_test_root("normalize-unused-warning");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/hello.txt"), b"hello").unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let request = NormalizationRequest::new(authoring_doc_with_media("no media refs"));

    let result = normalize_with_options(request, options);

    assert_eq!(result.result_status, "success");
    assert!(result.normalized_ir.is_some());
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.level == "warning" && item.code == "MEDIA.UNUSED_BINDING"));
}

#[test]
fn normalize_rejects_unused_media_bindings_when_policy_errors() {
    let root = unique_test_root("normalize-unused-error");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/hello.txt"), b"hello").unwrap();
    let mut options = test_options(&base_dir, &root.join("store"));
    options.media_policy.unused_binding_behavior = DiagnosticBehavior::Error;
    let request = NormalizationRequest::new(authoring_doc_with_media("no media refs"));

    let result = normalize_with_options(request, options);

    assert_eq!(result.result_status, "invalid");
    assert!(result.normalized_ir.is_none());
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.level == "error" && item.code == "MEDIA.UNUSED_BINDING"));
}

#[test]
fn normalize_keeps_skipped_references_in_valid_output() {
    let root = unique_test_root("normalize-skipped");
    let options = test_options(&root, &root.join("store"));
    let request = NormalizationRequest::new(authoring_doc(
        r#"
        <img src="https://example.test/hero.png">
        <img src="data:image/png;base64,AAAA">
        <object data="{{ dynamic_media }}"></object>
        "#,
        vec![],
    ));

    let result = normalize_with_options(request, options);

    assert_eq!(result.result_status, "success");
    assert!(result
        .diagnostics
        .items
        .iter()
        .all(|item| item.level != "error"));
    let normalized = result.normalized_ir.expect("normalized_ir");
    assert_eq!(normalized.media_references.len(), 3);
    assert!(normalized
        .media_references
        .iter()
        .all(|reference| reference.resolution_status() == "skipped"));
}

#[test]
fn normalize_drops_empty_skipped_references_from_valid_output() {
    let root = unique_test_root("normalize-empty-refs");
    let options = test_options(&root, &root.join("store"));
    let request = NormalizationRequest::new(authoring_doc(
        r#"<img src=""> [sound:] <object data=""></object>"#,
        vec![],
    ));

    let result = normalize_with_options(request, options);

    assert_eq!(result.result_status, "success");
    assert!(result
        .diagnostics
        .items
        .iter()
        .all(|item| item.level != "error"));
    let normalized = result.normalized_ir.expect("normalized_ir");
    assert!(normalized.media_references.is_empty());
}

#[test]
fn normalize_rejects_media_without_explicit_options() {
    let mut request =
        NormalizationRequest::new(authoring_doc_with_media("<img src=\"hello.txt\">"));
    request.comparison_context = Some(ComparisonContext::normalized(
        "baseline-fingerprint",
        "risk-policy.review@1.0.0",
    ));

    let result = normalize(request);

    assert_eq!(result.result_status, "invalid");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.code == "MEDIA.NORMALIZE_OPTIONS_REQUIRED"));
    assert!(result.comparison_context.is_some());
    let merge_risk_report = result.merge_risk_report.expect("merge risk report");
    assert_eq!(merge_risk_report.comparison_status, "unavailable");
    assert_eq!(
        merge_risk_report.baseline_artifact_fingerprint,
        "baseline-fingerprint"
    );
}

fn authoring_doc_with_media(back: &str) -> AuthoringDocument {
    let media = vec![AuthoringMedia {
        id: "media:hello".into(),
        desired_filename: "hello.txt".into(),
        source: AuthoringMediaSource::Path {
            path: "assets/hello.txt".into(),
        },
        declared_mime: Some("text/plain".into()),
    }];
    authoring_doc(back, media)
}

fn authoring_doc(back: &str, media: Vec<AuthoringMedia>) -> AuthoringDocument {
    let mut fields = BTreeMap::new();
    fields.insert("Front".into(), "front".into());
    fields.insert("Back".into(), back.into());
    AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc".into(),
        notetypes: vec![AuthoringNotetype {
            id: "basic-main".into(),
            kind: "normal".into(),
            name: Some("Basic".into()),
            original_stock_kind: Some("basic".into()),
            original_id: None,
            fields: None,
            templates: None,
            css: None,
            field_metadata: vec![],
        }],
        notes: vec![AuthoringNote {
            id: "note-1".into(),
            notetype_id: "basic-main".into(),
            deck_name: "Default".into(),
            fields,
            tags: vec![],
        }],
        media,
    }
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

fn test_options(base_dir: &std::path::Path, media_store_dir: &std::path::Path) -> NormalizeOptions {
    NormalizeOptions {
        base_dir: base_dir.to_path_buf(),
        media_store_dir: media_store_dir.to_path_buf(),
        media_policy: MediaPolicy::default_strict(),
    }
}

fn unique_test_root(label: &str) -> std::path::PathBuf {
    let mut root = std::env::temp_dir();
    root.push(format!(
        "anki-forge-media-{label}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}
