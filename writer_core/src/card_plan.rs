use std::collections::{BTreeMap, BTreeSet};

use authoring_core::{NormalizedIr, NormalizedNote, NormalizedNotetype, NormalizedTemplate};

use crate::apkg::strip_html_preserving_media_filenames;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlannedCard {
    pub template_index: usize,
    pub card_ord: u32,
}

pub fn plan_cards(note: &NormalizedNote, notetype: &NormalizedNotetype) -> Vec<PlannedCard> {
    if notetype.kind == "cloze" {
        return plan_cloze_cards(note, notetype);
    }

    notetype
        .templates
        .iter()
        .enumerate()
        .filter(|(_, template)| template_generates_card(note, notetype, template))
        .map(|(template_index, template)| PlannedCard {
            template_index,
            card_ord: template.ord.unwrap_or(template_index as u32),
        })
        .collect()
}

pub fn count_cards(normalized_ir: &NormalizedIr) -> usize {
    let notetypes = normalized_ir
        .notetypes
        .iter()
        .map(|notetype| (notetype.id.as_str(), notetype))
        .collect::<BTreeMap<_, _>>();

    normalized_ir
        .notes
        .iter()
        .filter_map(|note| {
            notetypes
                .get(note.notetype_id.as_str())
                .map(|notetype| plan_cards(note, notetype).len())
        })
        .sum()
}

fn plan_cloze_cards(note: &NormalizedNote, notetype: &NormalizedNotetype) -> Vec<PlannedCard> {
    let Some((template_index, template)) = notetype.templates.iter().enumerate().next() else {
        return Vec::new();
    };
    let values = cloze_values(note, template);

    scan_cloze_card_ords(values.into_iter().map(String::as_str))
        .0
        .into_iter()
        .map(|card_ord| PlannedCard {
            template_index,
            card_ord,
        })
        .collect()
}

pub fn has_malformed_cloze(note: &NormalizedNote, notetype: &NormalizedNotetype) -> bool {
    if notetype.kind != "cloze" {
        return false;
    }
    let Some(template) = notetype.templates.first() else {
        return false;
    };
    let values = cloze_values(note, template);
    scan_cloze_card_ords(values.into_iter().map(String::as_str)).1
}

fn cloze_values<'a>(note: &'a NormalizedNote, template: &NormalizedTemplate) -> Vec<&'a String> {
    let cloze_fields = cloze_field_names(template);
    if cloze_fields.is_empty() {
        note.fields.values().collect()
    } else {
        cloze_fields
            .iter()
            .filter_map(|field_name| note.fields.get(field_name))
            .collect()
    }
}

fn cloze_field_names(template: &NormalizedTemplate) -> BTreeSet<String> {
    if let Some(requirement) = template
        .generation_requirement
        .as_ref()
        .filter(|requirement| requirement.kind == "cloze")
    {
        return requirement.field_names.iter().cloned().collect();
    }

    template_field_references(&template.question_format, "cloze")
}

fn template_field_references(template: &str, filter: &str) -> BTreeSet<String> {
    let mut fields = BTreeSet::new();
    let prefix = format!("{{{{{filter}:");
    let mut remaining = template;
    while let Some(start) = remaining.find(&prefix) {
        let after_prefix = &remaining[start + prefix.len()..];
        let Some(end) = after_prefix.find("}}") else {
            break;
        };
        let field = after_prefix[..end].trim();
        if !field.is_empty() {
            fields.insert(field.to_string());
        }
        remaining = &after_prefix[end + 2..];
    }
    fields
}

fn scan_cloze_card_ords<'a>(values: impl Iterator<Item = &'a str>) -> (BTreeSet<u32>, bool) {
    let mut ords = BTreeSet::new();
    let mut malformed = false;
    for value in values {
        let mut remaining = value;
        while let Some(start) = remaining.find("{{c") {
            let after_prefix = &remaining[start + 3..];
            let digit_count = after_prefix.bytes().take_while(u8::is_ascii_digit).count();
            if digit_count == 0 {
                if after_prefix.starts_with("::") {
                    malformed = true;
                }
                remaining = &after_prefix[after_prefix.len().min(1)..];
                continue;
            }
            let after_digits = &after_prefix[digit_count..];
            if !after_digits.starts_with("::") {
                malformed = true;
                remaining = after_digits;
                continue;
            }
            let body = &after_digits[2..];
            let Some(close) = body.find("}}") else {
                malformed = true;
                break;
            };
            if body[..close].is_empty() || contains_numbered_cloze_start(&body[..close]) {
                malformed = true;
            }
            match after_prefix[..digit_count]
                .parse::<u32>()
                .ok()
                .and_then(|number| number.checked_sub(1))
            {
                Some(card_ord) => {
                    ords.insert(card_ord);
                }
                None => malformed = true,
            }
            remaining = &body[close + 2..];
        }
    }
    (ords, malformed)
}

fn contains_numbered_cloze_start(value: &str) -> bool {
    let mut remaining = value;
    while let Some(start) = remaining.find("{{c") {
        let after_prefix = &remaining[start + 3..];
        if after_prefix
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_digit)
        {
            return true;
        }
        remaining = &after_prefix[after_prefix.len().min(1)..];
    }
    false
}

fn template_generates_card(
    note: &NormalizedNote,
    notetype: &NormalizedNotetype,
    template: &NormalizedTemplate,
) -> bool {
    let Some(requirement) = template.generation_requirement.as_ref() else {
        return default_template_generates_card(note, notetype, &template.question_format);
    };

    match requirement.kind.as_str() {
        "none" => true,
        "all" => requirement
            .field_names
            .iter()
            .all(|name| note_field_is_nonempty(note, notetype, name)),
        _ => requirement
            .field_names
            .iter()
            .any(|name| note_field_is_nonempty(note, notetype, name)),
    }
}

fn default_template_generates_card(
    note: &NormalizedNote,
    notetype: &NormalizedNotetype,
    source: &str,
) -> bool {
    let mut rendered = String::new();
    let mut cursor = 0;
    let mut active_sections = Vec::new();

    while let Some(relative_open) = source[cursor..].find("{{") {
        let open = cursor + relative_open;
        if active_sections.iter().all(|active| *active) {
            rendered.push_str(&source[cursor..open]);
        }
        let Some(relative_close) = source[open + 2..].find("}}") else {
            break;
        };
        let close = open + 2 + relative_close;
        let expression = source[open + 2..close].trim();

        if let Some(field) = expression.strip_prefix('#') {
            active_sections.push(note_field_is_nonempty(note, notetype, field.trim()));
        } else if let Some(field) = expression.strip_prefix('^') {
            active_sections.push(!note_field_is_nonempty(note, notetype, field.trim()));
        } else if expression.starts_with('/') {
            active_sections.pop();
        } else if !expression.starts_with('!') && active_sections.iter().all(|active| *active) {
            let field = expression.rsplit(':').next().unwrap_or_default().trim();
            if let Some(value) = note.fields.get(field) {
                rendered.push_str(value);
            }
        }
        cursor = close + 2;
    }
    if active_sections.iter().all(|active| *active) {
        rendered.push_str(&source[cursor..]);
    }

    !strip_html_preserving_media_filenames(&rendered)
        .trim()
        .is_empty()
}

fn note_field_is_nonempty(
    note: &NormalizedNote,
    notetype: &NormalizedNotetype,
    field_name: &str,
) -> bool {
    if !notetype
        .fields
        .iter()
        .any(|field| field.name.as_str() == field_name)
    {
        return false;
    }

    note.fields
        .get(field_name)
        .map(|value| {
            !strip_html_preserving_media_filenames(value)
                .trim()
                .is_empty()
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use authoring_core::{NormalizedField, NormalizedNote, NormalizedNotetype, NormalizedTemplate};
    use std::collections::BTreeMap;

    fn normal_card_fixture(front: &str, value: &str) -> (NormalizedNote, NormalizedNotetype) {
        (
            NormalizedNote {
                id: "note".into(),
                notetype_id: "normal".into(),
                deck_name: "Deck".into(),
                fields: BTreeMap::from([("Front".into(), value.into())]),
                tags: Vec::new(),
                mtime_secs: None,
            },
            NormalizedNotetype {
                id: "normal".into(),
                kind: "normal".into(),
                name: "Normal".into(),
                original_stock_kind: None,
                original_id: None,
                fields: vec![NormalizedField {
                    name: "Front".into(),
                    ord: Some(0),
                    config_id: None,
                    tag: None,
                    prevent_deletion: false,
                    sort: true,
                }],
                templates: vec![NormalizedTemplate {
                    name: "Card".into(),
                    ord: Some(0),
                    config_id: None,
                    question_format: front.into(),
                    answer_format: "{{Front}}".into(),
                    browser_question_format: None,
                    browser_answer_format: None,
                    target_deck_name: None,
                    browser_font_name: None,
                    browser_font_size: None,
                    generation_requirement: None,
                }],
                css: String::new(),
                field_metadata: Vec::new(),
            },
        )
    }

    #[test]
    fn anki_default_uses_rendered_front_emptiness() {
        let (empty_note, notetype) = normal_card_fixture("<div>{{Front}}</div>", "");
        assert!(plan_cards(&empty_note, &notetype).is_empty());

        let (filled_note, notetype) = normal_card_fixture("<div>{{Front}}</div>", "answer");
        assert_eq!(plan_cards(&filled_note, &notetype).len(), 1);

        let (static_note, notetype) = normal_card_fixture("Prompt {{Front}}", "");
        assert_eq!(plan_cards(&static_note, &notetype).len(), 1);
    }

    #[test]
    fn cloze_card_ords_are_distinct_sorted_and_zero_based() {
        let (ords, malformed) =
            scan_cloze_card_ords(["{{c2::Spain}} {{c1::Madrid}} {{c1::Europe}}"].into_iter());

        assert_eq!(ords.into_iter().collect::<Vec<_>>(), vec![0, 1]);
        assert!(!malformed);
    }

    #[test]
    fn malformed_and_zero_cloze_markers_are_reported() {
        let (ords, malformed) =
            scan_cloze_card_ords(["{{c0::zero}} {{c1:not-valid}} {{c::missing}}"].into_iter());

        assert!(ords.is_empty());
        assert!(malformed);
    }

    #[test]
    fn missing_cloze_ordinal_is_malformed_by_itself() {
        let (ords, malformed) = scan_cloze_card_ords(["{{c::missing}}"].into_iter());

        assert!(ords.is_empty());
        assert!(malformed);
    }

    #[test]
    fn unrelated_mustache_text_starting_with_c_is_not_cloze() {
        let (ords, malformed) = scan_cloze_card_ords(["{{custom}}"].into_iter());

        assert!(ords.is_empty());
        assert!(!malformed);
    }

    #[test]
    fn unclosed_and_nested_cloze_markers_are_reported() {
        for value in ["{{c1::unclosed", "{{c1::outer {{c2::inner}} body}}"] {
            let (_, malformed) = scan_cloze_card_ords([value].into_iter());
            assert!(malformed, "{value}");
        }
    }
}
