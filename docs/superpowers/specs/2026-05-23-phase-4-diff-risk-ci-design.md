# Phase 4 Diff / Risk / CI Design

- Date: 2026-05-23
- Status: Approved in brainstorming, written for planning handoff
- Scope: latest `docs/api-design.md` Phase 4: Diff / Risk / CI
- Supersedes for roadmap purposes: the older Phase 4 label used by `2026-04-06-phase-4-language-bindings-dx-design.md`

## 1. Purpose

Phase 4 turns the existing Product API, BuildReport, inspect, diff, and update-safety foundations into a CI-ready release-safety product.

The phase answers one user question:

> If I build this deck and import it over a previous version, is it safe?

The answer must be available through both Rust and CLI surfaces. The Rust Product API is the normative implementation path. The CLI is a machine-readable projection of the same report for CI.

This phase does not redesign Product authoring, media registration, or Phase 3 identity/update-safety semantics. It composes them into a complete report-driven build flow.

## 2. Selected Approach

Three approaches were considered.

1. `BuildReport` centered.
   `Project::build(compare_to(...))` produces a complete report with artifact, inspect, diff, risk, policy, and diagnostics. CLI JSON is a projection of that same report.
2. CLI report centered.
   The CLI would orchestrate build, inspect, diff, and risk primarily for CI, with the Rust Product API staying thinner.
3. Independent diff/risk tools first.
   Build separate inspect/diff/risk APIs and commands before integrating them into `Project::build`.

The selected approach is `BuildReport` centered.

Reasons:

- It matches `docs/api-design.md`: `BuildReport` is the final user-visible truth.
- It keeps Rust API and CLI behavior aligned.
- It prevents a CLI-only CI path from diverging from Product builds.
- It lets Phase 3 `update_safety` become part of the full risk model instead of a parallel concept.
- It gives users the primary workflow they expect: build once, receive the artifact and safety report together.

## 3. Scope

Phase 4 includes:

1. `Project::build(BuildOptions::compare_to(...).fail_on(...))`.
2. `BuildReport` fields for diff, import risk, and policy result.
3. Product-facing risk classification over existing inspect, diff, and update-safety evidence.
4. JSON report serialization for Rust-generated reports.
5. CLI support for build with `compare_to`, `fail_on`, and report JSON output.
6. CI-oriented exit behavior and examples.
7. Tests proving Rust and CLI produce the same canonical report projection.

Phase 4 excludes:

1. Full Python Product API release.
2. Node/Python parity for this new report surface.
3. APKG import back into an editable `Project`.
4. Native Anki scheduling/revlog migration.
5. A new semantic writer core. `writer_core` remains artifact-observation focused.

## 4. Architecture

The report-driven build flow is:

```text
Project
  -> lower Product API
  -> normalize Authoring IR
  -> build APKG
  -> inspect current artifact
  -> inspect previous APKG when compare_to exists
  -> artifact diff
  -> semantic/import risk
  -> apply fail_on policy
  -> BuildReport
  -> Rust return value / CLI JSON projection
```

Ownership boundaries:

- `anki_forge::build` owns `BuildOptions`, `BuildReport`, `RiskLevel`, policy application, and report serialization types.
- `anki_forge::diff` owns Product-facing diff summaries and wraps lower-level `writer_core::DiffReport` without moving Product semantics into `writer_core`.
- `anki_forge::risk` owns import/update risk classification and aggregates evidence from update safety, artifact diff, inspect reports, and diagnostics.
- `writer_core` continues to own APKG/staging inspection and artifact-level `DiffReport`.
- `contract_tools` exposes CLI commands that call the same Rust Product build/report path. It must not reimplement a separate CI pipeline.

This keeps lower layers factual and Product layers interpretive.

## 5. Rust API

Phase 4 extends the existing build API rather than introducing a second build entrypoint.

Target usage:

```rust
let report = project.build(
    BuildOptions::new()
        .output("jp-core.apkg")
        .compare_to("previous/jp-core.apkg")
        .fail_on(RiskLevel::High)
        .report_json("build-report.json")
)?;

report.ensure_success()?;
```

`BuildOptions` gains:

```text
fail_on: Option<RiskLevel>
report_json: Option<PathBuf>
```

Existing `compare_to` remains the previous APKG baseline input. Its Phase 4 meaning expands from Phase 3 update-safety evidence to full diff/risk evidence.

Repeated single-value setters keep last-call-wins behavior, consistent with existing builder methods.

## 6. BuildReport Shape

`BuildReport` remains the single result object for successful, blocked, and partially failed builds.

Target fields:

```text
BuildReport
  artifact: Option<ApkgArtifact>
  counts: BuildCounts
  media: MediaSummary
  diagnostics: Vec<Diagnostic>
  metrics: BuildMetrics
  inspect: Option<InspectSummary>
  update_safety: Option<UpdateSafetySummary>
  diff: Option<BuildDiffSummary>
  risk: Option<ImportRiskReport>
  policy: BuildPolicyResult
  status: BuildStatus
```

`BuildStatus` should be a stable enum-like value serialized as:

```text
success | blocked | invalid | error
```

Warning-only builds keep `status = success`; warnings are represented by diagnostics and report counts, not by a separate build status.

`diff` is omitted when no baseline is requested. `risk` is still allowed without a baseline when risks can be derived from current diagnostics, but full import/update risk requires `compare_to`.

`report_json` writes the stable report projection whenever a report exists, including policy-blocked and invalid results. Failure to write the report file is an IO failure and should be represented in the returned `BuildError`.

`BuildReport::ensure_success()` fails when:

1. any error diagnostic exists,
2. artifact is missing,
3. build status is `blocked`, `invalid`, or `error`,
4. policy status is blocked.

Policy blocking returns a `BuildError` with the full report attached:

```text
BuildFailureCause::PolicyBlocked
```

The artifact may exist in a policy-blocked result. This is intentional so CI can upload both artifact and report for review.

## 7. Diff Model

Phase 4 has two diff levels.

`ArtifactDiff` answers:

> What observed APKG facts changed?

This level primarily wraps or references `writer_core::DiffReport`. It compares inspect observations such as notetypes, templates, fields, media, metadata, and references.

`SemanticDiff` answers:

> What do those changes mean for Product API users and Anki import/update behavior?

This level derives user-facing summaries from artifact diff, Product source maps, identity evidence, and update-safety results.

The BuildReport should keep raw lower-level diff evidence available while offering a concise Product-facing summary:

```text
BuildDiffSummary
  comparison_status
  artifact_diff
  semantic_changes
  summary_counts
  limitations
```

`writer_core::DiffReport` must not learn Product risk semantics. It remains an artifact observation diff.

## 8. Risk Model

Risk does not replace diagnostics or diff.

```text
diagnostics = validation/build/execution problems
diff = observed changes
risk = import/update interpretation of changes
policy = should this build block?
```

Risk levels:

```text
Info | Low | Medium | High | Critical
```

Target finding shape:

```text
ImportRiskFinding
  code: String
  level: RiskLevel
  category: String
  message: String
  source: Option<SourcePath>
  evidence_refs: Vec<String>
  suggested_action: Option<String>
```

First-version risk rules:

| Code | Default Level | Meaning |
| --- | --- | --- |
| `RISK.BASELINE_UNAVAILABLE` | High | `compare_to` was requested but the previous APKG could not be inspected completely. |
| `RISK.NOTE_GUID_DRIFT` | High | Same stable id maps to a different derived/current GUID than the previous artifact. |
| `RISK.NOTETYPE_CONFIG_ID_DRIFT` | High | Notetype, field, or template stable merge identity changed unexpectedly. |
| `RISK.TEMPLATE_REORDER` | High | Template ordinal changed and may affect existing card scheduling. |
| `RISK.TEMPLATE_REMOVED` | Critical | A template/card ordinal disappeared from the update path. |
| `RISK.FIELD_REMOVED_OR_RENAMED` | Medium | A field disappeared or changed in a way that cannot be proven as a safe rename. |
| `RISK.CARD_COUNT_CHANGED` | Medium | Card count changed unexpectedly. It may be promoted when linked to template removal. |
| `RISK.MEDIA_REFERENCE_BROKEN` | High | A referenced media file is missing or unresolved. |
| `RISK.MEDIA_REMOVED` | Medium | Previously present media is absent from the new artifact. |

Risk evidence sources:

- Phase 3 `update_safety` summary and diagnostics.
- Current and previous `InspectReport`.
- `writer_core::DiffReport`.
- Product lowering source maps for user-facing source paths.
- Existing manual/oracle evidence for Anki import-sensitive behavior.

## 9. Policy

`fail_on(RiskLevel)` blocks builds whose highest risk level is greater than or equal to the threshold.

Policy output:

```text
BuildPolicyResult
  status: passed | blocked | not_evaluated
  threshold: Option<RiskLevel>
  highest_risk: Option<RiskLevel>
  blocking_findings: Vec<String>
```

Default behavior:

- If no `fail_on` is set, risk findings are reported but do not block.
- If `compare_to` is set and the baseline is unreadable, `RISK.BASELINE_UNAVAILABLE` is emitted.
- With `fail_on(High)`, unreadable baseline blocks.
- Error diagnostics still fail independently of risk policy.

## 10. CLI And JSON Report

The CLI provides the CI-facing projection of the same Rust build/report path.

Preferred command shape once Product project input exists:

```bash
anki-forge build \
  --input project.json \
  --output jp-core.apkg \
  --compare-to previous/jp-core.apkg \
  --fail-on high \
  --report-json build-report.json \
  --output-format contract-json
```

If Product project input is not ready, the first implementation may expose a narrower command that accepts the current supported build input format. That narrower command must still call shared Rust report-generation code.

CLI behavior:

- Successful build with no blocking risk exits `0`.
- Warning-only builds exit `0` and include warnings in JSON.
- Validation/build errors exit non-zero and include a report when one is available.
- Policy-blocked builds exit non-zero with `status = blocked`.
- `--report-json` writes the same canonical report projection emitted to stdout in `contract-json` mode.

The JSON report should be stable enough for CI and artifact upload, but it should be versioned separately from internal Rust structs so implementation details can evolve.

## 11. CI Example

The documentation should include a GitHub Actions example:

```yaml
- name: Build Anki package
  run: anki-forge build --compare-to previous.apkg --fail-on high --report-json build-report.json

- name: Upload build report
  uses: actions/upload-artifact@v4
  with:
    name: anki-forge-build-report
    path: build-report.json
```

The example should show how CI distinguishes:

- build failure,
- policy-blocked high-risk update,
- warning-only build,
- successful update-safe build.

## 12. Testing Strategy

Rust unit tests:

- `RiskLevel` ordering.
- `fail_on` threshold behavior.
- `BuildPolicyResult` aggregation.
- `BuildReport::ensure_success()` behavior for `blocked`.
- report JSON projection stability.

Product build integration tests:

- `Project::build(compare_to(...))` includes diff, risk, and policy.
- policy-blocked result carries artifact and report when artifact generation completed.
- stable id/GUID drift becomes high risk.
- template reorder and template removal become high or critical risk.
- field removal or unsafe rename becomes medium or higher risk.
- missing media reference becomes high risk.
- card count change is reported and promoted when linked to template removal.

Writer diff tests:

- `writer_core::diff_reports()` remains artifact-observation only.
- Product risk findings are not asserted in writer tests.

CLI contract tests:

- Run the same fixture through Rust API and CLI.
- Compare canonical JSON report projection.
- Verify exit codes for success, warning-only, blocked, invalid, and error states.

Oracle/manual tests:

- Each high-risk rule that depends on Anki import behavior must cite or include evidence from manual scenarios, roundtrip oracle, upstream Anki source, or documented Anki behavior.
- Template ord, field/template config id, and GUID preservation need explicit evidence because they affect scheduling and import merge safety.

## 13. Rollout Order

Recommended implementation order:

1. Add report serialization projection and stable `BuildStatus`.
2. Add `RiskLevel`, `ImportRiskReport`, and policy types.
3. Extend `BuildOptions` with `fail_on` and `report_json`.
4. Attach current artifact inspect and previous artifact inspect to Product build flow.
5. Attach artifact diff to `BuildReport`.
6. Implement first-version risk rules from existing update-safety, diagnostics, and diff evidence.
7. Apply `fail_on` policy and add `PolicyBlocked` failure cause.
8. Add CLI flags and JSON report output by calling the shared Rust build path.
9. Add CI documentation and examples.
10. Add oracle-backed tests for Anki-sensitive high-risk rules.

This order gets a stable report carrier in place before risk rules accrete.

## 14. Acceptance Criteria

Phase 4 is complete when all are true:

1. `Project::build(compare_to(...).fail_on(...))` returns a complete report.
2. `BuildReport` contains diff, risk, and policy sections.
3. Policy-blocked builds preserve the report and, when available, the artifact path.
4. Rust and CLI report projections match for shared fixtures.
5. CLI can emit report JSON to stdout and write it with `--report-json`.
6. CI examples demonstrate blocked, warning-only, and successful flows.
7. High-risk rules have regression tests and Anki-behavior evidence.
8. Product risk semantics stay out of `writer_core`.
9. Existing Phase 3 update-safety behavior remains valid and becomes evidence for the broader Phase 4 risk model.

## 15. Open Decisions Locked For Planning

The following decisions are fixed for the implementation plan:

1. Use the latest `docs/api-design.md` Phase 4 definition: Diff / Risk / CI.
2. Choose Rust + CLI + JSON report for the first delivery slice.
3. Center the design on `BuildReport`, not a CLI-only pipeline.
4. Keep Node and Python parity out of this phase's first slice.
5. Treat `writer_core::DiffReport` as lower-level evidence, not the Product risk model.
