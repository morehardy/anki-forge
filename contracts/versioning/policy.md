# Bundle Versioning Policy

The bundle version is the only public compatibility axis for Anki Forge contracts.
Component versions are internal governance metadata only.

Internal component versions may track schema, fixtures, service envelope, and error registry evolution, but they do not define public compatibility claims on their own.

## Executable change governance

Bundle versions must be valid SemVer. Repository CI compares the current bundle
with the PR base commit, or the previous commit on a push. Standalone `verify`
continues to check an extracted bundle without requiring Git or source files.
`verify --baseline-manifest <path>` additionally checks the actual version change.

The comparison uses the same asset closure as the packager, including transitive
fixture dependencies. Changed files are identified by before/after BLAKE3
digests. The manifest is compared as a canonical projection of its asset map and
compatibility axis; version/component bookkeeping and `bundle_change` are
excluded to avoid a recursive digest. Other assets are compared byte for byte.

If published assets change, the current manifest must reference a `bundle_change`
YAML record with `baseline_version`, `bundle_version`, `compatibility_class`, a
non-empty `summary`, and the exact `changes` list (`path`, `before`, `after`).
Digests have the form `blake3:<64 lowercase hex digits>`; a missing side is null.
Generate the inventory with `contract_tools changes --manifest <current>
--baseline-manifest <baseline>`, review the classification and summary, and
register the completed file under `assets.bundle_change`. A stale baseline,
omitted asset, or subsequent content change invalidates the record.

| Compatibility class | Minimum version increase |
| --- | --- |
| `additive_compatible` | minor |
| `behavior_tightening_compatible` | minor |
| `behavior_changing_incompatible` | major, with non-empty `migration_notes` |
| `fixture_only_non_semantic` | patch; only files under `fixtures/` may change |
| `documentation_only_normative_clarification` | patch; only Markdown under `semantics/` or `versioning/` may change |

The major/minor rule applies to contract bundles even before 1.0, matching the
evolution fixtures. It is independent of the Rust crate's pre-1.0 SemVer policy.
Version decreases are rejected even when assets are unchanged. Prerelease or
build-metadata changes alone do not satisfy a required major/minor/patch bump.
Removing published files, removing/retargeting manifest keys, or removing or
retiring diagnostic codes requires the incompatible class. Arbitrary behavioral
compatibility cannot be inferred from file diffs: reviewers remain responsible
for the declared class and migration notes. CI verifies that the declaration
covers the actual bytes and enforces the corresponding version increase.

The historical `evolution` fixtures validate the policy vocabulary. They do not
replace the baseline comparison or serve as evidence for a new change.

## Production diagnostic inventory

`verify --source-root <repository>` checks code-shaped string literals in
`anki_forge/src` and `contract_tools/src` against the current registry. Every
built-in diagnostic or risk code must be an entire literal (including constants,
helper arguments, match arms and macro arguments), use `UPPERCASE.CODE` or legacy
`AF` plus digits, and be active or deprecated. Do not synthesize built-in codes
with formatting or concatenation; adapters may forward existing codes.
The Rust syntax scanner ignores comments, documentation and explicitly test-only
items, and checks all production feature/platform branches. Registry coverage is
also exercised by a repository integration test.

Semantic assets are discovered from the manifest: all entries resolving under
`semantics/`, plus keys ending in `_semantics` even if relocated, must have
frontmatter with non-empty, manifest-registered `asset_refs`.
