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
