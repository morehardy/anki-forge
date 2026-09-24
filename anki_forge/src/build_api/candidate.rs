use std::path::Path;

use super::{
    identity::PackageIdentity, ApkgArtifact, BuildError, BuildErrorKind as Kind, BuildReport,
};
use crate::{
    diagnostics::{Diagnostic, Severity},
    product::ProductDocument,
    Project,
};

/// Normalize once and pass the native identity plan directly to the writer.
/// The caller owns inspection, comparison and publication of this private file.
pub(super) fn generate(
    project: &Project,
    document: &ProductDocument,
    inputs: &Path,
    mut identity: PackageIdentity,
) -> Result<(ApkgArtifact, BuildReport), BuildError> {
    let media_store = inputs.join("media-store");
    let mut normalized =
        super::normalize::normalize(document, inputs, &media_store).map_err(|cause| {
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
    let models: std::collections::BTreeMap<_, _> = normalized
        .normalized_ir
        .notetypes
        .iter()
        .map(|m| (&m.id, m))
        .collect();
    report.counts = super::BuildCounts {
        notes: normalized.normalized_ir.notes.len(),
        cards: normalized
            .normalized_ir
            .notes
            .iter()
            .map(|note| {
                crate::writer_core::card_plan::plan_cards(note, models[&note.notetype_id]).len()
            })
            .sum(),
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
    let result = crate::writer_core::build::build_with_identity_plan(
        &normalized.normalized_ir,
        &policy,
        &context,
        &target,
        &target,
        Some(&guids),
        Some(&ids),
    )
    .map_err(|cause| generation_error("BUILD.WRITER_FAILED", cause, &report))?;
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
        let mut error = BuildError::new(Kind::Validation, code, "writer rejected authored content");
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
    let artifact = ApkgArtifact::temporary_from_candidate(&path).map_err(|cause| {
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

fn generation_error(code: &str, cause: anyhow::Error, report: &BuildReport) -> BuildError {
    let kind = if cause.chain().any(|source| source.is::<std::io::Error>()) {
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
        let error = generate(&project, &document, inputs.path(), identity).unwrap_err();
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
            (Point::Sync, "MEDIA.CAS_WRITE_FAILED"),
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
        let error = generate(&project, &document, inputs.path(), identity).unwrap_err();
        assert!(error
            .report()
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == "MEDIA.UNUSED_BINDING"));
        assert_eq!(error.kind(), Kind::Internal);
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
