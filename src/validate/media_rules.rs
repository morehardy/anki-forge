use std::collections::HashMap;

use crate::domain::media::MediaRef;
use crate::domain::package_spec::PackageSpec;
use crate::validate::diagnostic::Diagnostic;
use crate::validate::mode::ValidationConfig;

pub fn normalize(spec: &mut PackageSpec, mode: ValidationConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut normalized_media = Vec::with_capacity(spec.media.len());

    for (media_index, media) in spec.media.iter().enumerate() {
        let logical_path = format!("media[{media_index}].logical_name");
        let source_path = format!("media[{media_index}].source_path");

        let normalized_name = normalize_media_name(&media.logical_name);
        let normalized_source = media.source_path.trim().to_owned();

        if normalized_name.is_empty() {
            if mode.is_strict() {
                diagnostics.push(Diagnostic::error(
                    logical_path,
                    "media logical name must not be empty",
                ));
            } else {
                diagnostics.push(Diagnostic::warning(logical_path, "empty media name removed"));
            }

            if mode.is_permissive() {
                continue;
            }
        }

        if normalized_source.is_empty() {
            if mode.is_strict() {
                diagnostics.push(Diagnostic::error(
                    source_path,
                    "media source path must not be empty",
                ));
            } else {
                diagnostics.push(Diagnostic::warning(source_path, "empty media source removed"));
            }

            if mode.is_permissive() {
                continue;
            }
        }

        if let Some(first_index) = seen.get(&normalized_name) {
            if mode.is_strict() {
                diagnostics.push(Diagnostic::error(
                    format!("media[{media_index}]"),
                    format!(
                        "duplicate media logical name `{normalized_name}` first seen at media[{first_index}]"
                    ),
                ));
            } else {
                diagnostics.push(Diagnostic::warning(
                    format!("media[{media_index}]"),
                    format!("duplicate media `{normalized_name}` removed"),
                ));
                continue;
            }
        } else {
            seen.insert(normalized_name.clone(), media_index);
        }

        normalized_media.push(MediaRef::new(normalized_name, normalized_source));
    }

    spec.media = normalized_media;
    diagnostics
}

#[must_use]
pub fn normalize_media_name(name: &str) -> String {
    name.trim()
        .replace('\\', "/")
        .split('/')
        .filter_map(|segment| {
            let trimmed = segment.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}
