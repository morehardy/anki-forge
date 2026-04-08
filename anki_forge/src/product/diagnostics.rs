#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductDiagnostic {
    pub code: &'static str,
    pub message: String,
}

impl ProductDiagnostic {
    pub fn io_image_required(note_id: &str) -> Self {
        Self {
            code: "PHASE5A.IO_IMAGE_REQUIRED",
            message: format!("Image occlusion note '{note_id}' requires a non-empty image."),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringDiagnostic {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductLoweringError {
    pub product_diagnostics: Vec<ProductDiagnostic>,
    pub lowering_diagnostics: Vec<LoweringDiagnostic>,
}
