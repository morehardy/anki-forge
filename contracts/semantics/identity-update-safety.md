# Identity Update Safety Semantics

Phase 3 update safety is built around `identity-index-v1`, `identity-lockfile-v1`, and `identity-note-v1`.

The only Phase 3 GUID derivation version is `guid.raw-stable-id.v1`. It sets `current_guid_candidate` to the resolved Product `stable_id` with no truncation or hashing. Changing this rule requires a new `guid_derivation_version`.

`IdentityIndex.source_ref` uses stable logical values:

- `current`
- `baseline.previous_apkg.primary`
- `baseline.identity_lockfile.primary`

Lockfile JSON must use lexicographic object-key ordering by Unicode scalar value after JSON string decoding. Arrays with semantic order preserve that order. Identity entries are sorted by `stable_id`.

Limitations describe source evidence and diagnostics describe build events. Implementations must derive overlapping values from one internal classifier pass.
