//! Private lowering and normalization for native Project builds.
use crate::{
    authoring_core::{MediaPolicy, NormalizationRequest, NormalizeOptions, NormalizedIr},
    diagnostics_backend::{Diagnostic, DiagnosticCode, Severity, SourcePath},
    product::{LoweringDiagnostic, ProductDiagnostic, ProductDocument, ProductLoweringError},
};
use std::path::Path;

pub(super) struct NormalizeOutput {
    pub normalized_ir: NormalizedIr,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug)]
pub(super) struct NormalizeError {
    message: String,
    pub diagnostics: Vec<Diagnostic>,
    pub io_cause: Option<std::io::Error>,
}

impl std::fmt::Display for NormalizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        for diagnostic in &self.diagnostics {
            write!(f, ": {}", diagnostic.code)?;
        }
        Ok(())
    }
}
impl std::error::Error for NormalizeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.io_cause.as_ref().map(|cause| cause as _)
    }
}

pub(super) fn normalize(
    document: &ProductDocument,
    base_dir: &Path,
    media_store_dir: &Path,
) -> Result<NormalizeOutput, NormalizeError> {
    let lowering = document.lower().map_err(|error| NormalizeError {
        message: "lower authored content".into(),
        diagnostics: map_product_lowering_error(&error),
        io_cause: None,
    })?;
    let mut diagnostics = map_product_diagnostics(lowering.product_diagnostics);
    diagnostics.extend(map_lowering_diagnostics(lowering.lowering_diagnostics));
    let source_map = lowering.source_map;
    let mut io_cause = None;
    let result = crate::authoring_core::normalize::normalize_with_prepared_media(
        NormalizationRequest::new(lowering.authoring_document),
        NormalizeOptions {
            base_dir: base_dir.to_owned(),
            media_store_dir: media_store_dir.to_owned(),
            media_policy: MediaPolicy::default_strict(),
        },
        None,
        &mut io_cause,
    );
    diagnostics.extend(result.diagnostics.items.into_iter().map(|item| {
        let source = item
            .path
            .as_deref()
            .and_then(|path| source_map.source_for_diagnostic_path(path))
            .map(SourcePath::new);
        let help = diagnostic_help(&item.code).map(str::to_owned);
        Diagnostic {
            code: DiagnosticCode::new(item.code),
            severity: match item.level.as_str() {
                "error" => Severity::Error,
                "warning" => Severity::Warning,
                _ => Severity::Info,
            },
            message: item.summary.replace("sniffed MIME", "observed MIME"),
            domain: None,
            stage: None,
            source,
            help,
        }
    }));
    if result.result_status != "success"
        || diagnostics.iter().any(|d| d.severity == Severity::Error)
    {
        return Err(NormalizeError {
            message: "normalization rejected authored content".into(),
            diagnostics,
            io_cause,
        });
    }
    let normalized_ir = result.normalized_ir.ok_or_else(|| NormalizeError {
        message: "normalization did not produce normalized_ir".into(),
        diagnostics: diagnostics.clone(),
        io_cause,
    })?;
    Ok(NormalizeOutput {
        normalized_ir,
        diagnostics,
    })
}

fn diagnostic_help(code: &str) -> Option<&'static str> {
    match code {
        "MEDIA.MISSING_REFERENCE" => Some("Include this media with Content::image, Content::sound, NoteTypeBuilder::asset, or Project::add_asset, using the same exported filename."),
        "MEDIA.UNSAFE_REFERENCE" => Some("Use a bare local filename for packaged media, without paths, URL escapes, or unsafe characters."),
        "MEDIA.UNUSED_BINDING" => Some("This explicit asset is retained in the package. Reference it from a note, template, or CSS, or remove it if unwanted."),
        "MEDIA.DECLARED_MIME_MISMATCH" => Some("Choose an export filename and declared MIME that match the media bytes."),
        "MEDIA.DUPLICATE_FILENAME_CONFLICT" => Some("Assign distinct export names with Media::with_export_name to assets with different bytes."),
        "MEDIA.UNKNOWN_MIME" => Some("Choose an export filename with a known extension or provide bytes with a recognized media format."),
        "MEDIA.CAS_WRITE_FAILED" => Some("Check temporary workspace permissions and disk space, then retry the build."),
        _ => None,
    }
}

fn map_product_lowering_error(error: &ProductLoweringError) -> Vec<Diagnostic> {
    map_product_diagnostics(error.product_diagnostics.clone())
        .into_iter()
        .chain(map_lowering_diagnostics(error.lowering_diagnostics.clone()))
        .collect()
}

fn map_product_diagnostics(diagnostics: Vec<ProductDiagnostic>) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| Diagnostic {
            code: DiagnosticCode::new(diagnostic.code),
            severity: Severity::Error,
            domain: None,
            stage: None,
            message: diagnostic.message,
            source: Some(SourcePath::new(
                diagnostic
                    .source_path
                    .unwrap_or_else(|| "project.lower".to_string()),
            )),
            help: diagnostic
                .byte_offset
                .map(|offset| format!("fix the template expression near byte offset {offset}")),
        })
        .collect()
}

fn map_lowering_diagnostics(diagnostics: Vec<LoweringDiagnostic>) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| Diagnostic {
            code: DiagnosticCode::new(diagnostic.code),
            severity: lowering_diagnostic_severity(diagnostic.code),
            domain: None,
            stage: None,
            message: diagnostic.message,
            source: Some(SourcePath::new(
                diagnostic
                    .source_path
                    .unwrap_or_else(|| "project.lower".to_string()),
            )),
            help: diagnostic
                .byte_offset
                .map(|offset| format!("fix the template expression near byte offset {offset}")),
        })
        .collect()
}

fn lowering_diagnostic_severity(code: &str) -> Severity {
    match code {
        "PHASE5A.FONT_BINDING_UNKNOWN_NOTETYPE" | "PRODUCT.MEDIA_HELPER_REFERENCE_UNREGISTERED" => {
            Severity::Error
        }
        _ => Severity::Warning,
    }
}
