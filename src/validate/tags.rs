use crate::domain::package_spec::PackageSpec;
use crate::validate::diagnostic::Diagnostic;
use crate::validate::mode::ValidationConfig;

pub fn normalize(spec: &mut PackageSpec, mode: ValidationConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (note_index, note) in spec.notes.iter_mut().enumerate() {
        let mut normalized_tags = Vec::new();

        for (tag_index, raw_tag) in note.tags.iter().enumerate() {
            let path = format!("notes[{note_index}].tags[{tag_index}]");
            let trimmed = raw_tag.trim();

            if trimmed.is_empty() {
                if mode.is_strict() {
                    diagnostics.push(Diagnostic::error(path, "tag must not be empty"));
                } else {
                    diagnostics.push(Diagnostic::warning(path, "empty tag removed"));
                }
                continue;
            }

            for token in trimmed.split_whitespace() {
                normalized_tags.push(token.to_ascii_lowercase());
            }
        }

        normalized_tags.sort();
        normalized_tags.dedup();
        note.tags = normalized_tags;
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    fn normalize_tag_vec(tags: &[&str]) -> Vec<String> {
        let mut normalized = tags
            .iter()
            .flat_map(|tag| tag.trim().split_whitespace().map(str::to_ascii_lowercase))
            .collect::<Vec<String>>();
        normalized.sort();
        normalized.dedup();
        normalized
    }

    #[test]
    fn tag_normalization_is_idempotent_for_samples() {
        let samples = vec![
            vec!["TagA", "tagb", "tagA"],
            vec!["  spaced   out  ", "MIXED Case"],
            vec!["", "   ", "one"],
            vec!["alpha beta", "beta gamma"],
        ];

        for sample in samples {
            let once = normalize_tag_vec(&sample);
            let once_refs = once.iter().map(String::as_str).collect::<Vec<_>>();
            let twice = normalize_tag_vec(&once_refs);
            assert_eq!(once, twice);
        }
    }
}
