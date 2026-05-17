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

    pub fn duplicate_field_key(
        note_type_id: &str,
        key: &str,
        first_field: &str,
        duplicate_field: &str,
    ) -> Self {
        Self {
            code: "NOTETYPE.FIELD_KEY_DUPLICATE",
            message: format!(
                "custom note type '{note_type_id}' uses field key '{key}' for both '{first_field}' and '{duplicate_field}'"
            ),
        }
    }

    pub fn duplicate_template_key(
        note_type_id: &str,
        key: &str,
        first_template: &str,
        duplicate_template: &str,
    ) -> Self {
        Self {
            code: "NOTETYPE.TEMPLATE_KEY_DUPLICATE",
            message: format!(
                "custom note type '{note_type_id}' uses template key '{key}' for both '{first_template}' and '{duplicate_template}'"
            ),
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
