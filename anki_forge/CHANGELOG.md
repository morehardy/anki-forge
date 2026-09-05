# Changelog

All notable changes to the Rust Distribution are documented here. The crate
follows Semantic Versioning; before 1.0, breaking public API changes require a
new minor release.

## [Unreleased]

### Fixed

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
