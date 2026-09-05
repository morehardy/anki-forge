# RFC 0006: Executable Contract Governance

## Problem and proposed behavior

A new diagnostic code, a newly indexed semantic document, or a contract edit
without a version bump previously passed the relevant gates. Close each gap
using the authoritative inputs instead of hand-maintained coverage lists.

`verify --source-root <repository>` inventories code-shaped literals from the
Rust syntax tree. It catches code constants, helper arguments, macros and feature
branches and reports missing/removed registry entries with file and line. It
ignores comments, documentation and explicitly test-only items. This is a
conservative static inventory, not arbitrary runtime data-flow analysis:
built-in codes must remain whole literals, while adapters may forward codes.

Semantic verification discovers every manifest entry under the resolved
`semantics/` directory, plus relocated `_semantics` keys, then checks frontmatter
and references. Merely adding a new manifest entry activates verification.

`verify --baseline-manifest <path>` compares independently loaded bundles using
the exact package dependency closure. It requires an exact BLAKE3 inventory in
`assets.bundle_change`, matching old/new versions, a reviewed compatibility
class, and the version increment specified in the version policy. Both additions
and deletions count; transitive fixture files count as well. Manifest key changes
count even when file contents do not change. Only version bookkeeping and the
record itself are excluded from digests to avoid recursive evidence.

`contract_tools changes --manifest <current> --baseline-manifest <previous>`
prints a record template. The author chooses the class and writes the summary;
incompatible changes require migration notes. Exact digests make omitted assets
and records copied before a later edit fail CI. File deletion, key removal or
retargeting, and code removal/retirement force the incompatible class. The gate
does not infer arbitrary schema/semantic compatibility; that remains a review
decision tied to the diff.

## CI and release integration

`scripts/check_contract_governance.sh` extracts the baseline's `contracts/` into
a temporary directory and invokes both repository checks. `make verify-ci` and
`make verify-fast` include it. Local work defaults to the merge-base with
`origin/main`. Pull requests provide their base SHA and main pushes provide the
before SHA, avoiding a self-comparison after merge. Contract release dispatch
requires an explicit previous bundle ref. Missing history fails visibly.

The source registry is also checked in normal workspace integration tests.
Extracted package tests still use plain `verify`, which validates the current
record's structure but needs neither a repository nor a historical bundle.

## Compatibility and acceptance

Publish the 75 previously emitted but unregistered codes without renaming or
changing their behavior. Tighten manifest versions to valid SemVer, add optional
`bundle_change` metadata, and advance the bundle from 0.5.0 to 0.6.0. Rebuild the
embedded bundle and update distribution metadata in the same PR. Legacy bundles
without change records can still be loaded and independently verified.

Regression tests must cover unknown and removed production codes, previously
omitted and newly added semantics, unchanged bundles, missing/stale inventories,
transitive changes, no/insufficient version bump, incompatible changes and
migration notes, asset removal/retargeting, and invalid/decreasing versions.
