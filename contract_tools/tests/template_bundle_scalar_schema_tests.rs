use std::{fs, path::Path};

use ankiforge::{tools::load_project, Field, Media, NoteType, Template};
use contract_tools::{
    contract_manifest_path,
    manifest::{load_manifest, resolve_asset_path},
    schema::load_schema,
};
use serde_json::{json, Value};

fn validator() -> jsonschema::JSONSchema {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    load_schema(resolve_asset_path(&manifest, "template_bundle_schema").unwrap()).unwrap()
}

fn manifest() -> Value {
    json!({"format_version": "template-bundle-v2", "note_type": {
        "key": "custom", "fields": [{"key": "front"}],
        "templates": [{"key": "card", "front_file": "front.html", "back_file": "back.html"}]
    }})
}

fn prepare(root: &Path) {
    for path in ["front.html", "back.html", "asset.txt"] {
        fs::write(root.join(path), "content").unwrap();
    }
}

fn check(validator: &jsonschema::JSONSchema, root: &Path, value: &Value, accepted: bool) {
    fs::write(
        root.join("anki-template.yaml"),
        serde_yaml::to_string(value).unwrap(),
    )
    .unwrap();
    let loaded = NoteType::from_bundle(root);
    assert_eq!(
        loaded.is_ok(),
        accepted,
        "bundle loader: {loaded:?}; {value}"
    );
    let project_path = root.join("project.json");
    fs::write(
        &project_path,
        serde_json::to_vec(&json!({
            "format_version": "ankiforge-project-v1", "namespace": "bundle-parity",
            "models": [{"kind": "bundle", "path": "."}], "notes": []
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        load_project(project_path, None).is_ok(),
        accepted,
        "project loader: {value}"
    );
    assert_eq!(validator.is_valid(value), accepted, "schema: {value}");
}

#[test]
fn bundle_paths_match_portable_resolver_without_rejecting_legal_file_names() {
    let validator = validator();
    for (path, accepted) in [
        ("front.html", true),
        ("./front.html", true),
        ("nested//file.html", true),
        ("nested/./file.html", true),
        ("中文.html", true),
        (" padded ", true),
        ("new\nline", true),
        ("\u{feff}", true),
        ("..name", true),
        ("", false),
        ("/absolute", false),
        ("../escape", false),
        ("nested/../front.html", false),
        ("nested/..", false),
        ("front:part.html", false),
        ("a\\b", false),
        ("C:front.html", false),
        ("null\0byte", false),
    ] {
        if accepted && !cfg!(unix) && (path.contains('\n') || path.ends_with(' ')) {
            continue;
        }
        for pointer in [
            "/css_file",
            "/note_type/templates/0/front_file",
            "/note_type/templates/0/back_file",
            "/note_type/templates/0/browser_front_file",
            "/note_type/templates/0/browser_back_file",
            "/assets/0/path",
        ] {
            let root = tempfile::tempdir().unwrap();
            prepare(root.path());
            if accepted {
                let destination = root.path().join(path);
                fs::create_dir_all(destination.parent().unwrap()).unwrap();
                fs::write(destination, "content").unwrap();
            }
            let mut value = manifest();
            value["css_file"] = Value::Null;
            value["note_type"]["templates"][0]["browser_front_file"] = Value::Null;
            value["note_type"]["templates"][0]["browser_back_file"] = Value::Null;
            value["assets"] = json!([{"path": "asset.txt", "export_as": "asset.txt"}]);
            *value.pointer_mut(pointer).unwrap() = json!(path);
            check(&validator, root.path(), &value, accepted);
        }
    }
}

#[test]
fn bundle_export_filenames_match_media_api() {
    let validator = validator();
    let mut names = vec![
        "image.png".to_owned(),
        "中文.png".to_owned(),
        "\u{feff}".to_owned(),
        "two words".to_owned(),
        "CONextra".to_owned(),
        "x.COM1".to_owned(),
        "a".repeat(255),
        "a".repeat(256),
        "é".repeat(127),
    ];
    names.extend(
        [
            "", ".", "..", " file", "file ", "file.", "a/b", "a\\b", "a<b", "a>b", "a:b", "a\"b",
            "a|b", "a?b", "a*b", "a[b", "a]b", "a\0b", "a\u{9f}b", "CON", "con.txt", "PRN", "AUX",
            "nul", "CLOCK$", "COM1", "com9.txt", "COM¹", "COM²", "COM³", "LPT1", "lpt9.txt",
            "LPT¹", "LPT²", "LPT³",
        ]
        .map(str::to_owned),
    );
    for name in names {
        let root = tempfile::tempdir().unwrap();
        prepare(root.path());
        let accepted = Media::file(root.path().join("asset.txt"))
            .unwrap()
            .with_export_name(&name)
            .is_ok();
        let mut value = manifest();
        value["assets"] = json!([{"path": "asset.txt", "export_as": name}]);
        check(&validator, root.path(), &value, accepted);
    }
}

#[test]
fn bundle_decks_match_authored_deck_rules() {
    let validator = validator();
    let mut decks = [
        "Default",
        "Parent::Child",
        "A:B::C:D",
        "Two Words",
        "中文::Café",
        "\u{feff}",
        "",
        ":",
        "Parent:::Child",
        "Parent::::Child",
        "::Child",
        "Parent::",
        " Parent",
        "Parent ",
        "Parent:: Child",
        "Parent ::Child",
    ]
    .map(str::to_owned)
    .to_vec();
    for c in (0..=0x10ffff).filter_map(char::from_u32) {
        if c.is_whitespace() || c.is_control() {
            decks.extend([
                format!("Parent::{c}Child"),
                format!("Parent{c}::Child"),
                format!("Par{c}ent"),
            ]);
        }
    }
    for deck in decks {
        let accepted = NoteType::builder("custom")
            .field(Field::new("front"))
            .template(Template::new("card").front("content").target_deck(&deck))
            .build()
            .is_ok();
        let root = tempfile::tempdir().unwrap();
        prepare(root.path());
        let mut value = manifest();
        value["note_type"]["templates"][0]["target_deck"] = json!(deck);
        check(&validator, root.path(), &value, accepted);
    }
}

#[test]
fn bundle_names_and_keys_match_builder_unicode_validation() {
    let validator = validator();
    for text in [
        "",
        " ",
        "\u{85}",
        "\u{feff}",
        "\u{200b}",
        "中文",
        " padded ",
        "two words",
        "bad\0name",
        "bad\u{9f}name",
        "field:key",
        "FrontSide",
        "a{b",
    ] {
        for location in [
            "model_key",
            "model_name",
            "template_key",
            "template_name",
            "field_key",
            "field_name",
        ] {
            let root = tempfile::tempdir().unwrap();
            prepare(root.path());
            let mut value = manifest();
            let mut builder = NoteType::builder(if location == "model_key" {
                text
            } else {
                "custom"
            });
            let mut field = Field::new(if location == "field_key" {
                text
            } else {
                "front"
            });
            let mut template = Template::new(if location == "template_key" {
                text
            } else {
                "card"
            })
            .front("content");
            let pointer = match location {
                "model_key" => "/note_type/key",
                "model_name" => {
                    builder = builder.name(text);
                    value["note_type"]["name"] = Value::Null;
                    "/note_type/name"
                }
                "template_key" => "/note_type/templates/0/key",
                "template_name" => {
                    template = template.name(text);
                    value["note_type"]["templates"][0]["name"] = Value::Null;
                    "/note_type/templates/0/name"
                }
                "field_key" => "/note_type/fields/0/key",
                _ => {
                    field = field.name(text);
                    value["note_type"]["fields"][0]["name"] = Value::Null;
                    "/note_type/fields/0/name"
                }
            };
            *value.pointer_mut(pointer).unwrap() = json!(text);
            let accepted = builder.field(field).template(template).build().is_ok();
            check(&validator, root.path(), &value, accepted);
        }
    }
}

#[test]
fn bundle_optional_nulls_and_required_types_match_loader() {
    let validator = validator();
    let root = tempfile::tempdir().unwrap();
    prepare(root.path());
    let mut value = manifest();
    value["css_file"] = Value::Null;
    value["note_type"]["name"] = Value::Null;
    value["note_type"]["cloze_field"] = Value::Null;
    value["note_type"]["fields"][0]["name"] = Value::Null;
    for property in [
        "name",
        "browser_front_file",
        "browser_back_file",
        "target_deck",
        "generation_rule",
    ] {
        value["note_type"]["templates"][0][property] = Value::Null;
    }
    check(&validator, root.path(), &value, true);
    for pointer in [
        "/format_version",
        "/note_type/key",
        "/note_type/fields",
        "/note_type/templates",
        "/note_type/fields/0/key",
        "/note_type/templates/0/key",
        "/note_type/templates/0/front_file",
        "/note_type/templates/0/back_file",
    ] {
        let mut invalid = value.clone();
        *invalid.pointer_mut(pointer).unwrap() = json!({});
        check(&validator, root.path(), &invalid, false);
    }
    for property in ["sort", "required"] {
        for invalid_scalar in [Value::Null, json!(0), json!("true")] {
            let mut invalid = value.clone();
            invalid["note_type"]["fields"][0][property] = invalid_scalar;
            check(&validator, root.path(), &invalid, false);
        }
        let mut valid = value.clone();
        valid["note_type"]["fields"][0][property] = json!(true);
        check(&validator, root.path(), &valid, true);
    }
    for assets in [
        Value::Null,
        json!([{"path": {}, "export_as": "file"}]),
        json!([{"path": "asset.txt", "export_as": {}}]),
    ] {
        let mut invalid = value.clone();
        invalid["assets"] = assets;
        check(&validator, root.path(), &invalid, false);
    }
}

#[test]
fn bundle_cardinality_and_generation_rules_match_loader() {
    let validator = validator();
    let root = tempfile::tempdir().unwrap();
    prepare(root.path());
    for property in ["fields", "templates"] {
        let mut value = manifest();
        value["note_type"][property] = json!([]);
        check(&validator, root.path(), &value, false);
    }
    for (rule, accepted) in [
        (json!({"kind": "anki_default"}), true),
        (json!({"kind": "all", "fields": ["front"]}), true),
        (json!({"kind": "any", "fields": ["front"]}), true),
        (json!({"kind": "all", "fields": []}), false),
        (json!({"kind": "all", "fields": ["front", "front"]}), false),
        (json!({"kind": "all", "fields": null}), false),
        (json!({"kind": "all", "fields": [123]}), false),
        (json!({"kind": "all", "fields": [true]}), false),
        (json!({"kind": "all", "fields": [null]}), false),
        (json!({"kind": "all"}), false),
    ] {
        let mut value = manifest();
        value["note_type"]["templates"][0]["generation_rule"] = rule;
        check(&validator, root.path(), &value, accepted);
    }
    let mut normal = manifest();
    let mut second = normal["note_type"]["templates"][0].clone();
    second["key"] = json!("second");
    normal["note_type"]["templates"]
        .as_array_mut()
        .unwrap()
        .push(second);
    check(&validator, root.path(), &normal, true);
    fs::write(root.path().join("front.html"), "{{cloze:front}}").unwrap();
    for (rule, accepted) in [
        (Value::Null, true),
        (json!({"kind": "anki_default"}), true),
        (json!({"kind": "all", "fields": ["front"]}), false),
    ] {
        let mut value = manifest();
        value["note_type"]["cloze_field"] = json!("front");
        value["note_type"]["templates"][0]["generation_rule"] = rule;
        check(&validator, root.path(), &value, accepted);
    }
    let mut value = manifest();
    value["note_type"]["cloze_field"] = json!("front");
    let mut second = value["note_type"]["templates"][0].clone();
    second["key"] = json!("second");
    value["note_type"]["templates"]
        .as_array_mut()
        .unwrap()
        .push(second);
    check(&validator, root.path(), &value, false);
}

#[test]
fn bundle_yaml_scalar_types_are_not_coerced_to_strings() {
    let validator = validator();
    let root = tempfile::tempdir().unwrap();
    prepare(root.path());
    let mut base = manifest();
    base["assets"] = json!([{"path": "asset.txt", "export_as": "asset.txt"}]);
    for pointer in [
        "/format_version",
        "/note_type/key",
        "/note_type/fields/0/key",
        "/note_type/templates/0/key",
        "/note_type/templates/0/front_file",
        "/note_type/templates/0/back_file",
        "/assets/0/path",
        "/assets/0/export_as",
    ] {
        for scalar in [Value::Null, json!(123), json!(true)] {
            let mut value = base.clone();
            *value.pointer_mut(pointer).unwrap() = scalar;
            check(&validator, root.path(), &value, false);
            assert_eq!(
                NoteType::from_bundle(root.path()).unwrap_err().code(),
                "TEMPLATE.BUNDLE_MANIFEST_INVALID"
            );
        }
    }
    base["css_file"] = Value::Null;
    base["note_type"]["name"] = Value::Null;
    base["note_type"]["cloze_field"] = Value::Null;
    base["note_type"]["fields"][0]["name"] = Value::Null;
    for name in [
        "name",
        "browser_front_file",
        "browser_back_file",
        "target_deck",
    ] {
        base["note_type"]["templates"][0][name] = Value::Null;
    }
    check(&validator, root.path(), &base, true);
    for pointer in [
        "/css_file",
        "/note_type/name",
        "/note_type/cloze_field",
        "/note_type/fields/0/name",
        "/note_type/templates/0/name",
        "/note_type/templates/0/browser_front_file",
        "/note_type/templates/0/browser_back_file",
        "/note_type/templates/0/target_deck",
    ] {
        for scalar in [json!(123), json!(true)] {
            let mut value = base.clone();
            *value.pointer_mut(pointer).unwrap() = scalar;
            check(&validator, root.path(), &value, false);
            assert_eq!(
                NoteType::from_bundle(root.path()).unwrap_err().code(),
                "TEMPLATE.BUNDLE_MANIFEST_INVALID"
            );
        }
    }
    // Explicitly quoted scalar-looking strings remain valid string values.
    for text in ["null", "123", "true"] {
        let mut value = manifest();
        value["note_type"]["key"] = json!(text);
        check(&validator, root.path(), &value, true);
    }
}

#[test]
fn bundle_duplicate_yaml_keys_remain_invalid() {
    let root = tempfile::tempdir().unwrap();
    prepare(root.path());
    let yaml = serde_yaml::to_string(&manifest()).unwrap();
    fs::write(
        root.path().join("anki-template.yaml"),
        format!("{yaml}\nformat_version: template-bundle-v2\n"),
    )
    .unwrap();
    assert_eq!(
        NoteType::from_bundle(root.path()).unwrap_err().code(),
        "TEMPLATE.BUNDLE_MANIFEST_INVALID"
    );
}

#[test]
fn bundle_and_project_share_scalar_contract_definitions() {
    let contracts = Path::new(env!("CARGO_MANIFEST_DIR")).join("../contracts/schema");
    let bundle: Value =
        serde_json::from_slice(&fs::read(contracts.join("template-bundle.schema.json")).unwrap())
            .unwrap();
    let project: Value =
        serde_json::from_slice(&fs::read(contracts.join("project-input.schema.json")).unwrap())
            .unwrap();
    for definition in [
        "stableKey",
        "fieldKey",
        "displayName",
        "deckName",
        "exportFilename",
    ] {
        assert_eq!(
            bundle["definitions"][definition], project["definitions"][definition],
            "{definition}"
        );
    }
}

#[test]
fn bundle_manifest_budget_is_checked_before_parsing() {
    let root = tempfile::tempdir().unwrap();
    prepare(root.path());
    let source = serde_yaml::to_string(&manifest()).unwrap();
    let limit = 256 << 10;
    let padded = format!("{source}{}", " ".repeat(limit - source.len()));
    fs::write(root.path().join("anki-template.yaml"), &padded).unwrap();
    let _model = NoteType::from_bundle(root.path()).unwrap();
    fs::write(root.path().join("anki-template.yaml"), format!("{padded} ")).unwrap();
    let error = NoteType::from_bundle(root.path()).unwrap_err();
    assert_eq!(error.code(), "TEMPLATE.BUNDLE_RESOURCE_LIMIT_EXCEEDED");
}

#[test]
fn bundle_export_filename_utf8_budget_is_checked_after_schema_validation() {
    let validator = validator();
    let root = tempfile::tempdir().unwrap();
    prepare(root.path());
    let media = Media::file(root.path().join("asset.txt")).unwrap();
    let mut cases = Vec::new();
    for valid in [
        format!("{}a", "é".repeat(127)),
        format!("{}abc", "中".repeat(84)),
        format!("{}abc", "😀".repeat(63)),
    ] {
        assert_eq!(valid.len(), 255);
        cases.extend([
            (valid.clone(), true),
            (format!("{valid}a"), false),
            (format!("{valid}{}", "a".repeat(17)), false),
        ]);
    }
    cases.push(("é".repeat(128), false));
    for (name, accepted) in cases {
        assert!(name.chars().count() < 255);
        assert_eq!(name.len() <= 255, accepted);
        let mut value = manifest();
        value["assets"] = json!([{"path": "asset.txt", "export_as": name}]);
        assert!(validator.is_valid(&value), "shape schema: {name:?}");
        let direct = media.clone().with_export_name(&name);
        assert_eq!(direct.is_ok(), accepted, "Media filename: {name:?}");
        if let Ok(media) = direct {
            assert_eq!(media.filename(), name);
        } else {
            assert_eq!(direct.unwrap_err().code(), "MEDIA.EXPORT_NAME_INVALID");
        }
        fs::write(
            root.path().join("anki-template.yaml"),
            serde_yaml::to_string(&value).unwrap(),
        )
        .unwrap();
        let loaded = NoteType::from_bundle(root.path());
        assert_eq!(loaded.is_ok(), accepted, "bundle filename: {name:?}");
        if let Err(error) = loaded {
            assert_eq!(error.code(), "MEDIA.EXPORT_NAME_INVALID");
        }
    }
}
