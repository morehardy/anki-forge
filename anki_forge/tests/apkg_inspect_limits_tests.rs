#![cfg(feature = "internal-tools")]

use std::io::Write;

use anki_forge::writer::{inspect_apkg, inspect_apkg_with_limits, InspectLimits};
use prost::Message;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[derive(Clone, PartialEq, Message)]
struct Map {
    #[prost(message, repeated, tag = "1")]
    entries: Vec<Entry>,
}

#[derive(Clone, PartialEq, Message)]
struct Entry {
    #[prost(string, tag = "1")]
    name: String,
}

fn write_archive(
    path: &std::path::Path,
    version: u8,
    payloads: &[Vec<u8>],
    zip: CompressionMethod,
    nested: bool,
) -> usize {
    let mut archive = ZipWriter::new(std::fs::File::create(path).unwrap());
    let options = SimpleFileOptions::default().compression_method(zip);
    archive.start_file("meta", options).unwrap();
    archive.write_all(&[8, version]).unwrap();
    let map = if version == 3 {
        Map {
            entries: (0..payloads.len())
                .map(|i| Entry {
                    name: format!("asset-{i}.bin"),
                })
                .collect(),
        }
        .encode_to_vec()
    } else {
        serde_json::to_vec(
            &(0..payloads.len())
                .map(|i| (i.to_string(), format!("asset-{i}.bin")))
                .collect::<std::collections::BTreeMap<_, _>>(),
        )
        .unwrap()
    };
    archive.start_file("media", options).unwrap();
    if version == 3 {
        archive
            .write_all(&zstd::stream::encode_all(map.as_slice(), 0).unwrap())
            .unwrap();
    } else {
        archive.write_all(&map).unwrap();
    }
    for (index, bytes) in payloads.iter().enumerate() {
        archive.start_file(index.to_string(), options).unwrap();
        if nested {
            archive
                .write_all(&zstd::stream::encode_all(bytes.as_slice(), 0).unwrap())
                .unwrap();
        } else {
            archive.write_all(bytes).unwrap();
        }
    }
    archive.finish().unwrap();
    map.len()
}

fn assert_limit(path: &std::path::Path, limits: &InspectLimits, resource: &str) {
    let error = inspect_apkg_with_limits(path, limits).unwrap_err();
    let exceeded = error
        .limit_exceeded()
        .unwrap_or_else(|| panic!("expected {resource}, got {error}"));
    assert_eq!(exceeded.resource, resource);
    assert!(exceeded.observed > exceeded.limit);
}

#[test]
fn default_inspection_rejects_oversized_meta_instead_of_degrading() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("meta-bomb.apkg");
    let mut archive = ZipWriter::new(std::fs::File::create(&path).unwrap());
    archive
        .start_file(
            "meta",
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
        )
        .unwrap();
    // Version 3 plus an ignored protobuf bytes field containing 64 KiB of zeros.
    archive.write_all(&[8, 3, 18, 128, 128, 4]).unwrap();
    archive.write_all(&vec![0; 65_536]).unwrap();
    archive.finish().unwrap();

    let error = inspect_apkg(&path).expect_err("metadata must have a finite default budget");
    assert!(error
        .to_string()
        .contains("INSPECT.RESOURCE_LIMIT_EXCEEDED"));
}

#[test]
fn rejects_advertised_entry_count_before_the_zip_index_is_allocated() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("index.apkg");
    let mut archive = ZipWriter::new(std::fs::File::create(&path).unwrap());
    archive
        .start_file("media", SimpleFileOptions::default())
        .unwrap();
    archive.write_all(b"{}").unwrap();
    archive.finish().unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    let end = bytes.len() - 22;
    bytes[end + 8..end + 10].copy_from_slice(&200u16.to_le_bytes());
    bytes[end + 10..end + 12].copy_from_slice(&200u16.to_le_bytes());
    std::fs::write(&path, bytes).unwrap();
    let mut limits = InspectLimits::default();
    limits.max_entries = 100;
    let error = inspect_apkg_with_limits(&path, &limits).unwrap_err();
    assert_eq!(
        error.limit_exceeded().expect("preflight limit").resource,
        "entries"
    );
}

#[test]
fn legacy_and_nested_zstd_media_enforce_exact_decoded_boundary() {
    let root = tempfile::tempdir().unwrap();
    for (version, zip, nested) in [
        (1, CompressionMethod::Deflated, false),
        (2, CompressionMethod::Stored, false),
        (3, CompressionMethod::Stored, true),
        (3, CompressionMethod::Deflated, true),
    ] {
        let path = root.path().join(format!("v{version}-{zip:?}.apkg"));
        write_archive(&path, version, &[vec![b'a'; 1024]], zip, nested);
        let mut limits = InspectLimits::default();
        for limit in [1024, 1025] {
            limits.max_media_bytes = limit;
            let report = inspect_apkg_with_limits(&path, &limits).unwrap();
            assert_eq!(report.observations.media[0]["size"], 1024);
            assert_eq!(
                report.observations.media[0]["sha1"],
                "8eca554631df9ead14510e1a70ae48c70f9b9384"
            );
        }
        limits.max_media_bytes = 1023;
        assert_limit(&path, &limits, "media_bytes");
    }
}

#[test]
fn multiple_media_entries_share_both_cumulative_budgets() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("total.apkg");
    let map_size = write_archive(
        &path,
        1,
        &[vec![0; 1024], vec![0; 1024]],
        CompressionMethod::Deflated,
        false,
    );
    let total = 2 + map_size as u64 + 2048;
    let mut limits = InspectLimits::default();
    limits.max_decoded_total_bytes = total;
    limits.max_zip_total_bytes = total;
    assert!(inspect_apkg_with_limits(&path, &limits).is_ok());
    limits.max_decoded_total_bytes = total - 1;
    assert_limit(&path, &limits, "decoded_total_bytes");
    limits.max_decoded_total_bytes = total;
    limits.max_zip_total_bytes = total - 1;
    assert_limit(&path, &limits, "zip_total_bytes");
}

#[test]
fn concatenated_frames_share_budget_and_high_window_is_typed() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("frames.apkg");
    let frame = zstd::stream::encode_all(&vec![b'a'; 1024][..], 0).unwrap();
    write_archive(
        &path,
        3,
        &[frame.repeat(2)],
        CompressionMethod::Deflated,
        false,
    );
    let mut limits = InspectLimits::default();
    limits.max_media_bytes = 2048;
    assert_eq!(
        inspect_apkg_with_limits(&path, &limits)
            .unwrap()
            .observations
            .media[0]["size"],
        2048
    );
    limits.max_media_bytes = 2047;
    assert_limit(&path, &limits, "media_bytes");
    limits.max_media_bytes = 2048;
    limits.max_zstd_window_bytes = 512;
    assert_limit(&path, &limits, "zstd_window_bytes");
}

#[test]
fn limits_cover_archive_directory_map_and_outer_zip_entry() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("limits.apkg");
    let map_size = write_archive(
        &path,
        3,
        &[vec![0; 1024]],
        CompressionMethod::Deflated,
        true,
    );
    let mut limits = InspectLimits::default();
    limits.max_archive_bytes = 1;
    assert_limit(&path, &limits, "archive_bytes");
    limits = InspectLimits::default();
    limits.max_central_directory_bytes = 1;
    assert_limit(&path, &limits, "central_directory_bytes");
    limits = InspectLimits::default();
    limits.max_media_map_bytes = map_size as u64 - 1;
    assert_limit(&path, &limits, "media_map_bytes");
    limits = InspectLimits::default();
    limits.max_zip_entry_bytes = 2;
    assert_limit(&path, &limits, "zip_entry_bytes");
}

#[test]
fn media_map_entry_count_is_checked_before_payloads_are_opened() {
    let root = tempfile::tempdir().unwrap();
    for version in [1, 3] {
        let path = root.path().join(format!("map-{version}.apkg"));
        let mut archive = ZipWriter::new(std::fs::File::create(&path).unwrap());
        archive
            .start_file("meta", SimpleFileOptions::default())
            .unwrap();
        archive.write_all(&[8, version]).unwrap();
        archive
            .start_file("media", SimpleFileOptions::default())
            .unwrap();
        if version == 1 {
            // Count duplicate input keys, not just the final HashMap size.
            archive.write_all(br#"{"0":"a","0":"b","0":"c"}"#).unwrap();
        } else {
            // Repeated empty protobuf messages still allocate entries.
            archive
                .write_all(&zstd::stream::encode_all(&[10, 0, 10, 0, 10, 0][..], 0).unwrap())
                .unwrap();
        }
        archive.finish().unwrap();
        let mut limits = InspectLimits::default();
        limits.max_entries = 2;
        assert_limit(&path, &limits, "media_entries");
    }
}

fn project() -> anki_forge::prelude::Project {
    use anki_forge::prelude::*;
    let mut project = Project::new("Inspect limits").stable_id("inspect-limits");
    project
        .add_note(Note::basic("front", "back").stable_id("one"))
        .unwrap();
    project
}

#[test]
fn baseline_limits_reach_build_reports_in_every_update_mode() {
    use anki_forge::build::ComparisonStatus;
    use anki_forge::prelude::*;
    for mode in [
        UpdateSafetyMode::Strict,
        UpdateSafetyMode::ReportOnly,
        UpdateSafetyMode::Disabled,
    ] {
        let root = tempfile::tempdir().unwrap();
        let baseline = root.path().join("baseline.apkg");
        write_archive(
            &baseline,
            3,
            &[vec![0; 1024]],
            CompressionMethod::Deflated,
            true,
        );
        let output = root.path().join("output.apkg");
        let report_path = root.path().join("report.json");
        let mut limits = InspectLimits::default();
        limits.max_media_bytes = 1023;
        let result = project().build(
            BuildOptions::new()
                .output(&output)
                .compare_to(&baseline)
                .inspect_limits(limits)
                .update_safety(mode)
                .report_json(&report_path),
        );
        let report = if mode == UpdateSafetyMode::Strict {
            assert!(!output.exists());
            *result.unwrap_err().report
        } else {
            result.unwrap()
        };
        assert_eq!(report.comparison, ComparisonStatus::Unavailable);
        assert!(report.previous_inspect.is_none());
        assert!(report.diff.is_none());
        let diagnostics: Vec<_> = report
            .diagnostics
            .iter()
            .filter(|d| d.code.as_str() == "INSPECT.RESOURCE_LIMIT_EXCEEDED")
            .collect();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("media_bytes"));
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(report_path).unwrap()).unwrap();
        assert!(
            json["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["code"] == "INSPECT.RESOURCE_LIMIT_EXCEEDED"
                    && d["domain"] == "inspection")
        );
    }
}

#[test]
fn collection_boundary_preserves_observations_and_failure_preserves_publication() {
    use anki_forge::prelude::*;
    use std::io::Read;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("source.apkg");
    project().write_apkg(&path).unwrap();
    let expected = inspect_apkg(&path).unwrap();
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&path).unwrap()).unwrap();
    let mut encoded = Vec::new();
    archive
        .by_name("collection.anki21b")
        .unwrap()
        .read_to_end(&mut encoded)
        .unwrap();
    let size = zstd::stream::decode_all(encoded.as_slice()).unwrap().len() as u64;
    let mut limits = InspectLimits::default();
    limits.max_collection_bytes = size;
    assert_eq!(
        inspect_apkg_with_limits(&path, &limits)
            .unwrap()
            .artifact_fingerprint,
        expected.artifact_fingerprint
    );
    limits.max_collection_bytes -= 1;
    assert_limit(&path, &limits, "collection_bytes");

    let output = root.path().join("published.apkg");
    let artifacts = root.path().join("artifacts");
    std::fs::create_dir(&artifacts).unwrap();
    let package = artifacts.join("package.apkg");
    let lockfile = root.path().join("identity.json");
    project()
        .build(
            BuildOptions::new()
                .output(&output)
                .first_update_safe_build(&lockfile),
        )
        .unwrap();
    let original_output = std::fs::read(&output).unwrap();
    let original_lockfile = std::fs::read(&lockfile).unwrap();
    std::fs::write(&package, b"existing package").unwrap();
    limits.max_collection_bytes = 1;
    let report = project()
        .build(
            BuildOptions::new()
                .output(&output)
                .artifacts_dir(&artifacts)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true)
                .inspect_limits(limits),
        )
        .unwrap_err()
        .report;
    assert!(report.artifact.is_none());
    assert!(report
        .diagnostic_codes()
        .contains(&"INSPECT.RESOURCE_LIMIT_EXCEEDED".into()));
    assert!(!report.update_safety.as_ref().unwrap().lockfile_written);
    assert_eq!(std::fs::read(output).unwrap(), original_output);
    assert_eq!(std::fs::read(lockfile).unwrap(), original_lockfile);
    assert_eq!(std::fs::read(package).unwrap(), b"existing package");
}

#[test]
fn project_diff_with_limits_never_claims_complete_comparison() {
    let root = tempfile::tempdir().unwrap();
    let baseline = root.path().join("baseline.apkg");
    write_archive(
        &baseline,
        3,
        &[vec![0; 1024]],
        CompressionMethod::Stored,
        true,
    );
    let mut limits = InspectLimits::default();
    limits.max_media_bytes = 1023;
    let report = project()
        .diff_against_apkg_with_limits(baseline, limits)
        .unwrap_err()
        .report;
    assert_eq!(
        report.comparison,
        anki_forge::build::ComparisonStatus::Unavailable
    );
    assert!(report.diff.is_none());
}

#[test]
fn zip64_footer_uses_validated_64_bit_counts() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("zip64.apkg");
    write_archive(&path, 1, &[], CompressionMethod::Stored, false);
    let mut bytes = std::fs::read(&path).unwrap();
    let end_pos = bytes.len() - 22;
    let mut end = bytes.split_off(end_pos);
    let mut footer = Vec::from(&b"PK\x06\x06"[..]);
    footer.extend(44u64.to_le_bytes());
    footer.extend(45u16.to_le_bytes());
    footer.extend(45u16.to_le_bytes());
    footer.extend([0; 8]);
    footer.extend(2u64.to_le_bytes());
    footer.extend(2u64.to_le_bytes());
    footer.extend((u32::from_le_bytes(end[12..16].try_into().unwrap()) as u64).to_le_bytes());
    footer.extend((u32::from_le_bytes(end[16..20].try_into().unwrap()) as u64).to_le_bytes());
    footer.extend(b"PK\x06\x07");
    footer.extend(0u32.to_le_bytes());
    footer.extend((end_pos as u64).to_le_bytes());
    footer.extend(1u32.to_le_bytes());
    end[8..12].fill(255);
    end[12..20].fill(255);
    bytes.extend(footer);
    bytes.extend(end);
    std::fs::write(&path, &bytes).unwrap();
    let mut limits = InspectLimits::default();
    limits.max_entries = 2;
    assert!(inspect_apkg_with_limits(&path, &limits).is_ok());
    for offset in [24, 32] {
        bytes[end_pos + offset..end_pos + offset + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    }
    std::fs::write(&path, bytes).unwrap();
    assert_limit(&path, &limits, "entries");
}

#[test]
fn actual_zip_output_is_bounded_even_with_forged_small_sizes() {
    let root = tempfile::tempdir().unwrap();
    for version in [1, 3] {
        let path = root.path().join(format!("forged-{version}.apkg"));
        let payload = (0..1024).map(|i| (i % 251) as u8).collect();
        write_archive(
            &path,
            version,
            &[payload],
            CompressionMethod::Deflated,
            version == 3,
        );
        let mut bytes = std::fs::read(&path).unwrap();
        let central = bytes
            .windows(4)
            .enumerate()
            .find_map(|(offset, magic)| {
                (magic == b"PK\x01\x02" && bytes.get(offset + 46) == Some(&b'0')).then_some(offset)
            })
            .unwrap();
        let local =
            u32::from_le_bytes(bytes[central + 42..central + 46].try_into().unwrap()) as usize;
        bytes[central + 24..central + 28].copy_from_slice(&1u32.to_le_bytes());
        bytes[local + 22..local + 26].copy_from_slice(&1u32.to_le_bytes());
        std::fs::write(&path, bytes).unwrap();
        let mut limits = InspectLimits::default();
        limits.max_zip_entry_bytes = 64;
        assert_limit(&path, &limits, "zip_entry_bytes");
    }
}

#[test]
fn later_frames_and_skippable_frames_cannot_reset_budgets() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("later-frame.apkg");
    let mut frames = zstd::bulk::compress(b"a", 0).unwrap();
    frames.extend(zstd::bulk::compress(&vec![b'a'; 1024], 0).unwrap());
    write_archive(&path, 3, &[frames], CompressionMethod::Stored, false);
    let mut limits = InspectLimits::default();
    limits.max_zstd_window_bytes = 512;
    assert_limit(&path, &limits, "zstd_window_bytes");

    let mut frames = Vec::from(&b"\x50\x2a\x4d\x18"[..]);
    frames.extend(128u32.to_le_bytes());
    frames.extend([0; 128]);
    frames.extend(zstd::bulk::compress(b"a", 0).unwrap());
    write_archive(&path, 3, &[frames], CompressionMethod::Stored, false);
    assert_eq!(
        inspect_apkg(&path).unwrap().observations.media[0]["size"],
        1
    );
    limits = InspectLimits::default();
    limits.max_zip_total_bytes = 100;
    assert_limit(&path, &limits, "zip_total_bytes");
}

#[test]
fn malformed_zip_and_truncated_frames_are_not_complete_observations() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("malformed.apkg");
    let mut frame = zstd::bulk::compress(&vec![b'a'; 1024], 0).unwrap();
    frame.pop();
    write_archive(&path, 3, &[frame], CompressionMethod::Stored, false);
    let report = inspect_apkg(&path).unwrap();
    assert!(report.missing_domains.contains(&"media".into()));
    assert_ne!(report.observation_status, "complete");
    let mut bytes = std::fs::read(&path).unwrap();
    let end = bytes.len() - 22;
    bytes[end + 12..end + 16].copy_from_slice(&1u32.to_le_bytes());
    std::fs::write(&path, bytes).unwrap();
    assert!(inspect_apkg(&path).is_err());
}
