use crate::product::ProductDocument;
use std::path::Path;

use super::{
    artifact::PrivateCandidate, identity::PackageIdentity, BuildError, BuildErrorKind as Kind,
    BuildReport,
};
use crate::{
    diagnostics::{Diagnostic, Severity},
    Project,
};

/// Normalize once and pass the native identity plan directly to the writer.
/// The caller owns inspection, comparison and publication of this private file.
pub(super) fn generate(
    project: &Project,
    document: ProductDocument,
    inputs: &Path,
    identity: PackageIdentity,
) -> Result<(PrivateCandidate, BuildReport), BuildError> {
    use crate::writer_core::stream_zip::PackageFingerprint;
    // Native callers inspect the actual candidate before publication and do
    // not expose the low-level writer's whole-package SHA-1.
    generate_with_fingerprint(
        project,
        document,
        inputs,
        identity,
        PackageFingerprint::Omit,
    )
}

fn generate_with_fingerprint(
    project: &Project,
    document: ProductDocument,
    inputs: &Path,
    mut identity: PackageIdentity,
    fingerprint: crate::writer_core::stream_zip::PackageFingerprint,
) -> Result<(PrivateCandidate, BuildReport), BuildError> {
    let media_store = inputs.join("media-store");
    let mut prepared_media = if project.assets.values().next().is_some() {
        let mut prepared =
            crate::prepared_media::PreparedMedia::new_in_with_fingerprint(inputs, fingerprint)
                .map_err(|cause| {
                    generation_error(
                        "BUILD.MEDIA_STAGING_FAILED",
                        cause.into(),
                        &BuildReport::default(),
                    )
                })?;
        for media in project.assets.values() {
            prepared.register_owned(media.clone());
        }
        Some(prepared)
    } else {
        None
    };
    let mut normalized =
        super::normalize::normalize(document, inputs, &media_store, prepared_media.as_mut())
            .map_err(|cause| {
                let diagnostics: Vec<_> = cause
                    .diagnostics
                    .iter()
                    .cloned()
                    .map(Diagnostic::from_backend)
                    .collect();
                let code = diagnostics
                    .iter()
                    .find(|d| d.severity == Severity::Error)
                    .map(|d| d.code.as_str())
                    .unwrap_or("BUILD.NORMALIZE_FAILED");
                let kind = if cause.io_cause.is_some() {
                    Kind::Io
                } else {
                    Kind::Validation
                };
                let mut error = BuildError::new(kind, code, "normalize authored content");
                error.report.diagnostics = diagnostics;
                error.caused_by(cause)
            })?;
    let mut report = BuildReport {
        diagnostics: normalized
            .diagnostics
            .into_iter()
            .map(Diagnostic::from_backend)
            .collect(),
        ..BuildReport::default()
    };
    if normalized.normalized_ir.notes.is_empty() {
        let mut error = BuildError::new(
            Kind::Validation,
            "PROJECT.EMPTY",
            "project contains no notes",
        );
        error.report = Box::new(report);
        return Err(error);
    }
    identity
        .bind(project, &mut normalized.normalized_ir)
        .map_err(|mut error| {
            error.report = Box::new(report.clone());
            error
        })?;
    let cards = identity.bound_card_count(&normalized.normalized_ir);
    report.counts = super::BuildCounts {
        notes: normalized.normalized_ir.notes.len(),
        cards,
        media: normalized.normalized_ir.media_bindings.len(),
    };
    let (policy, context) = crate::runtime::defaults::load_embedded_writer_defaults()
        .map_err(|cause| generation_error("BUILD.RUNTIME_DEFAULTS_FAILED", cause, &report))?;
    let mut target =
        crate::writer_core::BuildArtifactTarget::new(inputs.join("candidate"), "candidate")
            .with_media_store_dir(media_store);
    let ids = identity.model_ids();
    let guids = identity.guid_plan();
    target.native_identity = Some(std::sync::Arc::new(identity));
    target.package_fingerprint = fingerprint;
    target.staging_output = crate::writer_core::staging::StagingOutput::NativeOnly;
    let attempt = crate::writer_core::build::build_with_prepared_media(
        &normalized.normalized_ir,
        &policy,
        &context,
        &target,
        &target,
        Some(&guids),
        Some(&ids),
        prepared_media.as_ref(),
    )
    .map_err(|cause| generation_error("BUILD.WRITER_FAILED", cause, &report))?;
    let result = attempt.result;
    for diagnostic in &result.diagnostics.items {
        report.diagnostics.push(Diagnostic {
            code: diagnostic.code.clone(),
            severity: match diagnostic.level.as_str() {
                "error" => Severity::Error,
                "warning" => Severity::Warning,
                _ => Severity::Info,
            },
            message: diagnostic.summary.clone(),
            source: diagnostic
                .path
                .as_ref()
                .map(|path| crate::diagnostics::SourceLocation {
                    path: path.clone(),
                    byte_range: None,
                }),
            help: None,
        });
    }
    if result.result_status != "success" {
        let code = report
            .diagnostics
            .iter()
            .find(|d| d.severity == Severity::Error)
            .map(|d| d.code.as_str())
            .unwrap_or("BUILD.WRITER_FAILED");
        let kind = if attempt.cause.as_ref().is_some_and(is_io_failure) {
            Kind::Io
        } else if result.result_status == "invalid" {
            Kind::Validation
        } else {
            Kind::Internal
        };
        let mut error = BuildError::new(kind, code, "generate private candidate");
        if let Some(cause) = attempt.cause {
            error = error.caused_by_anyhow(cause);
        }
        error.report = Box::new(report);
        return Err(error);
    }
    let path = result.apkg_ref.as_deref().ok_or_else(|| {
        BuildError::new(
            Kind::Internal,
            "BUILD.ARTIFACT_MISSING",
            "successful writer returned no artifact",
        )
    })?;
    let path = crate::writer_core::artifact_path_from_ref(&target, path)
        .map_err(|cause| generation_error("BUILD.ARTIFACT_MISSING", cause, &report))?;
    let artifact = PrivateCandidate::retain(&path).map_err(|cause| {
        let mut error = BuildError::new(
            Kind::Io,
            "BUILD.WORKSPACE_FAILED",
            "retain private candidate",
        )
        .caused_by(cause);
        error.report = Box::new(report.clone());
        error
    })?;
    Ok((artifact, report))
}

fn is_io_failure(cause: &anyhow::Error) -> bool {
    cause.chain().any(|source| {
        source.is::<std::io::Error>()
            || matches!(
                source
                    .downcast_ref::<rusqlite::Error>()
                    .and_then(rusqlite::Error::sqlite_error_code),
                Some(
                    rusqlite::ErrorCode::PermissionDenied
                        | rusqlite::ErrorCode::ReadOnly
                        | rusqlite::ErrorCode::SystemIoFailure
                        | rusqlite::ErrorCode::DiskFull
                        | rusqlite::ErrorCode::CannotOpen
                        | rusqlite::ErrorCode::FileLockingProtocolFailed
                )
            )
    })
}

fn generation_error(code: &str, cause: anyhow::Error, report: &BuildReport) -> BuildError {
    let kind = if is_io_failure(&cause) {
        Kind::Io
    } else {
        Kind::Internal
    };
    let mut error =
        BuildError::new(kind, code, "generate private candidate").caused_by_anyhow(cause);
    error.report = Box::new(report.clone());
    error
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_and_document(size: Option<usize>) -> (Project, ProductDocument) {
        let mut project = Project::new("dataflow-fingerprint").unwrap();
        project
            .add("one", crate::Note::basic("front", "back"))
            .unwrap();
        let mut media = Vec::new();
        if let Some(size) = size {
            let mut state = 42_u32;
            let bytes: Vec<_> = (0..size)
                .map(|_| {
                    state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    b'!' + ((state >> 24) % 90) as u8
                })
                .collect();
            project
                .add_asset(
                    crate::Media::bytes(bytes, "text/plain")
                        .unwrap()
                        .with_export_name("asset.txt")
                        .unwrap(),
                )
                .unwrap();
            media.push(
                serde_json::json!({"id":"asset.txt", "export_as":"asset.txt",
                "source":{"kind":"file", "path":"assets/asset.txt"}}),
            );
        }
        let document = serde_json::from_value(serde_json::json!({
            "product_document_version":"product-v3", "document_id":"dataflow-fingerprint",
            "note_types":[{"kind":"custom", "id":"model:basic", "note_type_kind":"normal",
                "fields":[{"key":"front", "name":"Front"}, {"key":"back", "name":"Back"}],
                "templates":[{"key":"card", "name":"Card 1", "front":"{{Front}}", "back":"{{Back}}"}]}],
            "notes":[{"kind":"custom", "note_type_id":"model:basic", "stable_id":"one", "deck_name":"Default",
                "fields":{"front":{"kind":"html", "value":"front"}, "back":{"kind":"html", "value":"back"}}}],
            "media":media
        })).unwrap();
        (project, document)
    }

    #[test]
    fn native_fingerprint_modes_keep_bytes_reports_and_full_inspection_identical() {
        use crate::writer_core::stream_zip::PackageFingerprint;
        for size in [None, Some(17), Some(2 * 1024 * 1024 + 17)] {
            let (project, document) = project_and_document(size);
            let identity = PackageIdentity::create(&project).unwrap();
            let mut results = Vec::new();
            for mode in [PackageFingerprint::Compute, PackageFingerprint::Omit] {
                let inputs = tempfile::tempdir().unwrap();
                let (candidate, report) = generate_with_fingerprint(
                    &project,
                    document.clone(),
                    inputs.path(),
                    identity.clone(),
                    mode,
                )
                .unwrap();
                let bytes = std::fs::read(candidate.path()).unwrap();
                let (summary, envelope) = crate::writer_core::inspect::inspect_native_package(
                    candidate.path(),
                    &super::super::InspectLimits::default(),
                )
                .unwrap();
                assert_eq!(summary.notes, 1);
                assert_eq!(summary.cards, 1);
                assert_eq!(summary.media, usize::from(size.is_some()));
                if size.is_some() {
                    let limits = super::super::InspectLimits {
                        max_media_bytes: 0,
                        ..Default::default()
                    };
                    let error = crate::writer_core::inspect::inspect_native_package(
                        candidate.path(),
                        &limits,
                    )
                    .unwrap_err();
                    assert_eq!(error.limit_exceeded().unwrap().resource, "media_bytes");
                }
                let artifact = candidate.into_artifact();
                let path = artifact.path().to_owned();
                drop(artifact);
                assert!(
                    !path.exists(),
                    "last candidate owner must clean its package"
                );
                results.push((
                    bytes,
                    serde_json::to_value(report.snapshot()).unwrap(),
                    summary,
                    serde_json::to_value(envelope).unwrap(),
                ));
                inputs.close().unwrap();
            }
            assert_eq!(results[0], results[1]);
        }
    }

    #[test]
    fn native_fingerprint_modes_preserve_real_writer_failures_and_cleanup() {
        use crate::writer_core::stream_zip::PackageFingerprint;
        for size in [None, Some(2 * 1024 * 1024 + 17)] {
            let (project, document) = project_and_document(size);
            let identity = PackageIdentity::create(&project).unwrap();
            let mut outcomes = Vec::new();
            for mode in [PackageFingerprint::Compute, PackageFingerprint::Omit] {
                let inputs = tempfile::tempdir().unwrap();
                std::fs::create_dir_all(inputs.path().join("candidate/package.apkg")).unwrap();
                let error = generate_with_fingerprint(
                    &project,
                    document.clone(),
                    inputs.path(),
                    identity.clone(),
                    mode,
                )
                .unwrap_err();
                assert!(error.publications().is_empty());
                assert_eq!(error.kind(), Kind::Io);
                outcomes.push((error.kind(), error.code().to_owned()));
                drop(error);
                inputs.close().unwrap();
            }
            assert_eq!(outcomes[0], outcomes[1]);
        }
    }

    #[test]
    fn owned_media_build_does_not_require_input_or_cas_copies() {
        use std::io::Read;
        let inputs = tempfile::tempdir().unwrap();
        let mut project = Project::new("owned-media").unwrap();
        project
            .add("one", crate::Note::basic("front", "back"))
            .unwrap();
        let mut media = Vec::new();
        for index in 0..20 {
            let name = format!("asset-{index:02}.txt");
            project
                .add_asset(
                    crate::Media::bytes(b"owned bytes".to_vec(), "text/plain")
                        .unwrap()
                        .with_export_name(&name)
                        .unwrap(),
                )
                .unwrap();
            media.push(serde_json::json!({"id": name, "export_as": name,
                "source": {"kind": "file", "path": format!("assets/{name}")}}));
        }
        let document: ProductDocument = serde_json::from_value(serde_json::json!({
            "product_document_version": "product-v3", "document_id": "owned-media",
            "note_types": [{"kind": "custom", "id": "model:basic", "note_type_kind": "normal",
                "fields": [{"key": "front", "name": "Front"}, {"key": "back", "name": "Back"}],
                "templates": [{"key": "card", "name": "Card 1", "front": "{{Front}}", "back": "{{Back}}"}]}],
            "notes": [{"kind": "custom", "note_type_id": "model:basic", "stable_id": "one", "deck_name": "Default",
                "fields": {"front": {"kind": "html", "value": "front"}, "back": {"kind": "html", "value": "back"}}}],
            "media": media
        })).unwrap();
        let (artifact, report) = generate(
            &project,
            document.clone(),
            inputs.path(),
            PackageIdentity::create(&project).unwrap(),
        )
        .unwrap();
        assert_eq!(report.counts.media, 20);
        assert!(!inputs.path().join("assets").exists());
        assert!(!inputs.path().join("media-store").exists());
        assert_eq!(
            std::fs::read_dir(inputs.path().join("candidate/staging/media"))
                .unwrap()
                .count(),
            0
        );
        let mut archive =
            zip::ZipArchive::new(std::fs::File::open(artifact.path()).unwrap()).unwrap();
        for index in 0..20 {
            let mut encoded = Vec::new();
            archive
                .by_name(&index.to_string())
                .unwrap()
                .read_to_end(&mut encoded)
                .unwrap();
            assert_eq!(
                zstd::stream::decode_all(encoded.as_slice()).unwrap(),
                b"owned bytes"
            );
        }
    }

    #[test]
    fn native_build_does_not_materialize_a_staging_manifest() {
        let inputs = tempfile::tempdir().unwrap();
        let manifest = inputs.path().join("candidate/staging/manifest.json");
        std::fs::create_dir_all(&manifest).unwrap();
        let (project, document) = project_and_document(None);
        let (candidate, report) = generate(
            &project,
            document,
            inputs.path(),
            PackageIdentity::create(&project).unwrap(),
        )
        .unwrap();
        assert_eq!(report.counts().notes, 1);
        assert!(candidate.path().is_file());
        assert!(manifest.is_dir());
    }

    #[test]
    fn writer_filesystem_failures_keep_native_io_causes() {
        use std::error::Error;

        for (obstruction, is_directory, expected_code) in [
            (
                "candidate/staging",
                false,
                "PHASE3.STAGING_MATERIALIZATION_FAILED",
            ),
            (
                "candidate/.package.apkg.tmp",
                true,
                "PHASE3.APKG_EMISSION_FAILED",
            ),
            (
                "candidate/package.apkg",
                true,
                "PHASE3.APKG_EMISSION_FAILED",
            ),
            (
                "candidate/staging/media/unused.txt",
                true,
                "MEDIA.CAS_OBJECT_COPY_FAILED",
            ),
        ] {
            let inputs = tempfile::tempdir().unwrap();
            let occupied = inputs.path().join(obstruction);
            if is_directory {
                std::fs::create_dir_all(&occupied).unwrap();
            } else {
                std::fs::create_dir_all(occupied.parent().unwrap()).unwrap();
                std::fs::write(&occupied, b"occupied").unwrap();
            }
            std::fs::write(inputs.path().join("unused.txt"), b"media bytes").unwrap();
            let mut project = Project::new("writer-io").unwrap();
            project
                .add("one", crate::Note::basic("front", "back"))
                .unwrap();
            let document: ProductDocument = serde_json::from_value(serde_json::json!({
                "product_document_version": "product-v3", "document_id": "writer-io",
                "note_types": [{"kind": "custom", "id": "model:basic", "note_type_kind": "normal",
                    "fields": [{"key": "front", "name": "Front"}, {"key": "back", "name": "Back"}],
                    "templates": [{"key": "card", "name": "Card 1", "front": "{{addon:Front}}", "back": "{{Back}}"}]}],
                "notes": [{"kind": "custom", "note_type_id": "model:basic", "stable_id": "one", "deck_name": "Default",
                    "fields": {"front": {"kind": "html", "value": "front"}, "back": {"kind": "html", "value": "back"}}}],
                "media": [{"id": "unused", "export_as": "unused.txt", "source": {"kind": "file", "path": "unused.txt"}}]
            })).unwrap();
            let error = generate(
                &project,
                document.clone(),
                inputs.path(),
                PackageIdentity::create(&project).unwrap(),
            )
            .unwrap_err();
            assert_eq!(error.code(), expected_code, "{obstruction}");
            assert_eq!(error.kind(), Kind::Io, "{obstruction}");
            assert_eq!(error.report().counts().notes, 1);
            assert!(error
                .report()
                .diagnostics()
                .iter()
                .any(|d| d.code == "TEMPLATE.FILTER_UNKNOWN"));
            assert!(error
                .report()
                .diagnostics()
                .iter()
                .any(|d| d.code == expected_code && d.source.is_some()));
            assert!(error.publications().is_empty());
            let mut cause = error.source();
            let original = loop {
                let next = cause.expect("writer filesystem cause must be retained");
                if let Some(io) = next.downcast_ref::<std::io::Error>() {
                    break io;
                }
                cause = next.source();
            };
            assert!(original.raw_os_error().is_some());
            let snapshot = serde_json::to_value(error.snapshot()).unwrap();
            assert_eq!(snapshot["result"]["kind"], "io");
            assert!(snapshot["result"]["causes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str() == Some(original.to_string().as_str())));
        }
    }

    #[test]
    fn normalization_io_failure_keeps_original_filesystem_cause() {
        use std::error::Error;

        let inputs = tempfile::tempdir().unwrap();
        std::fs::write(inputs.path().join("unused.txt"), b"unused text").unwrap();
        // This is an actual filesystem failure, independent of user permissions.
        std::fs::write(inputs.path().join("media-store"), b"occupied by a file").unwrap();
        let mut project = Project::new("normalization-io").unwrap();
        project
            .add("one", crate::Note::basic("front", "back"))
            .unwrap();
        let document: ProductDocument = serde_json::from_value(serde_json::json!({
            "product_document_version": "product-v3", "document_id": "normalization-io",
            "note_types": [{"kind": "custom", "id": "model:basic", "note_type_kind": "normal",
                "fields": [{"key": "front", "name": "Front"}, {"key": "back", "name": "Back"}],
                "templates": [{"key": "card", "name": "Card", "front": "{{addon:Front}}", "back": "{{Back}}"}]}],
            "notes": [{"kind": "custom", "note_type_id": "model:basic", "stable_id": "one", "deck_name": "Default",
                "fields": {"front": {"kind": "html", "value": "front"}, "back": {"kind": "html", "value": "back"}}}],
            "media": [{"id": "unused", "export_as": "unused.txt", "source": {"kind": "file", "path": "unused.txt"}}]
        })).unwrap();
        let identity = PackageIdentity::create(&project).unwrap();
        let error = generate(&project, document.clone(), inputs.path(), identity).unwrap_err();
        assert_eq!(error.kind(), Kind::Io);
        assert_eq!(error.code(), "MEDIA.CAS_WRITE_FAILED");
        assert!(error
            .report()
            .diagnostics()
            .iter()
            .any(|item| item.code == error.code()));
        assert!(error
            .report()
            .diagnostics()
            .iter()
            .any(|item| item.code == "TEMPLATE.FILTER_UNKNOWN"));
        let mut source = error.source();
        let io_error = loop {
            let cause = source.expect("original filesystem error in the source chain");
            if let Some(error) = cause.downcast_ref::<std::io::Error>() {
                break error;
            }
            source = cause.source();
        };
        assert!(
            io_error.raw_os_error().is_some(),
            "preserve the OS error, not its Display text"
        );
        assert!(error.publications().is_empty());
    }

    #[test]
    fn native_media_io_failures_keep_exact_error_and_baseline_observations() {
        use crate::authoring_core::media_io::io_failure::{self, Point};
        use std::{error::Error, io, sync::Arc};

        #[derive(Debug)]
        struct Marker(Arc<()>);
        impl std::fmt::Display for Marker {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("original scoped media IO failure")
            }
        }
        impl Error for Marker {}

        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("output.apkg");
        let mut project = Project::new("media-io-observations").unwrap();
        project
            .add("one", crate::Note::basic("front", "back"))
            .unwrap();
        let baseline = project.build(crate::BuildOptions::temporary()).unwrap();
        project
            .add_asset(crate::Media::bytes(b"owned text".to_vec(), "text/plain").unwrap())
            .unwrap();
        for (point, code) in [
            (Point::Read, "MEDIA.SOURCE_READ_FAILED"),
            (Point::Write, "MEDIA.CAS_WRITE_FAILED"),
        ] {
            std::fs::write(&destination, b"previous complete output").unwrap();
            let token = Arc::new(());
            let cause = io::Error::other(Marker(Arc::clone(&token)));
            let error = io_failure::during(point, cause, || {
                project.build(
                    crate::BuildOptions::to(&destination).update_from(baseline.artifact().path()),
                )
            })
            .unwrap_err();
            assert_eq!(error.kind(), Kind::Io);
            assert_eq!(error.code(), code);
            assert_eq!(error.report().baseline_counts().unwrap().notes, 1);
            assert!(error.report().comparison().is_none());
            assert!(error.publications().is_empty());
            assert_eq!(
                std::fs::read(&destination).unwrap(),
                b"previous complete output"
            );
            assert!(error
                .report()
                .diagnostics()
                .iter()
                .any(|item| item.code == code && item.source.is_some()));
            let mut source = error.source();
            let original = loop {
                let cause = source.expect("original media IO error in the source chain");
                if let Some(error) = cause.downcast_ref::<io::Error>() {
                    break error;
                }
                source = cause.source();
            };
            let marker = original
                .get_ref()
                .unwrap()
                .downcast_ref::<Marker>()
                .unwrap();
            assert!(
                Arc::ptr_eq(&token, &marker.0),
                "the exact error payload must survive, without reconstruction"
            );
            let snapshot = serde_json::to_value(error.snapshot()).unwrap();
            assert_eq!(snapshot["result"]["kind"], "io");
            assert!(snapshot["result"]["causes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str() == Some("original scoped media IO failure")));
            assert_eq!(snapshot["report"]["baseline_counts"]["notes"], 1);
        }
        // A scoped fault cannot contaminate a later operation.
        let output = project
            .build(crate::BuildOptions::to(&destination).update_from(baseline.artifact().path()))
            .unwrap();
        assert_eq!(output.report().counts().media, 1);
        assert!(output.artifact().path().is_file());
    }

    #[test]
    fn identity_binding_failure_retains_normalization_warnings() {
        let inputs = tempfile::tempdir().unwrap();
        std::fs::write(inputs.path().join("unused.txt"), b"unused text").unwrap();
        let mut project = Project::new("identity-warning").unwrap();
        project
            .add("one", crate::Note::basic("front", "back"))
            .unwrap();
        let document: ProductDocument = serde_json::from_value(serde_json::json!({
            "product_document_version": "product-v3", "document_id": "identity-warning",
            "note_types": [{"kind": "custom", "id": "model:basic", "note_type_kind": "normal",
                "fields": [{"key": "front", "name": "Front"}, {"key": "back", "name": "Back"}],
                "templates": [{"key": "card", "name": "Card", "front": "{{Front}}", "back": "{{Back}}"}]}],
            "notes": [{"kind": "custom", "note_type_id": "model:basic", "stable_id": "one", "deck_name": "Default",
                "fields": {"front": {"kind": "html", "value": "front"}, "back": {"kind": "html", "value": "back"}}}],
            "media": [{"id": "unused", "export_as": "unused.txt", "source": {"kind": "file", "path": "unused.txt"}}]
        })).unwrap();
        let mut identity = PackageIdentity::create(&project).unwrap();
        identity.models.clear();
        let error = generate(&project, document.clone(), inputs.path(), identity).unwrap_err();
        assert!(error
            .report()
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == "MEDIA.UNUSED_BINDING"));
        assert_eq!(error.kind(), Kind::Internal);
    }

    #[test]
    fn sqlite_disk_full_keeps_the_native_database_cause() {
        use std::error::Error;
        let root = tempfile::tempdir().unwrap();
        let database = rusqlite::Connection::open(root.path().join("limited.sqlite")).unwrap();
        database.pragma_update(None, "max_page_count", 1).unwrap();
        let cause = database
            .execute_batch("CREATE TABLE notes(value TEXT)")
            .unwrap_err();
        assert_eq!(
            cause.sqlite_error_code(),
            Some(rusqlite::ErrorCode::DiskFull)
        );
        let error = generation_error(
            "PHASE3.APKG_EMISSION_FAILED",
            anyhow::Error::new(cause).context("create collection"),
            &BuildReport::default(),
        );
        assert_eq!(error.kind(), Kind::Io);
        let original = error
            .source()
            .unwrap()
            .source()
            .unwrap()
            .downcast_ref::<rusqlite::Error>()
            .unwrap();
        assert_eq!(
            original.sqlite_error_code(),
            Some(rusqlite::ErrorCode::DiskFull)
        );
        assert!(error.publications().is_empty());
    }

    #[test]
    fn writer_io_failures_preserve_the_real_cause_and_observations() {
        let report = BuildReport {
            counts: super::super::BuildCounts {
                notes: 7,
                cards: 9,
                media: 1,
            },
            ..BuildReport::default()
        };
        let cause = anyhow::Error::new(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "read denied",
        ))
        .context("open media store");
        let error = generation_error("BUILD.WRITER_FAILED", cause, &report);
        assert_eq!(error.kind(), Kind::Io);
        assert_eq!(error.report().counts().notes, 7);
        let snapshot = serde_json::to_value(error.snapshot()).unwrap();
        assert!(snapshot["result"]["causes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cause| cause
                .as_str()
                .is_some_and(|text| text.contains("read denied"))));
    }
}
