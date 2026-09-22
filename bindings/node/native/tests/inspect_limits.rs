#[path = "../src/options.rs"]
#[allow(dead_code)]
mod options;

#[test]
fn inspection_budget_protocol_preserves_u64_without_floating_point() {
    for (json, expected) in [
        ("0", 0),
        ("9007199254740991", 9_007_199_254_740_991),
        ("\"9007199254740993\"", 9_007_199_254_740_993),
        ("\"18446744073709551615\"", u64::MAX),
    ] {
        let input: options::InspectInput =
            serde_json::from_str(&format!("{{\"maxEntries\":{json}}}")).unwrap();
        assert_eq!(input.limits().max_entries, expected);
    }
    for invalid in [
        "9007199254740992",
        "-1",
        "1.5",
        "\"-1\"",
        "\"+1\"",
        "\"1e5\"",
        "\"18446744073709551616\"",
        "\"\"",
    ] {
        assert!(serde_json::from_str::<options::InspectInput>(&format!(
            "{{\"maxEntries\":{invalid}}}"
        ))
        .is_err());
    }
}

#[test]
fn default_budget_values_are_safe_javascript_numbers() {
    for value in options::default_limits().as_object().unwrap().values() {
        assert!(value.as_u64().unwrap() <= 9_007_199_254_740_991);
    }
}
