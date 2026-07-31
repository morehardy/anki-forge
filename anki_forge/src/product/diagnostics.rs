#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub source_path: Option<String>,
    pub byte_offset: Option<usize>,
}

impl ProductDiagnostic {
    pub fn io_image_required(note_id: &str) -> Self {
        Self {
            code: "PHASE5A.IO_IMAGE_REQUIRED",
            message: format!("Image occlusion note '{note_id}' requires a non-empty image."),
            source_path: None,
            byte_offset: None,
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
            source_path: None,
            byte_offset: None,
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
            source_path: None,
            byte_offset: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub source_path: Option<String>,
    pub byte_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductLoweringError {
    pub product_diagnostics: Vec<ProductDiagnostic>,
    pub lowering_diagnostics: Vec<LoweringDiagnostic>,
}

impl ProductLoweringError {
    pub fn code(&self) -> crate::diagnostics::ErrorCode {
        self.product_diagnostics
            .first()
            .map(|diagnostic| crate::diagnostics::ErrorCode::from_code(diagnostic.code))
            .or_else(|| {
                self.lowering_diagnostics
                    .first()
                    .map(|diagnostic| crate::diagnostics::ErrorCode::from_code(diagnostic.code))
            })
            .unwrap_or_else(|| crate::diagnostics::ErrorCode::from_code("PROJECT.LOWER_FAILED"))
    }

    fn message(&self) -> &str {
        self.product_diagnostics
            .first()
            .map(|diagnostic| diagnostic.message.as_str())
            .or_else(|| {
                self.lowering_diagnostics
                    .first()
                    .map(|diagnostic| diagnostic.message.as_str())
            })
            .unwrap_or("product lowering failed")
    }
}

impl std::fmt::Display for ProductLoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

impl std::error::Error for ProductLoweringError {}

impl crate::diagnostics::ErrorCodeExt for ProductLoweringError {
    fn code(&self) -> crate::diagnostics::ErrorCode {
        ProductLoweringError::code(self)
    }
}
