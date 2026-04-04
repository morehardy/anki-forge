---
asset_refs:
  - policies/identity-policy.default.yaml
  - schema/normalized-ir.schema.json
---

# Identity Semantics

Deterministic identity resolution is the default path. When no override is
requested, normalization resolves identity as `det:<document_id>`.

`external` and `random` are explicit exception modes for auditable,
object-scoped overrides. Both modes require a non-empty `reason_code`.
`reason` is optional narrative context.

`external` preserves a caller-supplied identifier and resolves as
`ext:<external_id>`. It is invalid when `external_id` is missing.

`random` resolves as `rnd:<nonce>` using a non-deterministic nonce source and
must emit `PHASE2.IDENTITY_RANDOM_OVERRIDE` as a warning diagnostic. Choosing
`random` exits the reproducible identity guarantee for the targeted object.
