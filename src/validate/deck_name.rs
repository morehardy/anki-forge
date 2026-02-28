use crate::domain::package_spec::PackageSpec;
use crate::validate::diagnostic::Diagnostic;
use crate::validate::mode::ValidationConfig;

pub fn normalize(spec: &mut PackageSpec, mode: ValidationConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (deck_index, deck) in spec.decks.iter_mut().enumerate() {
        let path = format!("decks[{deck_index}].name");
        let (normalized, had_empty_segment) = normalize_deck_name(&deck.name);

        if had_empty_segment {
            if mode.is_strict() {
                diagnostics.push(Diagnostic::error(
                    path.clone(),
                    "deck name contains an empty hierarchy segment",
                ));
            } else {
                diagnostics.push(Diagnostic::warning(
                    path.clone(),
                    "removed empty hierarchy segment from deck name",
                ));
            }
        }

        if normalized.is_empty() {
            if mode.is_permissive() {
                diagnostics.push(Diagnostic::warning(
                    path,
                    "deck name was empty and was normalized to `Default`",
                ));
                deck.name = String::from("Default");
            } else {
                diagnostics.push(Diagnostic::error(path, "deck name must not be empty"));
                deck.name = normalized;
            }
        } else {
            deck.name = normalized;
        }
    }

    diagnostics
}

#[must_use]
pub fn normalize_deck_name(name: &str) -> (String, bool) {
    let mut had_empty_segment = false;
    let mut normalized_segments = Vec::new();

    for segment in name.split("::") {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            had_empty_segment = true;
            continue;
        }
        normalized_segments.push(trimmed);
    }

    (normalized_segments.join("::"), had_empty_segment)
}

#[cfg(test)]
mod tests {
    use super::normalize_deck_name;

    #[test]
    fn deck_name_normalization_is_idempotent_for_samples() {
        let samples = [
            "Default",
            "  Biology  ",
            "Parent:: Child",
            " Parent ::  Child :: Grandchild ",
            "Parent::::Child",
            "::Leading::Segment",
            "Trailing::",
            "  ",
        ];

        for sample in samples {
            let (once, _) = normalize_deck_name(sample);
            let (twice, _) = normalize_deck_name(&once);
            assert_eq!(once, twice, "sample `{sample}` was not idempotent");
        }
    }
}
