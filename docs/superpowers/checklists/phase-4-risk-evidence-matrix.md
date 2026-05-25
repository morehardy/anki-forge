# Phase 4 Risk Rule Evidence Matrix

Date: 2026-05-23
Source spec: docs/superpowers/specs/2026-05-23-phase-4-diff-risk-ci-design.md

| Rule | Level | Status | Required evidence | Repo evidence refs | First-slice behavior |
| --- | --- | --- | --- | --- | --- |
| RISK.BASELINE_UNAVAILABLE | High | enabled | compare_to requested and previous APKG inspect is unavailable | source:anki_forge/src/update_safety/baseline.rs, source:anki_forge/src/runtime/inspect.rs | Emit finding, set comparison unavailable, allow fail_on to block. |
| RISK.NOTE_GUID_DRIFT | High | enabled | stable note id maps to different GUID through update-safety reconcile evidence | source:anki_forge/src/update_safety/reconcile.rs, roundtrip:update-safety-guid-preservation | Emit finding from UPDATE diagnostics or reconcile conflicts. |
| RISK.NOTETYPE_CONFIG_ID_DRIFT | High | enabled | field/template/notetype merge config id changed unexpectedly | source:anki_forge/src/update_safety/merge_safety.rs, source:writer_core/src/inspect.rs | Emit finding from existing update-safety merge diagnostics. |
| RISK.TEMPLATE_REORDER | High | enabled | template ordinal changes affect card ord update behavior | manual:phase4-template-card-risk, source:writer_core/src/inspect.rs | Emit finding when same template identity changes ord. |
| RISK.TEMPLATE_REMOVED | Critical | enabled | template/card ordinal disappeared from update path | manual:phase4-template-card-risk, source:writer_core/src/inspect.rs | Emit finding when a previous template identity is absent in current evidence. |
| RISK.FIELD_REMOVED_OR_RENAMED | Medium | enabled | field disappeared or rename cannot be proven safe by stable identity | source:anki_forge/src/product/lowering.rs, source:writer_core/src/inspect.rs | Emit finding for field removal/rename in the first slice. Inspect exposes `config_id`, but the first writer diff adapter lacks paired before/after field payloads, so safe-rename proof is not applied and the report records a limitation. |
| RISK.CARD_COUNT_CHANGED | Medium | enabled | current and previous card count differ | source:writer_core/src/inspect.rs, manual:card-count-change-review | Emit finding; promote to High when linked to RISK.TEMPLATE_REMOVED. |
| RISK.MEDIA_REFERENCE_BROKEN | High | enabled | current diagnostics or inspect references show missing/unresolved media | source:anki_forge/src/product/project.rs, source:authoring_core media diagnostics | Emit finding with no baseline required. |
| RISK.MEDIA_REMOVED | Medium | enabled | media filename present in previous artifact is absent from current artifact | source:writer_core/src/diff.rs, source:writer_core/src/inspect.rs | Emit finding from artifact diff media removal. |

## Oracle Reference Files

- manual:phase4-template-card-risk -> docs/oracles/phase-4-template-card-risk.md.
- manual:card-count-change-review -> docs/api-design.md section 10.1 item "card ord changed, existing scheduling may attach to wrong card".
- roundtrip:update-safety-guid-preservation -> anki_forge/tests/update_safety_build_tests.rs.
