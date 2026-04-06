use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use serde_json::Value;

use crate::model::{DiffChange, DiffReport, InspectReport};
use crate::to_canonical_json;

const DOMAINS: [&str; 6] = [
    "notetypes",
    "templates",
    "fields",
    "media",
    "metadata",
    "references",
];

pub fn diff_reports(left: &InspectReport, right: &InspectReport) -> Result<DiffReport> {
    let mut uncompared_domains = BTreeSet::new();
    let mut comparison_limitations = BTreeSet::new();
    let mut changes = vec![];

    let mut comparison_status = compare_status(left, right);

    for domain in DOMAINS {
        if left.missing_domains.iter().any(|missing| missing == domain) {
            uncompared_domains.insert(domain.to_string());
            comparison_limitations.insert(format!("left report missing {domain} domain"));
            continue;
        }
        if right
            .missing_domains
            .iter()
            .any(|missing| missing == domain)
        {
            uncompared_domains.insert(domain.to_string());
            comparison_limitations.insert(format!("right report missing {domain} domain"));
            continue;
        }

        let left_entries = domain_entries(left, domain);
        let right_entries = domain_entries(right, domain);
        let mut selectors = BTreeSet::new();
        selectors.extend(left_entries.keys().cloned());
        selectors.extend(right_entries.keys().cloned());

        for selector in selectors {
            match (left_entries.get(&selector), right_entries.get(&selector)) {
                (Some(left_entry), Some(right_entry)) => {
                    if entry_payload(left_entry)? != entry_payload(right_entry)? {
                        changes.push(change_for_modified(
                            domain,
                            &selector,
                            left_entry,
                            right_entry,
                        )?);
                    }
                }
                (Some(left_entry), None) => {
                    changes.push(change_for_removed(domain, &selector, left_entry)?);
                }
                (None, Some(right_entry)) => {
                    changes.push(change_for_added(domain, &selector, right_entry)?);
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

    let summary = if changes.is_empty() {
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

fn domain_entries(report: &InspectReport, domain: &str) -> BTreeMap<String, Value> {
    let values = match domain {
        "notetypes" => &report.observations.notetypes,
        "templates" => &report.observations.templates,
        "fields" => &report.observations.fields,
        "media" => &report.observations.media,
        "metadata" => &report.observations.metadata,
        "references" => &report.observations.references,
        _ => return BTreeMap::new(),
    };

    let mut entries = BTreeMap::new();
    for value in values {
        let Some(selector) = value.get("selector").and_then(Value::as_str) else {
            continue;
        };
        entries.insert(selector.to_string(), value.clone());
    }
    entries
}

fn entry_payload(value: &Value) -> Result<String> {
    let payload = strip_non_semantic_fields(value);
    to_canonical_json(&payload)
}

fn strip_non_semantic_fields(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut map = map.clone();
            map.remove("selector");
            map.remove("evidence_refs");
            Value::Object(
                map.into_iter()
                    .map(|(key, value)| (key, strip_non_semantic_fields(&value)))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.iter().map(strip_non_semantic_fields).collect()),
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
