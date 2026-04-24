use anki_forge::{BasicNote, Deck, IdentityProvenance, ValidationCode};
use serde_json::json;

fn afid(canonical_payload: &str) -> String {
    format!("afid:v1:{}", blake3::hash(canonical_payload.as_bytes()))
}

#[test]
fn identity_provenance_uses_snake_case_wire_names() {
    assert_eq!(
        serde_json::to_value(IdentityProvenance::ExplicitStableId).expect("serialize"),
        json!("explicit_stable_id")
    );
    assert_eq!(
        serde_json::to_value(IdentityProvenance::InferredFromNoteFields).expect("serialize"),
        json!("inferred_from_note_fields")
    );
    assert_eq!(
        serde_json::to_value(IdentityProvenance::InferredFromNotetypeFields).expect("serialize"),
        json!("inferred_from_notetype_fields")
    );
    assert_eq!(
        serde_json::to_value(IdentityProvenance::InferredFromStockRecipe).expect("serialize"),
        json!("inferred_from_stock_recipe")
    );
}

#[test]
fn roundtrip_preserves_explicit_identity_snapshot_and_duplicate_detection() {
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello").stable_id("es-hola"))
        .expect("add explicit note");

    let raw = serde_json::to_string(&deck).expect("serialize deck");
    let mut roundtripped: Deck = serde_json::from_str(&raw).expect("deserialize deck");

    let err = roundtripped
        .add(BasicNote::new("hola", "again").stable_id("es-hola"))
        .expect_err("duplicate explicit identity should still be blocked");
    assert!(err.to_string().contains("AFID.STABLE_ID_DUPLICATE"));
}

#[test]
fn inferred_afid_without_snapshot_fails_to_deserialize() {
    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "afid:v1:deadbeef",
                    "stable_id": null,
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect_err("missing inferred snapshot must fail");

    assert!(err.to_string().contains("AFID.IDENTITY_SNAPSHOT_MISSING"));
}

#[test]
fn load_time_snapshot_hash_mismatch_is_rejected() {
    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "afid:v1:not-the-hash",
                    "stable_id": null,
                    "resolved_identity": {
                        "stable_id": "afid:v1:not-the-hash",
                        "recipe_id": "basic-front-back",
                        "provenance": "inferred_from_note_fields",
                        "canonical_payload": "{\"front\":\"hola\",\"back\":\"hello\"}",
                        "used_override": false
                    },
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect_err("hash mismatch must fail");

    assert!(err
        .to_string()
        .contains("AFID.IDENTITY_SNAPSHOT_HASH_MISMATCH"));
}

#[test]
fn forged_explicit_provenance_on_afid_note_fails_to_deserialize() {
    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "afid:v1:deadbeef",
                    "stable_id": null,
                    "resolved_identity": {
                        "stable_id": "afid:v1:deadbeef",
                        "provenance": "explicit_stable_id",
                        "used_override": false
                    },
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect_err("explicit provenance cannot be forged onto inferred AFID storage");

    assert!(err
        .to_string()
        .contains("AFID.IDENTITY_SNAPSHOT_INCOMPLETE"));
}

#[test]
fn explicit_provenance_cannot_claim_reserved_afid_namespace() {
    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "afid:v1:deadbeef",
                    "stable_id": "afid:v1:deadbeef",
                    "resolved_identity": {
                        "stable_id": "afid:v1:deadbeef",
                        "provenance": "explicit_stable_id",
                        "used_override": false
                    },
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect_err("explicit provenance cannot claim reserved AFID namespace");

    assert!(err
        .to_string()
        .contains("AFID.IDENTITY_SNAPSHOT_INCOMPLETE"));
}

#[test]
fn legacy_explicit_stable_id_cannot_claim_reserved_afid_namespace() {
    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "legacy-id",
                    "stable_id": "afid:v1:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect_err("legacy explicit stable_id cannot claim reserved AFID namespace");

    assert!(err
        .to_string()
        .contains("AFID.IDENTITY_SNAPSHOT_INCOMPLETE"));
}

#[test]
fn legacy_explicit_duplicate_without_snapshots_deserializes_and_validate_report_finds_it() {
    let deck = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "es-hola",
                    "stable_id": "es-hola",
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            },
            {
                "Basic": {
                    "id": "es-hola",
                    "stable_id": "es-hola",
                    "front": "hola",
                    "back": "again",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect("legacy explicit duplicate ids should still deserialize");

    let report = deck.validate_report().expect("validation report");
    assert!(report.has_errors());
    assert!(report
        .diagnostics()
        .iter()
        .any(|item| item.code == ValidationCode::StableIdDuplicate));
}

#[test]
fn incomplete_inferred_snapshot_fails_to_deserialize() {
    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "afid:v1:deadbeef",
                    "stable_id": null,
                    "resolved_identity": {
                        "stable_id": "afid:v1:deadbeef",
                        "recipe_id": "basic-front-back",
                        "provenance": "inferred_from_note_fields",
                        "used_override": false
                    },
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect_err("inferred snapshots require recipe and payload");

    assert!(err
        .to_string()
        .contains("AFID.IDENTITY_SNAPSHOT_INCOMPLETE"));
}

#[test]
fn inferred_snapshot_with_blank_recipe_id_fails_to_deserialize() {
    let payload = "{\"front\":\"hola\",\"back\":\"hello\"}";
    let stable_id = afid(payload);

    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": stable_id,
                    "stable_id": null,
                    "resolved_identity": {
                        "stable_id": stable_id,
                        "recipe_id": "   ",
                        "provenance": "inferred_from_note_fields",
                        "canonical_payload": payload,
                        "used_override": false
                    },
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect_err("inferred snapshots require a nonblank recipe id");

    assert!(err
        .to_string()
        .contains("AFID.IDENTITY_SNAPSHOT_INCOMPLETE"));
}

#[test]
fn inferred_snapshot_with_empty_canonical_payload_fails_to_deserialize() {
    let stable_id = afid("");

    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": stable_id,
                    "stable_id": null,
                    "resolved_identity": {
                        "stable_id": stable_id,
                        "recipe_id": "basic-front-back",
                        "provenance": "inferred_from_note_fields",
                        "canonical_payload": "",
                        "used_override": false
                    },
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect_err("inferred snapshots require a nonempty canonical payload");

    assert!(err
        .to_string()
        .contains("AFID.IDENTITY_SNAPSHOT_INCOMPLETE"));
}

#[test]
fn snapshot_note_id_mismatch_fails_to_deserialize() {
    let payload = "{\"front\":\"hola\",\"back\":\"hello\"}";
    let stable_id = afid(payload);

    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "afid:v1:other",
                    "stable_id": null,
                    "resolved_identity": {
                        "stable_id": stable_id,
                        "recipe_id": "basic-front-back",
                        "provenance": "inferred_from_note_fields",
                        "canonical_payload": payload,
                        "used_override": false
                    },
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect_err("snapshot stable id must match note id");

    assert!(err
        .to_string()
        .contains("AFID.IDENTITY_SNAPSHOT_NOTE_ID_MISMATCH"));
}

#[test]
fn load_time_duplicate_payload_is_classified() {
    let payload = "{\"front\":\"hola\",\"back\":\"hello\"}";
    let stable_id = afid(payload);

    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": stable_id,
                    "stable_id": null,
                    "resolved_identity": {
                        "stable_id": stable_id,
                        "recipe_id": "basic-front-back",
                        "provenance": "inferred_from_note_fields",
                        "canonical_payload": payload,
                        "used_override": false
                    },
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            },
            {
                "Basic": {
                    "id": stable_id,
                    "stable_id": null,
                    "resolved_identity": {
                        "stable_id": stable_id,
                        "recipe_id": "basic-front-back",
                        "provenance": "inferred_from_note_fields",
                        "canonical_payload": payload,
                        "used_override": false
                    },
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect_err("duplicate inferred payload should fail at load time");

    assert!(err.to_string().contains("AFID.IDENTITY_DUPLICATE_PAYLOAD"));
}

#[test]
fn inferred_collision_fails_to_deserialize() {
    let first_payload = "{\"front\":\"hola\",\"back\":\"hello\"}";
    let second_payload = "{\"front\":\"hola\",\"back\":\"again\"}";
    let stable_id = afid(first_payload);

    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": stable_id,
                    "stable_id": null,
                    "resolved_identity": {
                        "stable_id": stable_id,
                        "recipe_id": "basic-front-back",
                        "provenance": "inferred_from_note_fields",
                        "canonical_payload": first_payload,
                        "used_override": false
                    },
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            },
            {
                "Basic": {
                    "id": stable_id,
                    "stable_id": null,
                    "resolved_identity": {
                        "stable_id": stable_id,
                        "recipe_id": "basic-front-back",
                        "provenance": "inferred_from_note_fields",
                        "canonical_payload": second_payload,
                        "used_override": false
                    },
                    "front": "hola",
                    "back": "again",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect_err("colliding inferred snapshots should fail at load time");

    assert!(err.to_string().contains("AFID.IDENTITY_COLLISION"));
}

#[test]
fn legacy_duplicate_plain_ids_deserialize_and_validate_report_finds_them() {
    let deck = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "legacy-id",
                    "stable_id": null,
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            },
            {
                "Basic": {
                    "id": "legacy-id",
                    "stable_id": null,
                    "front": "adios",
                    "back": "bye",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect("legacy plain duplicate ids should still deserialize");

    let report = deck.validate_report().expect("validation report");
    assert!(report.has_errors());
    assert!(report
        .diagnostics()
        .iter()
        .any(|item| item.code == ValidationCode::StableIdDuplicate));
}
