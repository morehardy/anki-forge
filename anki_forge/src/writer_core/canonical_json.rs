use anyhow::Result;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::Serialize;
use serde_json::Value;

pub fn to_canonical_json(value: &impl Serialize) -> Result<String> {
    let mut value = serde_json::to_value(value)?;
    value.sort_all_objects();
    Ok(serde_json::to_string(&value)?)
}

/// Borrow a JSON value while recursively excluding metadata keys. Object keys
/// are sorted explicitly, including when a consumer enables `preserve_order`.
pub(crate) struct FilteredValue<'a> {
    pub value: &'a Value,
    pub excluded: &'static [&'static str],
}

impl Serialize for FilteredValue<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.value {
            Value::Object(map) => {
                let mut entries = map
                    .iter()
                    .filter(|(key, _)| !self.excluded.contains(&key.as_str()))
                    .collect::<Vec<_>>();
                entries.sort_unstable_by(|left, right| left.0.cmp(right.0));
                let mut output = serializer.serialize_map(Some(entries.len()))?;
                for (key, value) in entries {
                    output.serialize_entry(
                        key,
                        &Self {
                            value,
                            excluded: self.excluded,
                        },
                    )?;
                }
                output.end()
            }
            Value::Array(items) => {
                let mut output = serializer.serialize_seq(Some(items.len()))?;
                for value in items {
                    output.serialize_element(&Self {
                        value,
                        excluded: self.excluded,
                    })?;
                }
                output.end()
            }
            value => value.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_filter_preserves_canonical_values_and_sorts_nested_keys() {
        let value: Value = serde_json::from_str(r#"{"z":{"z":"尾","evidence_refs":[],"a":2},"numbers":[1,1.0,-0.0,0.0,18446744073709551615],"a":[{"selector":"kept","evidence_refs":["removed"],"a":"evidence_refs"}]}"#).unwrap();
        let output = serde_json::to_string(&FilteredValue {
            value: &value,
            excluded: &["evidence_refs"],
        })
        .unwrap();
        assert_eq!(
            output,
            r#"{"a":[{"a":"evidence_refs","selector":"kept"}],"numbers":[1,1.0,-0.0,0.0,18446744073709551615],"z":{"a":2,"z":"尾"}}"#
        );
        assert!(value["z"].get("evidence_refs").is_some());
    }
}
