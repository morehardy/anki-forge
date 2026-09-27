use ankiforge::{Note, Project};
use contract_tools::{
    contract_manifest_path,
    manifest::{load_manifest, resolve_asset_path},
    schema::load_schema,
};
use serde_json::json;

#[test]
fn project_tag_schema_matches_runtime_validation() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let validator =
        load_schema(resolve_asset_path(&manifest, "project_input_schema").unwrap()).unwrap();
    let check = |tags: Vec<String>, accepted: bool| {
        let value = json!({
            "format_version": "ankiforge-project-v1",
            "namespace": "tag-parity",
            "notes": [{
                "key": "one",
                "content": {
                    "kind": "basic",
                    "front": {"kind": "text", "value": "Front"},
                    "back": {"kind": "text", "value": "Back"}
                },
                "tags": tags
            }]
        });
        let runtime = Project::new("tag-parity")
            .unwrap()
            .add("one", Note::basic("Front", "Back").tags(tags.clone()));
        assert_eq!(runtime.is_ok(), accepted, "runtime tags: {tags:?}");
        if let Err(error) = runtime {
            assert_eq!(error.code(), "NOTE.TAG_INVALID");
        }
        assert_eq!(
            validator.is_valid(&value),
            accepted,
            "schema tags: {tags:?}"
        );
    };

    for tags in [
        vec![],
        vec!["topic::rust", "标签", "café", "日本語"],
        vec!["tag", "TAG", "é", "e\u{301}"],
        vec!["AFID::user", "afid:", "x_afid::user"],
        // Format characters are not Rust whitespace/control characters. In
        // particular, ECMAScript \s would incorrectly reject the BOM here.
        vec!["\u{180e}", "\u{200b}", "\u{2060}", "\u{feff}"],
    ] {
        check(tags.into_iter().map(str::to_owned).collect(), true);
    }
    for tags in [
        vec![""],
        vec!["afid::"],
        vec!["afid::user"],
        vec!["tag", "tag"],
    ] {
        check(tags.into_iter().map(str::to_owned).collect(), false);
    }

    // Discover the forbidden set from Rust itself, exercising every member
    // and its adjacent accepted scalar values rather than sampling ASCII.
    for character in (0..=0x10ffff).filter_map(char::from_u32) {
        if !(character.is_whitespace() || character.is_control()) {
            continue;
        }
        for tag in [
            character.to_string(),
            format!("{character}tag"),
            format!("tag{character}"),
            format!("left{character}right"),
        ] {
            check(vec![tag], false);
        }
        for adjacent in [
            u32::from(character).checked_sub(1),
            Some(u32::from(character) + 1),
        ]
        .into_iter()
        .flatten()
        .filter_map(char::from_u32)
        {
            if !adjacent.is_whitespace() && !adjacent.is_control() {
                check(vec![adjacent.to_string()], true);
            }
        }
    }
}
