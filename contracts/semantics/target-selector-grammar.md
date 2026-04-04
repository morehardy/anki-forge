---
asset_refs:
  - schema/normalized-ir.schema.json
---

# Target Selector Grammar

`target_selector` is a logical selector grammar with the canonical shape
`kind[k='v']` or `kind[k1='v1',k2='v2']`.

- `kind` identifies the logical target type.
- Each predicate is a `key='value'` pair.
- Array index selectors are forbidden. Any selector fragment like `[12]`
  must be rejected with `ArrayIndexNotAllowed`.
- Resolution must be deterministic:
  - zero matching targets maps to `PHASE2.SELECTOR_UNMATCHED`
  - more than one matching target maps to `PHASE2.SELECTOR_AMBIGUOUS`
