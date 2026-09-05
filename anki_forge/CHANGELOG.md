# Changelog

All notable changes to the Rust Distribution are documented here. The crate
follows Semantic Versioning; before 1.0, breaking public API changes require a
new minor release.

## [Unreleased]

### Fixed

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

### Compatibility

- Embedded contract bundle advances to `0.5.0`. To update previously distributed
  decks, provide `compare_to(previous.apkg)` or a lockfile with numeric model IDs.
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
