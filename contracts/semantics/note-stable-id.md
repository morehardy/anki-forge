---
asset_refs:
  - schema/note-identity-fixture.schema.json
  - fixtures/index.yaml
  - errors/error-registry.yaml
---

# Note Stable ID Semantics

`afid:v1:*` note identity is computed from a structured payload serialized by
`contracts/semantics/canonical-serialization.md`.

The payload object contains these semantic fields:

1. `algo_version`
2. `recipe_id`
3. `notetype_family`
4. `notetype_key`
5. `components`

Object keys must be serialized in lexicographic order at every nesting level by
the existing canonical JSON helper. Recipe implementations must not depend on
Rust struct declaration order for AFID bytes.

All recipe text normalization uses Unicode NFC and newline normalization only.
Identity normalization must not trim leading or trailing whitespace.

Recipe ids are stable compatibility boundaries:

1. `basic.core.v1`
2. `cloze.core.v2`
3. `io.core.v2`

Changing the meaning of any recipe input, normalization rule, canonical field, or error behavior requires a new `recipe_id`.

`ResolvedIdentitySnapshot` persists the resolver output used at add-time:

1. `stable_id`
2. `recipe_id` when inferred
3. `provenance`
4. `canonical_payload` when inferred
5. `used_override`

Deserialize-time rebuild must use the persisted snapshot and must not re-resolve inferred identity under the current code.

For inferred identities, the snapshot is the identity source of truth after deserialize. Rebuild must verify that `note.id == snapshot.stable_id` and that `snapshot.stable_id == afid:v1:<blake3(snapshot.canonical_payload)>`. Rebuild does not compare `canonical_payload` back to the current note fields.
