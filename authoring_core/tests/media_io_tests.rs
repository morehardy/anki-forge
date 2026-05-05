use authoring_core::{
    decode_inline_bytes, ingest_media_read_source_to_cas, object_store_path,
    CasExistingIntegrityReason, MediaIoError, MediaReadSource, MediaSniffConfidence,
};
use base64::Engine as _;
use std::fs;

#[test]
fn streams_file_source_to_cas_without_loading_payload_into_authoring_json() {
    let root = unique_test_root("stream-file");
    let source_path = root.join("input.bin");
    let media_store = root.join("store");
    fs::write(&source_path, vec![b'a'; 256 * 1024]).unwrap();

    let ingested =
        ingest_media_read_source_to_cas(MediaReadSource::File { path: &source_path }, &media_store)
            .unwrap();

    let object_path = object_store_path(&media_store, &ingested.blake3).unwrap();
    assert_eq!(fs::metadata(object_path).unwrap().len(), 256 * 1024);
    assert_eq!(ingested.size_bytes, 256 * 1024);
    assert!(matches!(
        ingested.sniffed_mime.as_ref().map(|item| item.confidence),
        None | Some(MediaSniffConfidence::Low)
    ));
}

#[test]
fn concurrent_same_object_ingest_uses_unique_temp_files() {
    let root = unique_test_root("concurrent-cas");
    let source_path = root.join("same.txt");
    let media_store = root.join("store");
    fs::write(&source_path, b"same payload").unwrap();

    let left_store = media_store.clone();
    let left_path = source_path.clone();
    let left = std::thread::spawn(move || {
        ingest_media_read_source_to_cas(MediaReadSource::File { path: &left_path }, &left_store)
            .unwrap()
    });
    let right_store = media_store.clone();
    let right_path = source_path.clone();
    let right = std::thread::spawn(move || {
        ingest_media_read_source_to_cas(MediaReadSource::File { path: &right_path }, &right_store)
            .unwrap()
    });

    let left = left.join().unwrap();
    let right = right.join().unwrap();

    assert_eq!(left.blake3, right.blake3);
    assert_eq!(
        fs::read_dir(media_store.join("tmp")).unwrap().count(),
        0,
        "temporary CAS files must be removed after finalize"
    );
}

#[test]
fn object_store_path_rejects_uppercase_hex() {
    let root = unique_test_root("lowercase-hex");
    let err = object_store_path(
        &root,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    )
    .unwrap_err();

    assert!(err.contains("lowercase"));
}

#[test]
fn inline_decode_reports_base64_and_size_errors_before_cas_ingest() {
    let decode = decode_inline_bytes("%%%", 64).unwrap_err();
    assert!(matches!(decode, MediaIoError::InlineBase64Decode { .. }));

    let too_large = base64::engine::general_purpose::STANDARD.encode([7_u8; 65]);
    let size = decode_inline_bytes(&too_large, 64).unwrap_err();
    assert!(matches!(size, MediaIoError::InlineBytesTooLarge { .. }));
}

#[test]
fn inline_decode_preflights_encoded_size_before_decode_allocation() {
    let mut oversized_invalid_base64 = "A".repeat(128);
    oversized_invalid_base64.replace_range(127..128, "%");
    let err = decode_inline_bytes(&oversized_invalid_base64, 64).unwrap_err();

    assert!(matches!(
        err,
        MediaIoError::InlineBytesTooLarge { size, limit: 64 } if size > 64
    ));
}

#[test]
fn corrupt_existing_cas_object_reports_integrity_error_and_cleans_temp_file() {
    let root = unique_test_root("corrupt-existing-cas");
    let source_path = root.join("input.txt");
    let media_store = root.join("store");
    let payload = b"same payload";
    fs::write(&source_path, payload).unwrap();

    let blake3 = blake3::hash(payload).to_hex().to_string();
    let object_path = object_store_path(&media_store, &blake3).unwrap();
    fs::create_dir_all(object_path.parent().unwrap()).unwrap();
    fs::write(&object_path, b"wrong").unwrap();

    let err =
        ingest_media_read_source_to_cas(MediaReadSource::File { path: &source_path }, &media_store)
            .unwrap_err();

    match err {
        MediaIoError::CasExistingIntegrity { path, reason } => {
            assert_eq!(path, object_path);
            assert!(matches!(
                reason,
                CasExistingIntegrityReason::Mismatch {
                    expected_size: 12,
                    actual_size: 5,
                    expected_blake3,
                    actual_blake3,
                } if expected_blake3 == blake3 && actual_blake3 != blake3
            ));
        }
        other => panic!("expected CAS integrity error, got {other:?}"),
    }
    assert_eq!(
        fs::read_dir(media_store.join("tmp")).unwrap().count(),
        0,
        "temporary CAS files must be removed after integrity failure"
    );
}

#[cfg(unix)]
#[test]
fn source_read_failure_reports_source_path() {
    let root = unique_test_root("source-read-context");
    let source_path = root.join("source-dir");
    let media_store = root.join("store");
    fs::create_dir_all(&source_path).unwrap();

    let err =
        ingest_media_read_source_to_cas(MediaReadSource::File { path: &source_path }, &media_store)
            .unwrap_err();

    assert!(matches!(
        err,
        MediaIoError::SourceRead { path: Some(path), .. } if path == source_path
    ));
}

#[test]
fn cas_write_failure_is_typed() {
    let root = unique_test_root("cas-write-failure");
    let source_path = root.join("input.txt");
    let media_store = root.join("store-is-file");
    fs::write(&source_path, b"hello").unwrap();
    fs::write(&media_store, b"not a directory").unwrap();

    let err =
        ingest_media_read_source_to_cas(MediaReadSource::File { path: &source_path }, &media_store)
            .unwrap_err();

    assert!(matches!(err, MediaIoError::CasWrite { .. }));
}

fn unique_test_root(label: &str) -> std::path::PathBuf {
    let mut root = std::env::temp_dir();
    root.push(format!(
        "anki-forge-media-io-{label}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}
