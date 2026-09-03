#![cfg(feature = "internal-tools")]

use anki_forge::update_safety::merge_safety::compare_notetype_merge_safety;
use anki_forge::update_safety::model::{
    FieldMergeEntry, IdentityIndex, NotetypeIdentityEntry, TemplateMergeEntry,
};

#[test]
fn field_config_id_drift_is_error() {
    let current = index_with_notetype(
        field("front", "Front", 0, 111),
        template("card", "Card", 0, 222),
    );
    let baseline = index_with_notetype(
        field("front", "Front", 0, 999),
        template("card", "Card", 0, 222),
    );

    let diagnostics = compare_notetype_merge_safety(&current, &baseline);

    assert!(diagnostics
        .iter()
        .any(|d| d.code.as_str() == "UPDATE.FIELD_MERGE_ID_CHANGED"));
}

#[test]
fn notetype_field_and_template_renames_are_warnings_when_ids_stay_stable() {
    let current = index_with_named_notetype(
        "Renamed",
        field("front", "Prompt", 0, 111),
        template("card", "Prompt Card", 0, 222),
    );
    let baseline = index_with_named_notetype(
        "Original",
        field("front", "Front", 0, 111),
        template("card", "Card", 0, 222),
    );

    let codes: Vec<_> = compare_notetype_merge_safety(&current, &baseline)
        .into_iter()
        .map(|d| d.code.as_str().to_string())
        .collect();

    assert!(codes.contains(&"UPDATE.NOTETYPE_RENAMED".into()));
    assert!(codes.contains(&"UPDATE.FIELD_RENAMED".into()));
    assert!(codes.contains(&"UPDATE.TEMPLATE_RENAMED".into()));
    assert!(!codes.contains(&"UPDATE.FIELD_MERGE_ID_CHANGED".into()));
}

#[test]
fn template_ord_changed_source_names_the_template_selector() {
    let current = index_with_notetype(
        field("front", "Front", 0, 111),
        template("card", "Card", 1, 222),
    );
    let baseline = index_with_notetype(
        field("front", "Front", 0, 111),
        template("card", "Card", 0, 222),
    );

    let diagnostics = compare_notetype_merge_safety(&current, &baseline);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "UPDATE.TEMPLATE_ORD_CHANGED")
        .expect("template ord diagnostic");

    assert_eq!(
        diagnostic.source.as_ref().map(|source| source.as_str()),
        Some("notetype[id='basic']::template[Card]")
    );
}

#[test]
fn notetype_and_template_set_changes_include_change_kind() {
    let current = index_with_named_notetype(
        "Basic",
        field("front", "Front", 0, 111),
        template("new-card", "New Card", 0, 333),
    );
    let baseline = index_with_named_notetype(
        "Basic",
        field("front", "Front", 0, 111),
        template("old-card", "Old Card", 0, 222),
    );
    let mut added_removed_current = current.clone();
    added_removed_current.notetypes[0].note_type_id = "basic-new".into();

    let mut diagnostics = compare_notetype_merge_safety(&current, &baseline);
    diagnostics.extend(compare_notetype_merge_safety(
        &added_removed_current,
        &baseline,
    ));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code.as_str() == "UPDATE.NOTETYPE_SET_CHANGED"
            && diagnostic.message.contains("change_kind=added")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code.as_str() == "UPDATE.NOTETYPE_SET_CHANGED"
            && diagnostic.message.contains("change_kind=removed")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code.as_str() == "UPDATE.TEMPLATE_SET_CHANGED"
            && diagnostic.message.contains("change_kind=added")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code.as_str() == "UPDATE.TEMPLATE_SET_CHANGED"
            && diagnostic.message.contains("change_kind=removed")
    }));
}

fn index_with_notetype(field: FieldMergeEntry, template: TemplateMergeEntry) -> IdentityIndex {
    index_with_named_notetype("Basic", field, template)
}

fn index_with_named_notetype(
    name: &str,
    field: FieldMergeEntry,
    template: TemplateMergeEntry,
) -> IdentityIndex {
    let mut index = IdentityIndex::empty_lockfile("project-a", "writer-policy.default@1.0.0");
    index.notetypes.push(NotetypeIdentityEntry {
        note_type_id: "basic".into(),
        anki_model_id: Some(1),
        name: name.into(),
        fields: vec![field],
        templates: vec![template],
    });
    index
}

fn field(key: &str, name: &str, ord: u32, config_id: i64) -> FieldMergeEntry {
    FieldMergeEntry {
        field_key: key.into(),
        field_name: name.into(),
        ord,
        config_id,
        tag: ord as i32,
    }
}

fn template(key: &str, name: &str, ord: u32, config_id: i64) -> TemplateMergeEntry {
    TemplateMergeEntry {
        template_key: key.into(),
        template_name: name.into(),
        ord,
        config_id,
    }
}
