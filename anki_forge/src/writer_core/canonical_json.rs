use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

pub fn to_canonical_json(value: &impl Serialize) -> Result<String> {
    let value = serde_json::to_value(value)?;
    let normalized = normalize(value);
    Ok(serde_json::to_string(&normalized)?)
}

fn normalize(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries = map.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, normalize(value)))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.into_iter().map(normalize).collect()),
        other => other,
    }
}
