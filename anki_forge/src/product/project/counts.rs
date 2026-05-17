use std::collections::{BTreeMap, BTreeSet};

use crate::build::InspectSummary;

pub(super) fn inspect_metadata_count(report: &crate::InspectReport, key: &str) -> usize {
    report
        .observations
        .metadata
        .iter()
        .find_map(|value| value.get(key).and_then(serde_json::Value::as_u64))
        .unwrap_or(0) as usize
}

pub(super) fn card_count_from_inspect_or_fallback(
    inspect: Option<&InspectSummary>,
    normalized: &authoring_core::NormalizedIr,
) -> usize {
    inspect
        .map(|summary| summary.cards)
        .unwrap_or_else(|| count_phase1_cards_without_inspect(normalized))
}

pub(super) fn count_phase1_cards_without_inspect(
    normalized: &authoring_core::NormalizedIr,
) -> usize {
    let templates_by_notetype = normalized
        .notetypes
        .iter()
        .map(|notetype| {
            let template_count = if notetype.kind == "cloze" {
                0
            } else {
                notetype
                    .templates
                    .iter()
                    .filter(|template| {
                        !template.question_format.trim().is_empty()
                            && !template.answer_format.trim().is_empty()
                    })
                    .count()
            };
            (
                notetype.id.as_str(),
                (notetype.kind.as_str(), template_count),
            )
        })
        .collect::<BTreeMap<_, _>>();
    normalized
        .notes
        .iter()
        .map(|note| {
            let Some((kind, template_count)) = templates_by_notetype.get(note.notetype_id.as_str())
            else {
                return 0;
            };
            if *kind == "cloze" {
                distinct_cloze_ords(note.fields.values().map(String::as_str))
            } else {
                *template_count
            }
        })
        .sum()
}

fn distinct_cloze_ords<'a>(fields: impl Iterator<Item = &'a str>) -> usize {
    let mut ords = BTreeSet::new();
    for value in fields {
        for part in value.split("{{c").skip(1) {
            let end = part
                .find(|ch: char| !ch.is_ascii_digit())
                .unwrap_or(part.len());
            if end > 0 {
                ords.insert(&part[..end]);
            }
        }
    }
    ords.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalized_with_one_fallback_card() -> authoring_core::NormalizedIr {
        let mut fields = BTreeMap::new();
        fields.insert("Front".into(), "front".into());
        fields.insert("Back".into(), "back".into());

        authoring_core::NormalizedIr {
            kind: "normalized-ir".into(),
            schema_version: "0.1.0".into(),
            document_id: "doc".into(),
            resolved_identity: "doc".into(),
            notetypes: vec![authoring_core::NormalizedNotetype {
                id: "basic".into(),
                kind: "normal".into(),
                name: "Basic".into(),
                original_stock_kind: None,
                original_id: None,
                fields: Vec::new(),
                templates: vec![authoring_core::NormalizedTemplate {
                    name: "Card 1".into(),
                    ord: Some(0),
                    config_id: None,
                    question_format: "{{Front}}".into(),
                    answer_format: "{{Back}}".into(),
                    browser_question_format: None,
                    browser_answer_format: None,
                    target_deck_name: None,
                    browser_font_name: None,
                    browser_font_size: None,
                }],
                css: String::new(),
                field_metadata: Vec::new(),
            }],
            notes: vec![authoring_core::NormalizedNote {
                id: "n1".into(),
                notetype_id: "basic".into(),
                deck_name: "Deck".into(),
                fields,
                tags: Vec::new(),
                mtime_secs: None,
            }],
            media_objects: Vec::new(),
            media_bindings: Vec::new(),
            media_references: Vec::new(),
        }
    }

    #[test]
    fn card_count_prefers_zero_card_inspect_result_over_fallback() {
        let normalized = normalized_with_one_fallback_card();
        let inspect = InspectSummary {
            cards: 0,
            ..InspectSummary::default()
        };

        assert_eq!(
            card_count_from_inspect_or_fallback(Some(&inspect), &normalized),
            0
        );
        assert_eq!(card_count_from_inspect_or_fallback(None, &normalized), 1);
    }
}
