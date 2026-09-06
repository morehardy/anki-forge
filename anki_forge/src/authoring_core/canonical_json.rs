use serde::Serialize;

pub fn to_canonical_json<T: Serialize>(value: &T) -> anyhow::Result<String> {
    let mut value = serde_json::to_value(value)?;
    // This is a no-op for the default sorted maps and also handles consumers
    // that enable serde_json's preserve_order feature through unification.
    value.sort_all_objects();
    Ok(serde_json::to_string(&value)?)
}
