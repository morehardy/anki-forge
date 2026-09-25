use std::{fs, path::Path};

use ankiforge::{note::Mask, tools::load_project, Field, Media, Note, NoteType, Project, Template};
use contract_tools::{
    contract_manifest_path,
    manifest::{load_manifest, resolve_asset_path},
    schema::load_schema,
};
use serde_json::{json, Value};

fn validator() -> jsonschema::JSONSchema {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    load_schema(resolve_asset_path(&manifest, "project_input_schema").unwrap()).unwrap()
}

fn recipe() -> Value {
    json!({
        "format_version": "ankiforge-project-v1", "namespace": "scalar-parity",
        "models": [{"kind": "custom", "key": "custom",
            "fields": [{"key": "front"}],
            "templates": [{"key": "card", "front": "{{front}}", "back": "{{front}}"}]}],
        "notes": [{"key": "one", "content": {"kind": "basic", "front": "Front", "back": "Back"}}]
    })
}

fn check_loader_and_schema(validator: &jsonschema::JSONSchema, value: &Value, accepted: bool) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("project.json");
    fs::write(&path, serde_json::to_vec(value).unwrap()).unwrap();
    let loaded = load_project(&path, None);
    assert_eq!(
        loaded.is_ok(),
        accepted,
        "loader: {loaded:?}; input: {value}"
    );
    assert_eq!(validator.is_valid(value), accepted, "schema input: {value}");
}

#[test]
fn deck_names_match_runtime_and_loader_at_every_input_location() {
    let validator = validator();
    let mut cases = vec![
        (None, true),
        (Some("".to_owned()), false),
        (Some(":".to_owned()), false),
        (Some("Parent:::Child".to_owned()), false),
        (Some("Parent::::Child".to_owned()), false),
    ];
    for valid in [
        "Default",
        "Parent::Child",
        "A:B::C:D",
        "中文::Café",
        "Two Words",
        "\u{feff}::\u{200b}",
    ] {
        cases.push((Some(valid.to_owned()), true));
    }
    for invalid in [
        "::Child",
        "Parent::",
        ":Child",
        "Parent:",
        " Parent",
        "Parent ",
        "Parent:: Child",
        "Parent ::Child",
    ] {
        cases.push((Some(invalid.to_owned()), false));
    }
    // Every scalar Rust treats as whitespace/control is checked at a
    // component boundary; internal non-control whitespace remains legal.
    for c in (0..=0x10ffff).filter_map(char::from_u32) {
        if c.is_control() || c.is_whitespace() {
            cases.push((Some(format!("Parent::{c}Child")), false));
            cases.push((Some(format!("Parent{c}::Child")), false));
            cases.push((Some(format!("Par{c}ent::Child")), !c.is_control()));
        }
    }
    for (deck, accepted) in cases {
        for location in ["default_deck", "note", "template"] {
            let mut value = recipe();
            let mut project = Project::new("scalar-parity").unwrap();
            let mut note = Note::basic("Front", "Back");
            let runtime_ok = match location {
                "default_deck" => {
                    value["default_deck"] = json!(deck);
                    if let Some(deck) = &deck {
                        project = project.default_deck(deck);
                    }
                    project.add("one", note).is_ok()
                }
                "note" => {
                    value["notes"][0]["deck"] = json!(deck);
                    if let Some(deck) = &deck {
                        note = note.deck(deck);
                    }
                    project.add("one", note).is_ok()
                }
                _ => {
                    value["models"][0]["templates"][0]["target_deck"] = json!(deck);
                    let mut template = Template::new("card").front("{{front}}").back("{{front}}");
                    if let Some(deck) = &deck {
                        template = template.target_deck(deck);
                    }
                    NoteType::builder("custom")
                        .field(Field::new("front"))
                        .template(template)
                        .build()
                        .is_ok()
                }
            };
            assert_eq!(runtime_ok, accepted, "runtime {location}: {deck:?}");
            check_loader_and_schema(&validator, &value, accepted);
        }
    }
}

fn image_bytes() -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../anki_forge/tests/fixtures/public-api/occlusion.png"),
    )
    .unwrap()
}

fn check_masks(masks: Vec<Value>, accepted: bool) {
    let bytes = image_bytes();
    let mut builder = Note::image_occlusion(Media::bytes(bytes.clone(), "image/png").unwrap());
    for mask in &masks {
        builder = builder.mask(Mask::rect(
            mask["key"].as_str().unwrap(),
            mask["x"].as_f64().unwrap(),
            mask["y"].as_f64().unwrap(),
            mask["width"].as_f64().unwrap(),
            mask["height"].as_f64().unwrap(),
        ));
    }
    assert_eq!(
        builder.build().is_ok(),
        accepted,
        "runtime masks: {masks:?}"
    );
    let mut value = recipe();
    value["assets"] =
        json!([{"key": "image", "source": {"kind": "bytes", "data": bytes, "mime": "image/png"}}]);
    value["notes"][0]["content"] =
        json!({"kind": "image_occlusion", "image": "image", "masks": masks});
    check_loader_and_schema(&validator(), &value, accepted);
}

#[test]
fn image_occlusion_mask_count_matches_runtime_and_loader() {
    for count in [0, 1, 500, 501] {
        let masks = (0..count)
            .map(|i| json!({"key": format!("mask-{i}"), "x": 0, "y": 0, "width": 1, "height": 1}))
            .collect();
        check_masks(masks, (1..=500).contains(&count));
    }
}

#[test]
fn image_occlusion_scalar_coordinates_match_runtime_and_loader() {
    for (property, coordinate, accepted) in [
        ("x", -0.1, false),
        ("y", -0.1, false),
        ("width", 0.0, false),
        ("height", 0.0, false),
        ("width", -1.0, false),
        ("height", -1.0, false),
        ("x", 0.0, true),
        ("y", 0.0, true),
        ("width", 0.25, true),
        ("height", 0.25, true),
    ] {
        let mut mask = json!({"key": "mask", "x": 0, "y": 0, "width": 1, "height": 1});
        mask[property] = json!(coordinate);
        check_masks(vec![mask], accepted);
    }
}

#[test]
fn model_and_template_names_match_runtime_and_loader() {
    let validator = validator();
    for name in [
        None,
        Some(""),
        Some(" \u{85} "),
        Some("bad\0name"),
        Some("bad\u{9f}name"),
        Some("模型"),
        Some(" padded name "),
        Some("\u{feff}"),
    ] {
        let accepted =
            name.is_none_or(|name| !name.trim().is_empty() && !name.chars().any(char::is_control));
        for location in ["model", "template"] {
            let mut value = recipe();
            let mut model = NoteType::builder("custom").field(Field::new("front"));
            let mut template = Template::new("card").front("{{front}}").back("{{front}}");
            if location == "model" {
                value["models"][0]["name"] = json!(name);
                if let Some(name) = name {
                    model = model.name(name);
                }
            } else {
                value["models"][0]["templates"][0]["name"] = json!(name);
                if let Some(name) = name {
                    template = template.name(name);
                }
            }
            assert_eq!(
                model.template(template).build().is_ok(),
                accepted,
                "runtime {location}: {name:?}"
            );
            check_loader_and_schema(&validator, &value, accepted);
        }
    }
}

#[test]
fn stable_keys_match_rust_unicode_trimming() {
    let validator = validator();
    for key in [
        "\u{feff}",
        "\u{200b}",
        "中文",
        "two words",
        "\u{85}key",
        "key\u{85}",
        "key\u{9f}",
    ] {
        let accepted =
            !key.trim().is_empty() && key.trim() == key && !key.chars().any(char::is_control);
        let mut value = recipe();
        value["notes"][0]["key"] = json!(key);
        let mut project = Project::new("scalar-parity").unwrap();
        assert_eq!(
            project.add(key, Note::basic("Front", "Back")).is_ok(),
            accepted
        );
        check_loader_and_schema(&validator, &value, accepted);
    }
}

#[test]
fn project_names_match_validation_at_build_time() {
    let validator = validator();
    for name in [
        None,
        Some(""),
        Some(" \u{85} "),
        Some("bad\0name"),
        Some("作品"),
        Some(" padded name "),
        Some("\u{feff}"),
    ] {
        let accepted =
            name.is_none_or(|name| !name.trim().is_empty() && !name.chars().any(char::is_control));
        let mut value = recipe();
        value["name"] = json!(name);
        let mut project = Project::new("scalar-parity").unwrap();
        if let Some(name) = name {
            project = project.name(name);
        }
        project.add("one", Note::basic("Front", "Back")).unwrap();
        let direct = project.build(ankiforge::BuildOptions::temporary());
        assert_eq!(direct.is_ok(), accepted, "runtime project name: {name:?}");
        if let Err(error) = direct {
            assert_eq!(error.code(), "BUILD.NAME_INVALID");
        }
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("project.json");
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        // The loader defers project configuration validation until build,
        // just like the infallible Project::name setter.
        let loaded = load_project(&path, None)
            .unwrap()
            .build(ankiforge::BuildOptions::temporary());
        assert_eq!(loaded.is_ok(), accepted, "loaded project name: {name:?}");
        if let Err(error) = loaded {
            assert_eq!(error.code(), "BUILD.NAME_INVALID");
        }
        assert_eq!(
            validator.is_valid(&value),
            accepted,
            "schema project name: {name:?}"
        );
    }
}

fn asset_recipe(key: &str) -> Value {
    let mut value = recipe();
    value["assets"] = json!([{"key": key, "source": {"kind": "bytes", "data": image_bytes(), "mime": "image/png"}}]);
    value["models"][0]["assets"] = json!([key]);
    value["notes"][0]["content"]["front"] = json!({"kind": "image", "asset": key});
    value["notes"].as_array_mut().unwrap().push(json!({
        "key": "io", "content": {"kind": "image_occlusion", "image": key,
            "masks": [{"key": "mask", "x": 0, "y": 0, "width": 1, "height": 1}]}
    }));
    value
}

#[test]
fn asset_aliases_match_loader_without_imposing_stable_key_rules() {
    let validator = validator();
    for key in [
        "",
        " ",
        "\u{85}",
        "\u{feff}",
        "\0",
        " asset ",
        "asset\nname",
        "path/like:name",
        "图像",
    ] {
        // These are opaque loader map keys, not authored stable identity keys.
        let accepted = !key.trim().is_empty();
        check_loader_and_schema(&validator, &asset_recipe(key), accepted);
    }
}

#[test]
fn export_filenames_match_media_validation_and_loader() {
    let validator = validator();
    let mut names = vec![(None, true)];
    for name in [
        "image.png",
        "图片.png",
        "Café.png",
        "\u{feff}",
        "two words.png",
        "a..b",
        "CONextra",
        "x.COM1",
    ] {
        names.push((Some(name.to_owned()), true));
    }
    for name in [
        "",
        " ",
        " image.png",
        "image.png ",
        ".",
        "..",
        "image.",
        "a/b",
        "a\\b",
        "a<b",
        "a>b",
        "a:b",
        "a\"b",
        "a|b",
        "a?b",
        "a*b",
        "a[b",
        "a]b",
        "a\0b",
        "a\u{9f}b",
        "CON",
        "con.png",
        "PRN.txt",
        "AUX",
        "nul",
        "CLOCK$.png",
        "COM1",
        "com9.png",
        "COM¹",
        "COM².png",
        "COM³",
        "LPT1",
        "lpt9.txt",
        "LPT¹",
        "LPT².png",
        "LPT³",
    ] {
        names.push((Some(name.to_owned()), false));
    }
    names.push((Some("a".repeat(255)), true));
    names.push((Some("a".repeat(256)), false));
    names.push((Some("é".repeat(127)), true));
    for (name, accepted) in names {
        let media = Media::bytes(image_bytes(), "image/png").unwrap();
        let runtime = name
            .as_ref()
            .map_or(Ok(media.clone()), |name| media.with_export_name(name));
        assert_eq!(runtime.is_ok(), accepted, "runtime filename: {name:?}");
        if let Err(error) = runtime {
            assert_eq!(error.code(), "MEDIA.EXPORT_NAME_INVALID");
        }
        let mut value = asset_recipe("image");
        value["assets"][0]["export_as"] = json!(name);
        check_loader_and_schema(&validator, &value, accepted);
    }
}

#[test]
fn byte_source_mime_syntax_matches_media_and_loader() {
    let validator = validator();
    for mime in [
        "bogus",
        "*/*",
        "image/*",
        "*/plain",
        "text/",
        "image/*+xml",
        "text/plain\n",
        "text/plain;broken",
        "text/plain; charset=bad space",
        "image/png; name=\"unterminated",
    ] {
        assert!(
            Media::bytes(Vec::new(), mime).is_err(),
            "runtime MIME {mime:?}"
        );
        let mut value = recipe();
        value["assets"] = json!([{"key":"bytes","source":{"kind":"bytes","data":[],"mime":mime}}]);
        check_loader_and_schema(&validator, &value, false);
    }
    // Include MIME parameters, quoted Unicode, suffixes and parser-supported
    // empty/trailing parameter forms so the schema does not overconstrain input.
    for top in ["text", "application", "*", "x*", "a+b"] {
        for sub in [
            "plain",
            "vnd.example+json",
            "*",
            "*+xml",
            "*+a+b",
            "+",
            "+xml",
            "x+",
        ] {
            for params in [
                "",
                ";",
                "; ",
                ";charset=utf-8",
                "; charset=UTF-8",
                ";a=b;c=d",
                ";a=b;",
                ";a=b; ",
                ";a=",
                ";a=;b=c",
                ";a=\"\"",
                ";a=\"\"\"",
                ";a=\"quoted value\"",
                ";a=\"中文\"; b=c",
                ";a=\"x\" ",
                ";a=\"x\" bad",
                "; a=b; c=",
                "; a=b ",
                ";\tcharset=utf-8",
            ] {
                let mime = format!("{top}/{sub}{params}");
                let accepted = Media::bytes(Vec::new(), &mime).is_ok();
                let mut value = recipe();
                value["assets"] =
                    json!([{"key":"bytes","source":{"kind":"bytes","data":[],"mime":mime}}]);
                check_loader_and_schema(&validator, &value, accepted);
            }
        }
    }
}

#[test]
fn media_type_requires_a_nonempty_subtype() {
    let error = Media::bytes(Vec::new(), "text/").unwrap_err();
    assert_eq!(error.code(), "MEDIA.TYPE_INVALID");
}

#[test]
fn cloze_model_cardinality_and_generation_match_runtime() {
    let validator = validator();
    for cloze in [None, Some("front")] {
        for count in [1, 2] {
            for rule in [
                Value::Null,
                json!({"kind":"anki_default"}),
                json!({"kind":"all","fields":["front"]}),
                json!({"kind":"any","fields":["front"]}),
            ] {
                let mut value = recipe();
                value["models"][0]["cloze_field"] = json!(cloze);
                value["models"][0]["templates"] = json!((0..count)
                    .map(|i| json!({
                        "key":format!("card-{i}"),
                        "front":if cloze.is_some() { "{{cloze:front}}" } else { "{{front}}" },
                        "back":"{{front}}", "generation_rule":rule,
                    }))
                    .collect::<Vec<_>>());
                let accepted = cloze.is_none()
                    || (count == 1 && (rule.is_null() || rule["kind"] == "anki_default"));
                check_loader_and_schema(&validator, &value, accepted);
            }
        }
    }
}

#[test]
fn inline_media_schema_uses_the_default_runtime_byte_budget() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema: Value = serde_json::from_slice(
        &fs::read(resolve_asset_path(&manifest, "project_input_schema").unwrap()).unwrap(),
    )
    .unwrap();
    // Compare the independent published contract with the runtime default
    // without materializing a multi-gigabyte serde_json array at the boundary.
    assert_eq!(
        schema["properties"]["assets"]["items"]["properties"]["source"]["oneOf"][1]["properties"]
            ["data"]["maxItems"],
        ankiforge::media::MediaLimits::default().max_bytes
    );
}

#[test]
fn project_bundle_path_can_refer_to_its_base_directory() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("front.html"), "{{front}}").unwrap();
    fs::write(root.path().join("back.html"), "{{front}}").unwrap();
    fs::write(
        root.path().join("anki-template.yaml"),
        serde_json::to_vec(&json!({
            "format_version":"template-bundle-v2", "note_type":{"key":"bundle",
                "fields":[{"key":"front"}],
                "templates":[{"key":"card","front_file":"front.html","back_file":"back.html"}]}
        }))
        .unwrap(),
    )
    .unwrap();
    let mut value = recipe();
    value["models"] = json!([{"kind":"bundle","path":""}]);
    let path = root.path().join("project.json");
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    load_project(&path, None).unwrap();
    assert!(validator().is_valid(&value));
}

#[test]
fn project_paths_reject_nul_on_all_platforms() {
    let validator = validator();
    for filename in ["\0", "sub/\0name"] {
        let mut value = recipe();
        value["models"] = json!([{"kind":"bundle","path":filename}]);
        check_loader_and_schema(&validator, &value, false);
        let mut value = recipe();
        value["assets"] = json!([{"key":"image","source":{"kind":"file","path":filename}}]);
        check_loader_and_schema(&validator, &value, false);
    }
}

#[cfg(unix)]
#[test]
fn project_file_paths_preserve_whitespace_and_bom_filenames() {
    let root = tempfile::tempdir().unwrap();
    let validator = validator();
    for filename in [" ", "\u{feff}"] {
        fs::write(root.path().join(filename), image_bytes()).unwrap();
        let mut value = recipe();
        value["assets"] = json!([{"key":"image","source":{"kind":"file","path":filename}}]);
        let path = root.path().join("project.json");
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        load_project(&path, None).unwrap();
        assert!(validator.is_valid(&value), "{filename:?}");
    }
}

#[test]
fn image_occlusion_overrides_allow_only_writable_fields() {
    let validator = validator();
    for field in [
        "header",
        "back_extra",
        "comments",
        "image",
        "occlusion",
        "unknown",
        "",
    ] {
        let accepted = matches!(field, "header" | "back_extra" | "comments");
        let mut value = asset_recipe("image");
        value["notes"][1]["content"]["fields"] = json!({field: "override"});
        let note = Note::image_occlusion(Media::bytes(image_bytes(), "image/png").unwrap())
            .mask(Mask::rect("mask", 0.0, 0.0, 1.0, 1.0))
            .build()
            .unwrap()
            .field(field, "override");
        assert_eq!(
            Project::new("fields").unwrap().add("one", note).is_ok(),
            accepted
        );
        check_loader_and_schema(&validator, &value, accepted);
    }
}

#[test]
fn export_filename_utf8_budget_is_checked_after_schema_validation() {
    let validator = validator();
    let media = Media::bytes(image_bytes(), "image/png").unwrap();
    // Include two-, three- and four-byte characters mixed with ASCII. Keep
    // every case below 255 characters so only the runtime byte budget differs.
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
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("project.json");
    for (name, accepted) in cases {
        assert!(name.chars().count() < 255);
        assert_eq!(name.len() <= 255, accepted);
        let mut value = asset_recipe("image");
        value["assets"][0]["export_as"] = json!(name);
        assert!(validator.is_valid(&value), "shape schema: {name:?}");
        let direct = media.clone().with_export_name(&name);
        assert_eq!(direct.is_ok(), accepted, "Media filename: {name:?}");
        if let Ok(media) = direct {
            assert_eq!(media.filename(), name);
        } else {
            assert_eq!(direct.unwrap_err().code(), "MEDIA.EXPORT_NAME_INVALID");
        }
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        let loaded = load_project(&path, None);
        assert_eq!(loaded.is_ok(), accepted, "project filename: {name:?}");
        if let Err(error) = loaded {
            assert_eq!(
                error
                    .downcast_ref::<ankiforge::media::MediaError>()
                    .unwrap()
                    .code(),
                "MEDIA.EXPORT_NAME_INVALID"
            );
        }
    }
}
