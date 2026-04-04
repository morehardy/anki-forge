use serde::Serialize;
use serde_json::{Map, Value};

pub fn to_canonical_json<T: Serialize>(value: &T) -> anyhow::Result<String> {
    let mut value = serde_json::to_value(value)?;
    sort_value(&mut value);
    Ok(serde_json::to_string(&value)?)
}

fn sort_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut sorted = Map::new();

            for key in keys {
                if let Some(mut child) = map.remove(&key) {
                    sort_value(&mut child);
                    sorted.insert(key, child);
                }
            }

            *map = sorted;
        }
        Value::Array(items) => {
            for item in items {
                sort_value(item);
            }
        }
        _ => {}
    }
}
