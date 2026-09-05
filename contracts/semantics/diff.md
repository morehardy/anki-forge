---
asset_refs:
  - schema/diff-report.schema.json
---

# Diff

Diff reports describe evidence and compatibility hints. They compare two
inspection reports and summarize what changed, where the comparison was
limited, and what the change implies.

The diff model is a reporting surface, not a gate. It carries comparison
completeness, unmatched domains, comparison limitations, and structured change
entries with selectors and optional evidence references.

Diff output should stay focused on the observable delta between inspection
reports and should not decide workflow success or failure by itself.

All nine observation buckets are compared: `notetypes`, `fields`, `templates`,
`media`, `field_metadata`, `browser_templates`, `template_target_decks`, `metadata`,
and `references`. Added, removed, and modified extended observations must not
disappear from the artifact diff. Field metadata and browser appearance changes
are low-severity artifact changes; target-deck declaration changes also yield
the semantic risk `RISK.TEMPLATE_TARGET_DECK_CHANGED` at medium level.

Missing domains on either side, including unknown future domain names, are
reported in `uncompared_domains` and make the comparison partial. Different
observation model versions also make it partial. An empty delta in a partial
comparison means only that no changes were detected in the compared domains;
it must not be summarized as an unqualified compatibility success.
