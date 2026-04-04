use std::{error::Error, fmt};

use crate::model::{DiagnosticItem, NormalizationRequest};

pub trait NonceSource {
    fn next_u64(&mut self) -> u64;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityError {
    ReasonCodeRequired,
    ExternalIdRequired,
    UnsupportedOverride(String),
}

impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentityError::ReasonCodeRequired => write!(f, "reason_code required"),
            IdentityError::ExternalIdRequired => write!(f, "external_id required"),
            IdentityError::UnsupportedOverride(mode) => {
                write!(f, "unsupported identity override mode: {mode}")
            }
        }
    }
}

impl Error for IdentityError {}

pub struct DefaultNonceSource;

impl NonceSource for DefaultNonceSource {
    fn next_u64(&mut self) -> u64 {
        rand::random::<u64>()
    }
}

pub fn resolve_identity(
    request: &NormalizationRequest,
    diagnostics: &mut Vec<DiagnosticItem>,
    nonce_source: &mut dyn NonceSource,
) -> anyhow::Result<String> {
    let document_id = request.input.metadata_document_id.trim();

    match request.identity_override_mode.as_deref().map(str::trim) {
        None | Some("") => Ok(format!("det:{document_id}")),
        Some("external") => {
            if !has_value(&request.reason_code) {
                return Err(IdentityError::ReasonCodeRequired.into());
            }

            let external_id = request
                .external_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or(IdentityError::ExternalIdRequired)?;

            Ok(format!("ext:{external_id}"))
        }
        Some("random") => {
            if !has_value(&request.reason_code) {
                return Err(IdentityError::ReasonCodeRequired.into());
            }

            diagnostics.push(DiagnosticItem {
                level: "warning".into(),
                code: "PHASE2.IDENTITY_RANDOM_OVERRIDE".into(),
                summary: "random override disables deterministic identity resolution".into(),
            });

            Ok(format!("rnd:{:016x}", nonce_source.next_u64()))
        }
        Some(other) => Err(IdentityError::UnsupportedOverride(other.to_string()).into()),
    }
}

fn has_value(value: &Option<String>) -> bool {
    value
        .as_deref()
        .map(str::trim)
        .is_some_and(|raw| !raw.is_empty())
}
