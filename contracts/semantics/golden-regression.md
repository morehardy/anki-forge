---
asset_refs:
  - schema/inspect-report.schema.json
  - schema/diff-report.schema.json
---

# Golden Regression

Golden regression checks use case-derived `inspect-report` and `diff-report`
artifacts as stable expectations.

Updating a golden requires verifying that the difference is intentional and
compatibility-relevant, not just a formatting or packaging noise change.

Golden files are an observation contract, so they should stay aligned with the
schema-governed report shape rather than embedding ad hoc output formats.
