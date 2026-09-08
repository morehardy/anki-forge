use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use serde_json::Value;

use crate::writer_core::canonical_json::FilteredValue;
use crate::writer_core::model::{DiffChange, DiffReport, InspectReport};

pub fn diff_reports(left: &InspectReport, right: &InspectReport) -> Result<DiffReport> {
    let mut uncompared_domains = BTreeSet::new();
    let mut comparison_limitations = BTreeSet::new();
    let mut changes = vec![];

    let mut comparison_status = compare_status(left, right);

    for (side, report) in [("left", left), ("right", right)] {
        for domain in &report.missing_domains {
            uncompared_domains.insert(domain.clone());
            comparison_limitations.insert(format!("{side} report missing {domain} domain"));
        }
    }
    if left.observation_model_version != right.observation_model_version {
        comparison_limitations.insert("observation model versions differ".into());
        if comparison_status == "complete" {
            comparison_status = "partial".into();
        }
    }
    for ((domain, left_values), (_, right_values)) in left
        .observations
        .domains()
        .into_iter()
        .zip(right.observations.domains())
    {
        if uncompared_domains.contains(domain) {
            continue;
        }

        let left_entries = domain_entries(left_values);
        let right_entries = domain_entries(right_values);
        let mut selectors = BTreeSet::new();
        selectors.extend(left_entries.keys().copied());
        selectors.extend(right_entries.keys().copied());

        for selector in selectors {
            if should_skip_selector(domain, selector, left, right) {
                continue;
            }
            match (left_entries.get(&selector), right_entries.get(&selector)) {
                (Some(left_entry), Some(right_entry)) => {
                    if entry_payload(domain, left_entry)? != entry_payload(domain, right_entry)? {
                        changes.push(change_for_modified(
                            domain,
                            selector,
                            left_entry,
                            right_entry,
                        )?);
                    }
                }
                (Some(left_entry), None) => {
                    changes.push(change_for_removed(domain, selector, left_entry)?);
                }
                (None, Some(right_entry)) => {
                    changes.push(change_for_added(domain, selector, right_entry)?);
                }
                (None, None) => {}
            }
        }
    }

    if comparison_status == "complete" && !uncompared_domains.is_empty() {
        comparison_status = if has_unavailable(left, right) {
            "unavailable".into()
        } else {
            "partial".into()
        };
    }

    let summary = if changes.is_empty() && comparison_status != "complete" {
        "no changes detected in compared domains; comparison is incomplete".into()
    } else if changes.is_empty() {
        "no compatibility-significant changes".into()
    } else {
        format!("{} change(s) detected", changes.len())
    };

    Ok(DiffReport {
        kind: "diff-report".into(),
        comparison_status,
        left_fingerprint: left.artifact_fingerprint.clone(),
        right_fingerprint: right.artifact_fingerprint.clone(),
        left_observation_model_version: left.observation_model_version.clone(),
        right_observation_model_version: right.observation_model_version.clone(),
        summary,
        uncompared_domains: uncompared_domains.into_iter().collect(),
        comparison_limitations: comparison_limitations.into_iter().collect(),
        changes,
    })
}

fn should_skip_selector(
    domain: &str,
    selector: &str,
    left: &InspectReport,
    right: &InspectReport,
) -> bool {
    domain == "references"
        && selector.starts_with("media-ref[")
        && (left.source_kind == "apkg" || right.source_kind == "apkg")
}

fn has_unavailable(left: &InspectReport, right: &InspectReport) -> bool {
    left.observation_status == "unavailable" || right.observation_status == "unavailable"
}

fn compare_status(left: &InspectReport, right: &InspectReport) -> String {
    if has_unavailable(left, right) {
        "unavailable".into()
    } else if left.observation_status == "complete" && right.observation_status == "complete" {
        "complete".into()
    } else {
        "partial".into()
    }
}

fn domain_entries(values: &[Value]) -> BTreeMap<&str, &Value> {
    let mut entries = BTreeMap::new();
    for value in values {
        let Some(selector) = value.get("selector").and_then(Value::as_str) else {
            continue;
        };
        entries.insert(selector, value);
    }
    entries
}

fn entry_payload(domain: &str, value: &Value) -> Result<String> {
    let excluded: &'static [&'static str] = if domain == "media" {
        &[
            "selector",
            "evidence_refs",
            "binding_id",
            "object_id",
            "object_ref",
        ]
    } else {
        &["selector", "evidence_refs"]
    };
    Ok(serde_json::to_string(&FilteredValue { value, excluded })?)
}

#[cfg(test)]
fn strip_non_semantic_fields(domain: &str, value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut map = map.clone();
            map.remove("selector");
            map.remove("evidence_refs");
            if domain == "media" {
                map.remove("binding_id");
                map.remove("object_id");
                map.remove("object_ref");
            }
            Value::Object(
                map.into_iter()
                    .map(|(key, value)| (key, strip_non_semantic_fields(domain, &value)))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|value| strip_non_semantic_fields(domain, value))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn change_for_modified(
    domain: &str,
    selector: &str,
    left: &Value,
    right: &Value,
) -> Result<DiffChange> {
    Ok(DiffChange {
        category: "modified".into(),
        domain: domain.into(),
        severity: severity_for_domain(domain).into(),
        selector: selector.into(),
        message: format!("{selector} changed"),
        compatibility_hint: compatibility_hint(domain),
        evidence_refs: merge_evidence_refs(left, right),
    })
}

fn change_for_added(domain: &str, selector: &str, right: &Value) -> Result<DiffChange> {
    Ok(DiffChange {
        category: "added".into(),
        domain: domain.into(),
        severity: severity_for_domain(domain).into(),
        selector: selector.into(),
        message: format!("{selector} was added"),
        compatibility_hint: compatibility_hint(domain),
        evidence_refs: evidence_refs(right),
    })
}

fn change_for_removed(domain: &str, selector: &str, left: &Value) -> Result<DiffChange> {
    Ok(DiffChange {
        category: "removed".into(),
        domain: domain.into(),
        severity: severity_for_domain(domain).into(),
        selector: selector.into(),
        message: format!("{selector} was removed"),
        compatibility_hint: compatibility_hint(domain),
        evidence_refs: evidence_refs(left),
    })
}

fn severity_for_domain(domain: &str) -> &'static str {
    match domain {
        "metadata" => "low",
        "field_metadata" | "browser_templates" => "low",
        _ => "medium",
    }
}

fn compatibility_hint(domain: &str) -> String {
    match domain {
        "notetypes" => "compare the stock notetype shape and fields".into(),
        "templates" => "compare the stock template render formats".into(),
        "fields" => "compare the stock field definitions".into(),
        "media" => "compare the media layout and payload metadata".into(),
        "metadata" => "compare aggregate counts and package metadata".into(),
        "references" => "compare note, card, and media-reference selectors".into(),
        "field_metadata" => "compare field labels and role hints".into(),
        "browser_templates" => "compare browser-specific template appearance".into(),
        "template_target_decks" => {
            "review template deck routing and resolved deck identities".into()
        }
        _ => "compare the selected observation domain".into(),
    }
}

fn evidence_refs(value: &Value) -> Vec<String> {
    value
        .get("evidence_refs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn merge_evidence_refs(left: &Value, right: &Value) -> Vec<String> {
    let mut refs = BTreeSet::new();
    refs.extend(evidence_refs(left));
    refs.extend(evidence_refs(right));
    refs.into_iter().collect()
}

#[cfg(test)]
mod ownership_tests {
    use super::*;

    #[test]
    fn comparison_indexes_borrow_observations_and_keep_last_duplicate_selector() {
        let values = vec![
            serde_json::json!({"selector":"row", "content":"old"}),
            serde_json::json!({"no_selector":"ignored"}),
            serde_json::json!({"selector":"row", "content":"current"}),
        ];
        let entries = domain_entries(&values);
        assert_eq!(entries.len(), 1);
        let indexed: &Value = entries["row"];
        assert!(
            std::ptr::eq(indexed, &values[2]),
            "comparison must not copy the observation tree"
        );
    }

    #[test]
    fn borrowed_payload_preserves_recursive_domain_filters_and_numeric_encoding() {
        let value = serde_json::json!({
            "selector":"row", "evidence_refs":["removed"], "object_id":"media only",
            "nested":[{"binding_id":"media only", "object_ref":"media only", "selector":"removed", "evidence_refs":[], "data":[1,1.0,-0.0,0.0,"尾"]}],
        });
        for domain in ["media", "metadata", "fields"] {
            let expected =
                crate::writer_core::to_canonical_json(&strip_non_semantic_fields(domain, &value))
                    .unwrap();
            assert_eq!(entry_payload(domain, &value).unwrap(), expected);
        }
        // JSON numbers with different encodings remain distinct, even when
        // IEEE equality would consider them equal.
        assert_ne!(
            entry_payload("fields", &serde_json::json!(-0.0)).unwrap(),
            entry_payload("fields", &serde_json::json!(0.0)).unwrap()
        );
    }
}
