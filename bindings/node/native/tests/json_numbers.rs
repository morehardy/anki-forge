#[path = "../src/json_numbers.rs"]
mod json_numbers;

#[test]
fn json_boundary_preserves_signed_and_unsigned_integer_limits() {
    let mut value = serde_json::json!({
        "ids": [9_007_199_254_740_991_u64, 9_007_199_254_740_992_u64, u64::MAX],
        "revision": {"minimum": i64::MIN, "maximum": i64::MAX},
        "negative_safe": -9_007_199_254_740_991_i64,
        "negative_unsafe": -9_007_199_254_740_992_i64,
        "count": 0, "ratio": 1.25, "optional": null, "name": "9007199254740993"
    });
    json_numbers::safe_json_numbers(&mut value);
    assert_eq!(
        value,
        serde_json::json!({
            "ids": [9_007_199_254_740_991_u64, "9007199254740992", "18446744073709551615"],
            "revision": {"minimum": "-9223372036854775808", "maximum": "9223372036854775807"},
            "negative_safe": -9_007_199_254_740_991_i64,
            "negative_unsafe": "-9007199254740992",
            "count": 0, "ratio": 1.25, "optional": null, "name": "9007199254740993"
        })
    );
}
