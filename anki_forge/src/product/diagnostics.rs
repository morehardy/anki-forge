#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductDiagnostic {
    pub code: &'static str,
    pub message: String,
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
