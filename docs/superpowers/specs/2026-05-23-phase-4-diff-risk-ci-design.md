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
2. `Project::diff_against_apkg(path)` as a read-only standalone comparison entrypoint.
3. `BuildReport` fields for diff, import risk, and policy result.
4. Product-facing risk classification over existing inspect, diff, and update-safety evidence.
5. JSON report serialization for Rust-generated reports.
6. CLI support for Product-document build with `compare_to`, `fail_on`, and report JSON output.
7. CI-oriented exit behavior and examples.
8. Tests proving Rust and CLI produce the same canonical report projection.

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
- `contract_tools` exposes CLI commands that call the same Rust Product build/report path. It must not reimplement a separate CI pipeline. Existing writer-contract commands such as `contract_tools build --input normalized-ir.json` may remain, but they are not the Phase 4 CI report surface unless they route through the shared report assembler.

This keeps lower layers factual and Product layers interpretive.

Standalone diff uses the same comparison assembler:

```text
Project
  -> lower Product API
  -> normalize Authoring IR
  -> materialize temporary current artifact for inspection
  -> inspect current artifact
  -> inspect previous APKG
  -> artifact diff
  -> semantic/import risk
  -> ProjectDiffReport
```

`diff_against_apkg(...)` must not invent a second comparison path. It may use temporary staging or APKG materialization internally so current project facts are observed through the same writer/inspect layer as `build(compare_to(...))`, but it does not copy or publish a final APKG unless a future option explicitly asks for retained artifacts.

The materialization step is a full temporary writer path: Product lowering, normalization, writer APKG/staging materialization, and inspect all run exactly as they would for `build(compare_to(...))`. It is not an in-memory shortcut. `build(compare_to(...))` and `diff_against_apkg(...)` both call a shared internal comparison assembler; the build path then adds final artifact copy, policy evaluation, report JSON writing, and CI exit behavior.

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

Standalone comparison API:

```rust
let diff = project.diff_against_apkg("previous/jp-core.apkg")?;

assert_eq!(diff.comparison, ComparisonStatus::Complete);
println!("{}", diff.summary());
```

Target report shape:

```text
ProjectDiffReport
  status: BuildStatus
  comparison: ComparisonStatus
  diagnostics: Vec<Diagnostic>
  current_inspect: Option<InspectSummary>
  previous_inspect: Option<InspectSummary>
  update_safety: Option<UpdateSafetySummary>
  diff: Option<BuildDiffSummary>
  risk: Option<ImportRiskReport>
  metrics: ComparisonMetrics
```

The public type may be exported as `anki_forge::diff::DiffReport`; this spec uses `ProjectDiffReport` to distinguish it from `writer_core::DiffReport`. It reuses the same `BuildDiffSummary`, `ImportRiskReport`, `ComparisonStatus`, evidence refs, and diagnostics as `BuildReport`.

```text
ComparisonMetrics
  duration: Duration
```

`Project::diff_against_apkg(...)` does not apply `fail_on` policy and does not write report JSON in the first slice. CI gating belongs to `Project::build(...fail_on...)` and `contract_tools product-build`. Standalone diff is for local inspection, tests, and API users who want comparison data without publishing a new APKG.

`ProjectDiffReport.status` uses the same `BuildStatus` values except `blocked`, which is not produced because no policy is evaluated:

```text
success | invalid | error
```

`success` means the comparison ran to completion. High-risk findings alone do not make standalone diff fail. `invalid` means user-controlled project or baseline input made the comparison incomplete, such as an unreadable previous APKG. `error` means infrastructure or execution failure.

Every standalone diff failure returns:

```text
ProjectDiffError { report: ProjectDiffReport, cause: BuildFailureCause }
```

Standalone diff may return only these `BuildFailureCause` variants:

```text
Diagnostics | Invalid | Io | Internal
```

`PolicyBlocked` and `MissingArtifact` are unreachable for standalone diff because it does not evaluate policy and does not require a published artifact.

Once `Project::diff_against_apkg(...)` is entered on an in-memory `Project`, failures should return `ProjectDiffError` with a partial report whenever possible. If the previous APKG cannot be read or inspected, `diff_against_apkg(...)` returns `Err(ProjectDiffError)` with `report.status = invalid`, `report.comparison = unavailable`, and `RISK.BASELINE_UNAVAILABLE` when enough report state exists. CLI/file parsing failures before a `Project` exists remain invocation failures outside this Rust API contract.

The separate standalone diff API is intentional even though callers could simulate it with a temporary build. It is listed in `docs/api-design.md`, gives users a read-only comparison mental model, avoids accidental publication of disposable APKGs, and makes tests express comparison intent without configuring artifact output or policy.

`Project::diff_against_apkg(...)` always cleans up its temporary staging and APKG materialization in the first slice. It does not accept `BuildOptions` and does not honor `artifacts_dir`; retained comparison artifacts can be added later as an explicit diff option if users need debugging output.

`BuildOptions` gains:

```text
fail_on: Option<RiskLevel>
report_json: Option<PathBuf>
```

Existing `BuildOptions` fields such as `output`, `artifacts_dir`, `inspect`, `compare_to`, `identity_lockfile`, `write_identity_lockfile`, and `update_safety` keep their current meanings. Phase 4 relies on the existing `artifacts_dir` field for explicit staging preservation; it does not add a second staging option.

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
  comparison: ComparisonStatus
  diff: Option<BuildDiffSummary>
  risk: Option<ImportRiskReport>
  policy: BuildPolicyResult
  status: BuildStatus
```

Existing report subtypes keep their current meanings from previous phases: `BuildCounts`, `MediaSummary`, `BuildMetrics`, `InspectSummary`, and `UpdateSafetySummary` are not redesigned in Phase 4. The new Phase 4 subtypes are `ComparisonStatus`, `BuildDiffSummary`, `ImportRiskReport`, `BuildPolicyResult`, and `BuildStatus`.

`ComparisonStatus` is the single authoritative comparison-availability value for both diff and risk:

```text
not_requested | complete | partial | unavailable
```

Nested diff and risk sections must not compute independent comparison status values. They use `BuildReport.comparison`.

`BuildStatus` should be a stable enum-like value serialized as:

```text
success | blocked | invalid | error
```

Warning-only builds keep `status = success`; warnings are represented by diagnostics and report counts, not by a separate build status.

Status definitions:

- `success`: build execution completed, an artifact exists, and no blocking policy or error diagnostic exists.
- `blocked`: build execution completed far enough to produce a policy result, and `fail_on` blocked on risk. The artifact may exist.
- `invalid`: user-controlled input or baseline evidence was invalid, missing, unreadable, schema-invalid, or semantically inconsistent. Examples include invalid Product input, invalid build options, unreadable `compare_to`, invalid lockfile, or validation diagnostics with `Severity::Error`.
- `error`: an infrastructure or execution failure prevented reliable completion. Examples include current-directory lookup failure, runtime asset loading failure, writer IO failure, report JSON write failure, and unexpected internal errors.

Precedence is:

```text
error > invalid > blocked > success
```

If multiple conditions occur, the report status uses the highest-precedence status while preserving all diagnostics, risk findings, and policy evidence that were available.

`diff` is omitted when no baseline is requested. `risk` is still allowed without a baseline when risks can be derived from current diagnostics, but full import/update risk requires `compare_to`. In the first slice, the only baseline-free risk from the listed rules is `RISK.MEDIA_REFERENCE_BROKEN`; all identity, field/template, card-count, and media-removal risks require a baseline.

`report_json` writes the stable report projection whenever a report exists, including policy-blocked and invalid results. Failure to write the report file is an IO failure:

- the in-memory report is still returned inside `BuildError`,
- the artifact path remains in the report if artifact creation already completed,
- the report receives an error diagnostic such as `REPORT.JSON_WRITE_FAILED`,
- report status becomes `error` by precedence,
- CLI exits non-zero.

`BuildReport::ensure_success()` fails when:

1. any error diagnostic exists,
2. artifact is missing,
3. build status is `blocked`, `invalid`, or `error`.

Its public shape remains:

```rust
pub fn ensure_success(&self) -> Result<(), BuildError>
```

Diagnostics use the existing `anki_forge::diagnostics::Diagnostic` shape. Every diagnostic carries `severity: Severity`, where `Severity` is `Error`, `Warning`, or `Info`. `ensure_success()` only treats `Severity::Error` as a diagnostic failure. Warning and info diagnostics remain report evidence.

Every `Project::build(...)` failure returns:

```text
BuildError { report: BuildReport, cause: BuildFailureCause }
```

The report may be partial, but it must exist. CLI failures that happen before entering the Product build path use invocation exit code `1` and do not need to produce a BuildReport.

Minimum `BuildFailureCause` mapping:

```text
Diagnostics: one or more `Severity::Error` diagnostics
MissingArtifact: no artifact path is available when success requires one
PolicyBlocked: `BuildPolicyResult.status = blocked`
Invalid: `BuildStatus = invalid` without a more specific diagnostic cause
Io: infrastructure or filesystem failure, including report JSON write failure
Internal: unexpected internal error when no narrower cause applies; the report must contain an error diagnostic with the message
```

`BuildFailureCause` refines `BuildStatus` for Rust error handling; it is not an independent user-facing status. Multiple causes may map to one status:

```text
success -> no BuildError
blocked -> PolicyBlocked
invalid -> Diagnostics | MissingArtifact | Invalid
error -> Io | Internal
```

When multiple causes map to the same status, choose the most specific cause by this precedence:

```text
Diagnostics > MissingArtifact > Invalid
Io > Internal
```

The artifact may exist in a policy-blocked result. This is intentional so CI can upload both artifact and report for review.

Artifact preservation rules:

- If a final APKG was copied to the explicit output path before a blocked, invalid, or error report is returned, the report preserves that artifact path.
- Temporary staging artifacts are cleaned up unless the caller supplied an explicit `artifacts_dir`.
- Invalid builds that fail before writer execution have no artifact. Invalid builds that occur after final APKG copy, such as an unusable later baseline comparison, preserve the artifact path.
- Report JSON write failure does not delete an already written APKG.

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
  artifact_diff: Option<ArtifactDiffSummary>
  semantic_changes: Vec<SemanticDiffChange>
  summary_counts: DiffSummaryCounts
  limitations: Vec<String>
```

First-slice semantic diff is intentionally narrow. It does not attempt to become a complete editable-project diff. It contains only Product-facing summaries needed to explain risk findings and CI output.

The two diff layers serve different consumers:

- `ArtifactDiffSummary` is debugging evidence. It is a stable Product-level subset and transformation of writer observations, not a field-for-field mirror of `writer_core::DiffReport`.
- `SemanticDiffChange` is user and CI explanation. It groups artifact changes into Product concepts and links them to risk codes.

```text
SemanticDiffChange
  category: notetype | field | template | note_identity | card_count | media | baseline
  selector: String
  change_kind: added | removed | modified | reordered | unavailable
  risk_codes: Vec<String>
  message: String
  source: Option<SourcePath>
```

```text
DiffSummaryCounts
  added: usize
  removed: usize
  modified: usize
  reordered: usize
  uncompared_domains: usize
```

```text
ArtifactDiffSummary
  changes: Vec<ArtifactDiffChange>
  limitations: Vec<String>
```

```text
ArtifactDiffChange
  category: added | removed | modified
  domain: String
  severity: String
  selector: String
  message: String
  evidence_refs: Vec<EvidenceRef>
```

`writer_core::DiffReport` must not learn Product risk semantics. It remains an artifact observation diff. The Product report may derive `ArtifactDiffSummary` from it, but it should not expose the raw writer type as the public Product report contract.

`ComparisonStatus` triggering conditions:

- `not_requested`: no `compare_to` baseline was supplied.
- `complete`: current and previous artifacts were inspected, neither inspect report has `observation_status = unavailable`, and no compared domain is missing from either side.
- `partial`: both artifacts were at least partly inspected, but one or more non-fatal domains are missing or degraded. The report remains actionable and must list the missing or degraded domains in `limitations`.
- `unavailable`: one side cannot be inspected at all, or a required core evidence domain is unavailable such that artifact comparison cannot be trusted. First-slice required core evidence domains are `notetypes`, `templates`, `fields`, `metadata`, and `card_evidence`.

Limitations must identify the affected domain and, when available, the affected selector. Domain-level limitations are acceptable when inspect cannot identify a narrower selector. Selector-level failures inside an otherwise comparable domain produce `partial` unless they make a required core domain unavailable as a whole.

`card_evidence` is a Product-level evidence domain, not necessarily a raw `writer_core::InspectObservations` field. In the first slice it may be derived from inspect metadata card counts and card/note/template references. If card evidence is missing, template/card-structure risk rules that depend on it must be deferred or emit limitations rather than claiming a complete comparison.

Minimum first-slice `card_evidence` is present when both current and previous inspect reports expose total card count and enough card/template ordinal evidence to connect card-count changes to template changes. If only total card count is available, card evidence is degraded and comparison is `partial` for card-dependent rules. If neither total count nor ordinal evidence is available, card evidence is missing.

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
  evidence_refs: Vec<EvidenceRef>
  suggested_action: Option<String>
```

`suggested_action` is optional human-facing guidance. CI logic must not depend on it.

```text
ImportRiskReport
  highest_level: Option<RiskLevel>
  findings: Vec<ImportRiskFinding>
  limitations: Vec<String>
```

```text
EvidenceRef
  kind: diagnostic | diff_change | inspect_observation | update_safety | oracle
  ref_id: String
```

`ref_id` is stable within a single report projection. It does not need to be globally stable across builds.

`oracle` evidence means a reference to behavior evidence outside the current report. First-slice oracle refs use these prefixes:

```text
manual:<scenario-id>
roundtrip:<fixture-id>
source:<source-id>
manual-doc:<citation-id>
```

Every oracle ref must resolve to a repo file, fixture id, or cited source recorded by the tests.

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

First-slice safe-rename rule:

- A field rename is considered proven safe only when current and previous inspect evidence show the same notetype identity and the same field `config_id` or stable field key for that field.
- If stable field identity evidence is missing, ambiguous, or changed, the first slice treats the change as `RISK.FIELD_REMOVED_OR_RENAMED`.
- A display-name-only field change with stable field identity may still be reported as an informational semantic change, but it must not emit the medium-risk removal/rename finding.
- Stable field key and field `config_id` derivation are expected from the existing Phase 1/3 custom-notetype identity work. The implementation plan must verify that inspect exposes this evidence before enabling the safe-rename branch. If inspect cannot expose it yet, the first slice treats field renames as unsafe until that evidence exists.

High-risk and critical rules that depend on Anki import/update behavior require evidence before they ship. Existing Phase 3 update-safety evidence may satisfy GUID and config-id preservation rules. Template ordinal and card-removal risks may cite upstream Anki source, existing manual scenarios, or new Phase 4 oracle evidence. If no evidence exists for a rule, that rule is not enabled as a blocking high/critical finding in the first slice.

`RISK.CARD_COUNT_CHANGED` promotion is deterministic: when the same report also contains `RISK.TEMPLATE_REMOVED`, the card-count finding is upgraded from `Medium` to `High` and links to the template-removal finding through `evidence_refs`. It remains a separate finding so CI can count card-count changes independently.

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

Baseline-unavailable behavior:

- `compare_to` path validation and inspect are attempted before risk classification.
- If the baseline cannot be read or inspected, the build sets `comparison = unavailable` and still creates an `ImportRiskReport` with `RISK.BASELINE_UNAVAILABLE`.
- Policy evaluation still runs against that risk report. With `fail_on(High)`, `BuildPolicyResult.status = blocked`.
- Report status remains `invalid` by precedence because the requested baseline input was unusable, even if policy also blocked.
- CLI exit code follows the final report status precedence.
- CI should gate on top-level `status`: any status other than `success` is failing. `policy.status` explains whether the failure was caused by risk threshold policy in addition to input/build status.

## 10. CLI And JSON Report

The CLI provides the CI-facing projection of the same Rust build/report path.

First-slice CLI input is a serialized `ProductDocument` JSON file that maps to the existing `anki_forge::product::ProductDocument` serde model. This is not the final declarative project format promised by later phases; it is a narrow CI entrypoint over the current Product model.

```bash
contract_tools product-build \
  --manifest contracts/manifest.yaml \
  --product-input project.product.json \
  --apkg-out jp-core.apkg \
  --compare-to previous/jp-core.apkg \
  --fail-on high \
  --report-json build-report.json \
  --output contract-json
```

Existing `contract_tools build --input normalized-ir.json` remains the writer-contract build command. It may gain report support later, but it is not the required Phase 4 CI entrypoint.

CLI argument meanings:

- `--manifest` is required in the first slice. Phase 4 uses it to resolve the contract bundle root, report/build schemas, writer policy, build context, and runtime defaults needed by Product build and report validation.
- `--product-input` is required and points to serialized `ProductDocument` JSON.
- `--apkg-out` is required and is the final APKG output path.
- `--compare-to` is optional and points to the previous APKG baseline.
- `--fail-on` is optional and accepts `info`, `low`, `medium`, `high`, or `critical`.
- `--report-json` is optional and writes the stable report projection to a file.
- `--output` controls stdout mode and accepts `contract-json` or `human`. Only `contract-json` is stable in the first slice. `human` may be a best-effort operator summary such as status, highest risk, and report path, and is not a conformance target.

Output overwrite behavior:

- `--apkg-out` overwrites an existing file when the build reaches final artifact copy.
- `--report-json` overwrites an existing file using an atomic temp-file-and-rename write when possible.
- Failure to overwrite either path is an IO failure represented in the BuildReport when report creation has started.

Manifest contract:

- Phase 4 uses the existing `contracts/manifest.yaml` shape and `assets` map. It does not introduce a new manifest format.
- Required first-slice asset keys are `writer_policy`, `build_context_default`, `writer_policy_schema`, `build_context_schema`, and the Phase 4 report schema key, `build_report_schema`.
- `writer_policy`, `build_context_default`, `writer_policy_schema`, and `build_context_schema` already exist in the current manifest. `build_report_schema` is new Phase 4 work.
- Manifest values are contract-relative paths and must resolve inside the bundle root using the existing runtime asset resolver.

CLI behavior:

- Successful build with no blocking risk exits `0`.
- Warning-only builds exit `0` and include warnings in JSON.
- Validation/build errors exit non-zero and include a report when one is available.
- Pure policy-blocked builds exit non-zero with `status = blocked`; if an invalid or error condition also exists, top-level status and exit code follow precedence.
- `--report-json` writes the same canonical report projection emitted to stdout in `contract-json` mode.

First-slice exit codes:

```text
0 = success
2 = blocked
3 = invalid
4 = error
1 = invocation failure before a BuildReport can be created
```

When status precedence produces `invalid` while policy is also blocked, the CLI exits `3` and the JSON report still includes `policy.status = blocked`.

Exit code `1` is reserved for failures that happen before report creation, including CLI parse errors, missing required flags, invalid enum values such as `--fail-on extreme`, unreadable or missing `--manifest`, and unreadable or unparsable `--product-input`. Exit code `3` requires a BuildReport with `status = invalid`, such as parsed Product input that fails validation or a parsed build request whose `compare_to` baseline cannot be inspected.

The JSON report is a versioned projection, not a raw dump of Rust structs. The first slice ships:

```text
kind: "anki-forge-build-report"
schema_version: "phase4-build-report-v1"
tool_version: facade_api_version()
```

The implementation should add a schema file such as `contracts/schema/build-report.schema.json` and validate CLI contract tests against it.

## 11. CI Example

The documentation should include a GitHub Actions example:

```yaml
- name: Download previous APKG
  uses: actions/download-artifact@v4
  with:
    name: previous-apkg
    path: .

- name: Build Anki package
  run: contract_tools product-build --manifest contracts/manifest.yaml --product-input project.product.json --apkg-out deck.apkg --compare-to previous.apkg --fail-on high --report-json build-report.json

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
- `Project::diff_against_apkg(...)` returns the same comparison, diff, and risk evidence as `Project::build(compare_to(...))` for the same project and baseline, excluding policy and artifact-output fields.
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
- Compare canonical JSON report projection. Volatile fields such as durations or generated temp paths must either be excluded from the stable projection or normalized before comparison.
- Validate the report projection against `contracts/schema/build-report.schema.json`.
- Verify exit codes for success, warning-only, blocked, invalid, and error states.

Oracle/manual tests:

- Each high-risk rule that depends on Anki import behavior must cite or include evidence from manual scenarios, roundtrip oracle, upstream Anki source, or documented Anki behavior.
- Template ord, field/template config id, and GUID preservation need explicit evidence because they affect scheduling and import merge safety.
- The implementation plan must include a risk-rule evidence matrix before coding risk rules. Each rule is marked `enabled` with evidence refs or `deferred` with the reason. Phase 4 first slice may defer rules that lack evidence, but the enabled set must still cover baseline unavailable, broken media references, stable-id/GUID drift, and at least one template/card-structure risk.

## 13. Rollout Order

Recommended implementation order:

1. Prepare the risk-rule evidence matrix and mark each first-version rule `enabled` or `deferred`.
2. Add report projection, schema versioning, stable `BuildStatus`, `RiskLevel`, `ImportRiskReport`, and policy types.
3. Extend `BuildOptions` with `fail_on` and `report_json`.
4. Attach current artifact inspect, previous artifact inspect, artifact diff, and first-slice semantic diff to Product build flow.
5. Implement enabled first-version risk rules from existing update-safety, diagnostics, diff, and oracle evidence.
6. Add `Project::diff_against_apkg(...)` as a read-only facade over the shared comparison assembler.
7. Apply `fail_on` policy and add `PolicyBlocked` failure cause.
8. Add `contract_tools product-build` with ProductDocument input, JSON report output, and shared Rust report path.
9. Add CI documentation and examples.
10. Add oracle-backed tests for Anki-sensitive high-risk rules.

This order gets a stable report carrier in place before risk rules accrete.

## 14. Acceptance Criteria

Phase 4 is complete when all are true:

1. `Project::build(compare_to(...).fail_on(...))` returns a complete report.
2. `Project::diff_against_apkg(...)` returns Product-level diff and risk evidence without publishing a new APKG.
3. `BuildReport` contains diff, risk, and policy sections.
4. Policy-blocked builds preserve the report and, when available, the artifact path.
5. Rust and CLI report projections match for shared fixtures.
6. CLI can emit report JSON to stdout and write it with `--report-json`.
7. CI examples demonstrate blocked, warning-only, and successful flows.
8. High-risk rules have regression tests and Anki-behavior evidence.
9. Product risk semantics stay out of `writer_core`.
10. Existing Phase 3 update-safety behavior remains valid and becomes evidence for the broader Phase 4 risk model.

## 15. Open Decisions Locked For Planning

The following decisions are fixed for the implementation plan:

1. Use the latest `docs/api-design.md` Phase 4 definition: Diff / Risk / CI.
2. Choose Rust + CLI + JSON report for the first delivery slice.
3. Center the design on `BuildReport`, not a CLI-only pipeline.
4. Keep Node and Python parity out of this phase's first slice.
5. Treat `writer_core::DiffReport` as lower-level evidence, not the Product risk model.
