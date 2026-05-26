# Phase 4 Template/Card Risk Oracle

This oracle records why template ordinal changes and template removal are treated as import/update risks.

Evidence source:

- Existing roadmap statement: docs/api-design.md section 10.1 says Anki cards are associated by note id plus card ordinal, so template order is import-sensitive.
- Repository observation source: writer_core/src/inspect.rs records template `ord`, card-count metadata, and card references from generated artifacts.
- Regression requirement: Phase 4 tests must build a previous APKG, build a current APKG with a removed or reordered template, and assert that the Product report emits `RISK.TEMPLATE_REMOVED` or `RISK.TEMPLATE_REORDER` with diff evidence refs.

Manual acceptance statement:

Changing or removing a template can change which existing cards are generated for the same note identity. Phase 4 therefore blocks high/critical template-card changes unless the user chooses a less strict `fail_on` threshold.

## Automated Regression Tests

- `oracle_template_removed_emits_critical_risk_with_evidence`
- `oracle_template_reorder_emits_high_risk_with_evidence`
