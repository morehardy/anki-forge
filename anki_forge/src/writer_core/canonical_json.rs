use anyhow::Result;
use serde::Serialize;

pub fn to_canonical_json(value: &impl Serialize) -> Result<String> {
    let mut value = serde_json::to_value(value)?;
    value.sort_all_objects();
    Ok(serde_json::to_string(&value)?)
}
