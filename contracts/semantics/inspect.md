---
asset_refs:
  - schema/inspect-report.schema.json
---

# Inspect

Inspection reports are stable observation models. They summarize what was
observed from staging or packaged output without collapsing into a raw byte
dump.

The report boundary includes the observation model version, source identity,
fingerprint, observation completeness, missing domains, degradation reasons,
and the structured observation buckets required by the schema.

Inspection must preserve compatibility-relevant structure and avoid packaging
noise that does not help compare writer outputs.
