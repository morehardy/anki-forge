---
asset_refs:
  - schema/comparison-context.schema.json
  - schema/merge-risk-report.schema.json
  - schema/normalized-ir.schema.json
  - policies/risk-policy.default.yaml
---

# Merge Risk Semantics

Merge risk reporting is an analysis artifact. It classifies comparison coverage
and communicates what the current normalization run can say about merge
confidence; it does not block or enforce authoring actions by itself.

`comparison_status` describes comparison completeness:

- `complete` means the current normalized artifact had enough baseline context
  for a full comparison.
- `partial` means some comparison context exists, but baseline coverage is
  reduced. `identity_index` baselines are reported as partial because they only
  support identity-level matching.
- `unavailable` means comparison could not be completed from the available
  baseline context.

`comparison_reasons` explains why the classification was chosen. Reason codes
  are descriptive reporting outputs, not policy verdicts. For example,
  `BASELINE_IDENTITY_INDEX_ONLY` explains that only identity-index evidence was
  available, while `BASELINE_UNAVAILABLE` explains that strict comparison was
  requested without a usable baseline fingerprint.

The merge risk report boundary is the emitted normalization result. It should
summarize baseline availability, policy version, and the current artifact
fingerprint without expanding into enforcement decisions or mutation planning.
