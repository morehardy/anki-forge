---
asset_refs:
  - schema/normalized-ir.schema.json
  - schema/normalization-result.schema.json
---

# Canonical Serialization

Canonical serialization must be stable for semantically equivalent JSON values.
Object keys are serialized in sorted lexicographic order at every nesting level.

Array ordering remains significant and must be preserved as provided by the
semantic producer.

Canonical JSON helpers must not inject extra fields, whitespace-sensitive
transformations, or placeholder identity values. The same logical payload must
serialize to the same byte sequence when the underlying semantic values are
unchanged.
