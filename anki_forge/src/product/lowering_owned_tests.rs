use super::*;
use serde_json::{json, Value};

fn assert_plans_equal(actual: &LoweringPlan, expected: &LoweringPlan) {
    assert_eq!(
        serde_json::to_vec(&actual.authoring_document).unwrap(),
        serde_json::to_vec(&expected.authoring_document).unwrap()
    );
    assert_eq!(actual.mappings, expected.mappings);
    assert_eq!(actual.source_map, expected.source_map);
    assert_eq!(actual.product_diagnostics, expected.product_diagnostics);
    assert_eq!(actual.lowering_diagnostics, expected.lowering_diagnostics);
}

fn assert_owned_matches_borrowed(raw: Value) -> LoweringPlan {
    let document: ProductDocument = serde_json::from_value(raw).unwrap();
    let original = document.clone();
    let serialized = serde_json::to_vec(&document).unwrap();
    let borrowed = document.lower().unwrap();
    let owned = lower_owned_product_v2_document(
        document.document_id().to_string(),
        document.product_v2().unwrap().clone(),
    );
    assert_plans_equal(&owned, &borrowed);
    assert_eq!(document, original);
    assert_eq!(serde_json::to_vec(&document).unwrap(), serialized);
    assert_plans_equal(&document.lower().unwrap(), &borrowed);
    owned
}

fn custom_notetype() -> Value {
    json!({
        "kind": "custom", "id": "custom", "note_type_kind": "normal",
        "fields": [
            {"name": "Prompt", "key": "prompt", "required": true, "identity": true},
            {"name": "Answer", "key": "answer", "sort": true},
            {"name": "Optional", "key": "optional"}
        ],
        "templates": [{
            "name": "Card", "key": "card", "front": "{{Prompt}}", "back": "{{Answer}}",
            "browser_front": "{{Answer}}", "target_deck": "子牌组",
            "generation_rule": {"kind": "all", "fields": ["prompt"]}
        }],
        "identity": {"kind": "fields", "fields": ["prompt"]},
        "css": ".card { color: red; }", "source_path": "input.note_types[0]"
    })
}

#[test]
fn owned_lowering_preserves_stock_custom_content_and_document_views() {
    for version in ["product-v2", "product-v3"] {
        let wide = "<b>宽🙂 &amp; \"quoted\"</b>".repeat(4096);
        let plan = assert_owned_matches_borrowed(json!({
            "product_document_version": version, "document_id": "owned-content",
            "default_deck_name": "Default",
            "note_types": [
                {"kind": "stock", "id": "basic"},
                {"kind": "stock", "id": "cloze"},
                {"kind": "stock", "id": "image_occlusion"},
                custom_notetype()
            ],
            "notes": [
                {"kind": "stock", "note_type_id": "basic", "deck_name": "牌组",
                 "fields": {"front": {"kind": "html", "value": wide},
                            "back": {"kind": "text", "value": "<>&\"'\n"}},
                 "tags": ["z", "汉字", "z"], "source_path": "input.notes[0]"},
                {"kind": "stock", "note_type_id": "cloze", "deck_name": "牌组",
                 "fields": {"text": {"kind": "html", "value": "{{c1::内容}}"}}},
                {"kind": "stock", "note_type_id": "image_occlusion", "stable_id": "io:1",
                 "deck_name": "牌组", "fields": {
                    "occlusion": {"kind": "html", "value": "mask"},
                    "image": {"kind": "image", "media_id": "image"}}},
                {"kind": "custom", "note_type_id": "custom", "deck_name": "牌组",
                 "fields": {"prompt": {"kind": "html", "value": "问题"},
                            "answer": {"kind": "sound", "media_id": "audio"}},
                 "tags": ["b", "a", "b"], "source_path": "input.notes[3]"},
                {"kind": "custom", "note_type_id": "custom", "stable_id": "explicit",
                 "deck_name": "", "fields": {
                    "prompt": {"kind": "text", "value": " "},
                    "answer": {"kind": "html", "value": ""}}}
            ],
            "media": [
                {"id": "image", "source": {"kind": "file", "path": "image.png"},
                 "export_as": "图像.png"},
                {"id": "audio", "source": {"kind": "inline_base64", "source_label": "audio",
                 "data_base64": "YQ=="}, "export_as": "audio.mp3"}
            ]
        }));
        assert_eq!(plan.authoring_document.notes.len(), 5);
        assert!(
            plan.product_diagnostics.is_empty(),
            "{:?}",
            plan.product_diagnostics
        );
        assert_eq!(plan.authoring_document.notes[0].fields["Front"], wide);
        assert_eq!(
            plan.authoring_document.notes[3].fields["Answer"],
            "[sound:audio.mp3]"
        );
    }
}

#[test]
fn owned_lowering_preserves_diagnostic_order_and_partial_notes() {
    for version in ["product-v2", "product-v3"] {
        let plan = assert_owned_matches_borrowed(json!({
            "product_document_version": version, "document_id": "owned-errors",
            "note_types": [
                {"kind": "stock", "id": "basic", "fields": [
                    {"name": "Front", "key": "front", "required": true}]},
                custom_notetype(),
                {"kind": "future_notetype", "source_path": "input.types[2]"}
            ],
            "notes": [
                {"kind": "stock", "note_type_id": "basic", "deck_name": "D",
                 "fields": {"unknown": {"kind": "html", "value": "x"}},
                 "source_path": "input.notes[0]"},
                {"kind": "custom", "note_type_id": "custom", "stable_id": "invalid",
                 "deck_name": "D", "fields": {
                    "answer": {"kind": "future_content", "value": "x"},
                    "prompt": {"kind": "html", "value": ""},
                    "unknown": {"kind": "html", "value": "x"}},
                 "source_path": "input.notes[1]"},
                {"kind": "custom", "note_type_id": "custom", "stable_id": "missing-media",
                 "deck_name": "D", "fields": {
                    "prompt": {"kind": "sound", "media_id": "absent"},
                    "answer": {"kind": "image", "media_id": "absent"}},
                 "source_path": "input.notes[2]"},
                {"kind": "custom", "note_type_id": "undeclared", "deck_name": "D"},
                {"kind": "stock", "note_type_id": "undeclared", "deck_name": "D"},
                {"kind": "future_note", "source_path": "input.notes[5]"}
            ]
        }));
        assert_eq!(
            plan.product_diagnostics
                .iter()
                .map(|item| item.code)
                .collect::<Vec<_>>(),
            [
                "PRODUCT.UNSUPPORTED_KIND",
                "PRODUCT.FIELD_UNKNOWN",
                "PRODUCT.REQUIRED_FIELD_MISSING",
                "PRODUCT.FIELD_UNKNOWN",
                "PRODUCT.FIELD_CONTENT_KIND_UNSUPPORTED",
                "PRODUCT.REQUIRED_FIELD_MISSING",
                "PRODUCT.MEDIA_MISSING",
                "PRODUCT.MEDIA_MISSING",
                "PRODUCT.CUSTOM_NOTE_TYPE_MISSING",
                "PRODUCT.STOCK_NOTE_TYPE_MISSING",
                "PRODUCT.UNSUPPORTED_KIND"
            ]
        );
        assert_eq!(plan.authoring_document.notes.len(), 1);
        assert_eq!(plan.authoring_document.notes[0].id, "missing-media");
    }
}

#[test]
fn owned_lowering_preserves_duplicate_field_keys_names_and_notetype_resolution() {
    for version in ["product-v2", "product-v3"] {
        let mut notetype = custom_notetype();
        notetype["fields"] = json!([
            {"name": "Prompt", "key": "prompt", "required": true},
            {"name": "Other", "key": "prompt", "required": true},
            {"name": "Answer", "key": "answer"},
            {"name": "Answer", "key": "optional"}
        ]);
        let mut replaced = notetype.clone();
        replaced["fields"][1]["name"] = json!("Last declaration");
        let plan = assert_owned_matches_borrowed(json!({
            "product_document_version": version, "document_id": "owned-duplicates",
            "note_types": [notetype, replaced],
            "notes": [{
                "kind": "custom", "note_type_id": "custom", "stable_id": "duplicate:1",
                "deck_name": "D", "fields": {
                    "prompt": {"kind": "html", "value": "<b>keep both</b>"},
                    "answer": {"kind": "html", "value": "overwritten"},
                    "optional": {"kind": "html", "value": ""}
                }
            }]
        }));
        assert_eq!(plan.authoring_document.notes.len(), 1);
        let fields = &plan.authoring_document.notes[0].fields;
        assert_eq!(fields["Prompt"], "<b>keep both</b>");
        assert_eq!(fields["Last declaration"], "<b>keep both</b>");
        assert_eq!(fields["Answer"], "");
    }
}

#[test]
fn owned_lowering_preserves_custom_cloze_and_transport_diagnostics() {
    let mut notetype = custom_notetype();
    notetype["note_type_kind"] = json!("cloze");
    notetype["cloze_field"] = json!("prompt");
    notetype["templates"] = json!([{
        "name": "Cloze", "key": "cloze", "front": "{{cloze:Prompt}}", "back": "{{cloze:Prompt}}"
    }]);
    let plan = assert_owned_matches_borrowed(json!({
        "product_document_version": "product-v3", "document_id": "owned-cloze",
        "note_types": [notetype], "notes": [{
            "kind": "custom", "note_type_id": "custom", "deck_name": "D",
            "fields": {"prompt": {"kind": "html", "value": "{{c1::句子}}"}}
        }]
    }));
    assert_eq!(plan.authoring_document.notes.len(), 1);
    assert!(plan.product_diagnostics.is_empty());

    let plan = assert_owned_matches_borrowed(json!({
        "product_document_version": "product-v999", "document_id": "unknown-version"
    }));
    assert_eq!(
        plan.product_diagnostics[0].code,
        "PRODUCT.VERSION_UNSUPPORTED"
    );
}

#[test]
fn owned_lowering_moves_html_allocations_into_authoring_fields() {
    let document: ProductDocument = serde_json::from_value(json!({
        "product_document_version": "product-v3", "document_id": "owned-storage",
        "note_types": [{"kind": "stock", "id": "basic"}, custom_notetype()],
        "notes": [
            {"kind": "stock", "note_type_id": "basic", "stable_id": "stock:1", "deck_name": "D",
             "fields": {"front": {"kind": "html", "value": "stock HTML".repeat(4096)}}},
            {"kind": "custom", "note_type_id": "custom", "stable_id": "custom:1", "deck_name": "D",
             "fields": {"prompt": {"kind": "html", "value": "custom HTML".repeat(4096)}}}
        ]
    }))
    .unwrap();
    let payload = document.product_v2().unwrap().clone();
    let pointers = payload
        .notes
        .iter()
        .map(|note| {
            let content = match note {
                ProductNoteV2::Stock(note) => &note.fields["front"],
                ProductNoteV2::Custom(note) => &note.fields["prompt"],
                ProductNoteV2::Unknown(_) => unreachable!(),
            };
            let ProductFieldContentV2::Html { value } = content else {
                unreachable!()
            };
            value.as_ptr()
        })
        .collect::<Vec<_>>();
    let plan = lower_owned_product_v2_document("owned-storage".to_string(), payload);
    assert_eq!(
        plan.authoring_document.notes[0].fields["Front"].as_ptr(),
        pointers[0]
    );
    assert_eq!(
        plan.authoring_document.notes[1].fields["Prompt"].as_ptr(),
        pointers[1]
    );
}
