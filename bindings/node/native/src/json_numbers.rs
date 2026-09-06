use serde_json::Value;

/// Encode unsafe integers before JSON reaches JavaScript's number parser.
pub fn safe_json_numbers(value: &mut Value) {
    match value {
        Value::Number(number)
            if number.as_u64().is_some_and(|n| n > 9_007_199_254_740_991)
                || number.as_i64().is_some_and(|n| n < -9_007_199_254_740_991) =>
        {
            *value = Value::String(number.to_string());
        }
        Value::Array(values) => values.iter_mut().for_each(safe_json_numbers),
        Value::Object(values) => values.values_mut().for_each(safe_json_numbers),
        _ => {}
    }
}
