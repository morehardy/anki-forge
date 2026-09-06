# Changelog

All notable changes to the Rust Distribution are documented here. The crate
follows Semantic Versioning; before 1.0, breaking public API changes require a
new minor release.

## [Unreleased]

### Fixed

- Clean up only build-owned file-media input copies after normalization,
  including failed preparation, so repeated and incremental exports can reuse
  an artifact directory without deleting caller-owned sources or aliases.
- Let internal-tools Package callers select finite inspection budgets with
  `build_with_limits`; result inspection retains the selected budgets. Default
  exports continue to enforce resource limits before publication.
- Import Decks into a single editable Project state, preserving raw HTML,
  identity evidence, source locations, stock declaration order, and media.
  Project additions no longer fail late or silently lose media after import.
- Share versioned-document and Project builds through private pipeline stages
  and one report finalizer. Late failures preserve completed counts and evidence;
  candidates remain private until inspection and policy acceptance.
- Own temporary APKG output with shared artifact handles instead of leaking
  the entire staging workspace. Late errors retain published artifact ownership.
- Batch collection schema creation and compact into a private temporary database
  before packaging, retaining compaction and atomic output publication. Avoid
  repeated canonical-JSON sorting, source-map cloning, and media parsing for
  text that cannot contain a supported reference.
- Populate APKG note/card data in one transaction with reusable SQL statements,
  and index actual cards and explicit Project stable IDs instead of repeatedly
  scanning all prior entries. Failed writes still preserve published artifacts.
- Reuse typed staging data internally while retaining staging files and the
  owned `StagingPackage` interface. Builds without a comparison baseline inspect
  actual APKG contents without constructing unused observation JSON/fingerprints;
  standalone inspection and baseline comparison retain their full reports.
- Avoid rendering custom fields twice and use field keys for source-name lookups.
- Register every built-in diagnostic and risk code and check production source
  coverage. Discover semantics from the manifest and require exact baseline
  change evidence with version bumps in repository/release contract gates.
- Bound APKG inspection, including ZIP/ZIP64 indexing, nested zstd windows,
  per-entry and cumulative expansion. Stream media hashes and temporary
  collections; expose `InspectLimits` overrides and terminal resource diagnostics.
  Include the documented security acceptance limits in contract bundle `0.5.0`.
- Reject output, report, or writable lockfile aliases of the comparison baseline
  before writes, including symlinks, hard links, and the implicit artifact package.
- Check actual staging-manifest aliases and recheck newly created destinations
  before lockfile/report writes to preserve files on case-insensitive filesystems.
- Keep outputs, retained packages, baselines, and identity lockfiles outside
  writable staging/media directories so policy rejection preserves their bytes.
- Avoid an extra temporary APKG copy when only an explicit output is retained.
- Create private candidates on the artifact workspace's filesystem and reserve
  lockfile temporary files exclusively to avoid truncating aliased inputs/outputs.
- Reuse one baseline inspection for identity reconciliation and diff; publish
  APKG outputs and identity lockfiles only after comparison and risk gates pass.
  Blocked reports retain diff/risk evidence without an unpublished artifact path.
- Write hierarchical deck names with Anki's native `U+001F` separator and
  include all parent decks in generated APKG collections.
- Deduplicate deck aliases with Anki's Unicode case-insensitive comparison and
  use the same canonical human deck names in staging and APKG observations.
- Preserve numeric Anki notetype IDs across reordering, insertion, and baselined
  updates; carry model assignments through staging, APKG inspection, and lockfiles.
- Detect field additions and removals with lockfile-only update safety, and compare
  all nine inspect observation domains without overstating partial comparisons.
- Advance modification times for changed notes using full-content baseline
  revisions, while preserving unchanged times and reproducible builds. This fixes
  answer/tag changes being skipped by Anki because every export used time `1`.
- Preserve existing lockfiles when report-only builds lack verified baseline
  identity/revision evidence; rejected lockfiles remain high risk for policy gates.
- Version the expanded inspection evidence as `phase3-inspect-v2`; cross-version
  saved-report comparisons are partial and supported by Node/Python bindings.
- Preserve single empty fields during APKG inspection so unchanged notes retain
  their full-content revision and modification time.
- Coalesce diagnostic and diff evidence for the same removed field, retaining the
  higher risk level without duplicating the finding.
- Keep identity-rejected APKG baselines high risk even when raw comparison is
  complete; `fail_on(High)` blocks publication and preserves existing files.
- Canonicalize empty browser overrides and zero font sizes in staging inspection
  so APKG roundtrips do not report false browser-template changes.

### Compatibility

- Breaking Rust interface change: replace `artifact.path` with `artifact.path()`.
  Keep the report/artifact handle alive or call `artifact.persist_to(path)`;
  copying a temporary path no longer retains its file. Automatic `report_json`
  now requires `output` or `artifacts_dir`. These changes require a new minor
  crate release, not a patch release in the published 0.1 line.
- Internal-tools callers build versioned inputs with `ProductDocument::build`
  (or the runtime adapter), not `Project::from_product_document`. Documents are
  not editable Projects, and Product v2/v3 interpretation remains unchanged.
- Contract bundle `0.6.2` updates only the three writer fixture package hashes:
  batched SQLite transactions and compaction change header counters and APKG bytes, but
  logical rows, media, staging fingerprints and full inspection semantics remain
  unchanged. Existing package hashes must always be computed from actual bytes.
- Embedded contract bundle advances to `0.6.0`, adding registry coverage and
  executable contract governance to the 0.5.0 update-safety changes. To update
  previously distributed decks, provide `compare_to(previous.apkg)` or a lockfile
  with numeric model IDs.
  A legacy lockfile with null model IDs needs the previous APKG in strict mode;
  rewrite the lockfile after migration. Note GUID derivation is unchanged.
- Strict updates also require note revision evidence. Legacy lockfiles recover it
  from the previous APKG. Use the latest distributed baseline, and persist revised
  lockfiles for future releases; baseline-free exports remain first-release builds.

## [0.1.0] - 2026-08-26

### Added

- Initial production Rust Distribution with typed `Deck` and `Project` APIs.
- Self-contained contract bundle `0.3.0` and default writer runtime.
- Update-safety, media, custom note type, and structured diagnostic capabilities
  through the supported `prelude` facade.
- An explicitly unsupported `internal-tools` feature for the repository's
  unpublished contract tooling and deep conformance tests.

[Unreleased]: https://github.com/morehardy/anki-forge/compare/anki-forge-v0.1.0...HEAD
[0.1.0]: https://github.com/morehardy/anki-forge/releases/tag/anki-forge-v0.1.0
