use std::collections::BTreeMap;
#[cfg(feature = "internal-tools")]
use std::collections::BTreeSet;

#[cfg(feature = "internal-tools")]
use anyhow::Result;
use serde_json::Value;

#[cfg(feature = "internal-tools")]
use crate::writer_core::model::{DiffChange, DiffReport, InspectReport};

#[cfg(feature = "internal-tools")]
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

        let mut left_entries = domain_entries(left_values).into_iter();
        let mut right_entries = domain_entries(right_values).into_iter();
        let mut left_next = left_entries.next();
        let mut right_next = right_entries.next();
        // Merge the sorted indexes directly, retaining last-duplicate-wins
        // semantics without another selector set or repeated tree lookups.
        loop {
            let (selector, left_entry, right_entry) = match (left_next, right_next) {
                (Some((selector, value)), right)
                    if right.is_none_or(|(other, _)| selector < other) =>
                {
                    left_next = left_entries.next();
                    (selector, Some(value), None)
                }
                (left, Some((selector, value)))
                    if left.is_none_or(|(other, _)| selector < other) =>
                {
                    right_next = right_entries.next();
                    (selector, None, Some(value))
                }
                (Some((selector, left)), Some((_, right))) => {
                    left_next = left_entries.next();
                    right_next = right_entries.next();
                    (selector, Some(left), Some(right))
                }
                _ => break,
            };
            if should_skip_selector(domain, selector, left, right) {
                continue;
            }
            match (left_entry, right_entry) {
                (Some(left_entry), Some(right_entry)) => {
                    if !entry_payloads_equal(domain, left_entry, right_entry) {
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

#[cfg(feature = "internal-tools")]
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

#[cfg(feature = "internal-tools")]
fn has_unavailable(left: &InspectReport, right: &InspectReport) -> bool {
    left.observation_status == "unavailable" || right.observation_status == "unavailable"
}

#[cfg(feature = "internal-tools")]
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

fn entry_payloads_equal(domain: &str, left: &Value, right: &Value) -> bool {
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
    filtered_values_equal(left, right, excluded)
}

fn filtered_values_equal(left: &Value, right: &Value, excluded: &[&str]) -> bool {
    match (left, right) {
        (Value::Object(left), Value::Object(right)) => {
            let semantic_entries = |key: &&String| !excluded.contains(&key.as_str());
            left.keys().filter(semantic_entries).count()
                == right.keys().filter(semantic_entries).count()
                && left.iter().all(|(key, value)| {
                    excluded.contains(&key.as_str())
                        || right
                            .get(key)
                            .is_some_and(|other| filtered_values_equal(value, other, excluded))
                })
        }
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| filtered_values_equal(left, right, excluded))
        }
        // Preserve canonical JSON's numeric representation, including 1 vs
        // 1.0 and signed zero. Value/Number equality alone loses signed zero.
        (Value::Number(left), Value::Number(right)) => left.to_string() == right.to_string(),
        _ => left == right,
    }
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

#[cfg(feature = "internal-tools")]
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

#[cfg(feature = "internal-tools")]
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

#[cfg(feature = "internal-tools")]
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

#[cfg(feature = "internal-tools")]
fn severity_for_domain(domain: &str) -> &'static str {
    match domain {
        "metadata" => "low",
        "field_metadata" | "browser_templates" => "low",
        _ => "medium",
    }
}

#[cfg(feature = "internal-tools")]
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

#[cfg(feature = "internal-tools")]
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

#[cfg(feature = "internal-tools")]
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
    fn structural_comparison_matches_canonical_payloads() {
        let mut values: Vec<Value> = serde_json::from_str(
            r#"[null,false,true,0,1,-1,1.0,-0.0,0.0,18446744073709551615,
                -9223372036854775808,1e-200,1e200,"","尾 \\ \"\n",[],[1,2],[2,1],{},
                {"selector":"ignored","evidence_refs":["ignored"]},
                {"object_id":"media only","binding_id":1,"object_ref":null},
                {"a":null},{"a":1,"b":2},{"b":2,"a":1}]"#,
        )
        .unwrap();
        for value in values.clone() {
            values.push(serde_json::json!({
                "selector":"row", "evidence_refs":["removed"],
                "nested":[{"binding_id":"media only", "object_ref":"media only", "data":value}],
            }));
        }
        for domain in ["media", "metadata", "fields"] {
            let canonical: Vec<_> = values
                .iter()
                .map(|value| {
                    crate::writer_core::to_canonical_json(&strip_non_semantic_fields(domain, value))
                        .unwrap()
                })
                .collect();
            for (i, left) in values.iter().enumerate() {
                for (j, right) in values.iter().enumerate() {
                    assert_eq!(
                        entry_payloads_equal(domain, left, right),
                        canonical[i] == canonical[j],
                        "{domain}: {left} vs {right}"
                    );
                }
            }
        }
    }
}
