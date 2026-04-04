use authoring_core::selector::{
    parse_selector, resolve_selector, SelectorError, SelectorResolveError, SelectorTarget,
};

#[test]
fn rejects_array_index_segments() {
    let err = parse_selector("note[id='n1']/fields[12]").expect_err("selector should be rejected");
    assert_eq!(err, SelectorError::ArrayIndexNotAllowed);
}

#[test]
fn accepts_kind_with_predicates() {
    let selector = parse_selector("note[id='n1']").expect("selector should parse");

    assert_eq!(selector.kind, "note");
    assert_eq!(selector.predicates, vec![("id".into(), "n1".into())]);
}

#[test]
fn accepts_slash_inside_quoted_value() {
    let selector = parse_selector("note[id='a/b']").expect("selector should parse");
    assert_eq!(selector.predicates, vec![("id".into(), "a/b".into())]);
}

#[test]
fn quoted_bracket_digits_are_not_treated_as_array_index() {
    let selector = parse_selector("note[id='x[12]y']").expect("selector should parse");
    assert_eq!(selector.predicates, vec![("id".into(), "x[12]y".into())]);
}

#[test]
fn kind_only_selector_is_rejected() {
    let err = parse_selector("note").expect_err("kind-only selector should be invalid");
    assert_eq!(err, SelectorError::InvalidPredicate);
}

#[test]
fn chained_predicate_blocks_are_rejected() {
    let err =
        parse_selector("note[id='n1'][deck='d1']").expect_err("selector should be invalid");
    assert_eq!(err, SelectorError::InvalidPredicate);
}

#[test]
fn resolver_returns_unmatched_for_zero_matches() {
    let selector = parse_selector("note[id='n1']").expect("selector should parse");
    let targets = vec![SelectorTarget::new("note", [("id", "n2")])];

    let err = resolve_selector(&selector, &targets).expect_err("selector should not match");
    assert_eq!(err, SelectorResolveError::Unmatched);
}

#[test]
fn resolver_returns_ambiguous_for_multiple_matches() {
    let selector = parse_selector("note[id='n1']").expect("selector should parse");
    let targets = vec![
        SelectorTarget::new("note", [("id", "n1")]),
        SelectorTarget::new("note", [("id", "n1")]),
    ];

    let err = resolve_selector(&selector, &targets).expect_err("selector should be ambiguous");
    assert_eq!(err, SelectorResolveError::Ambiguous);
}
