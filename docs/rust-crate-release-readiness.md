# Rust Crate Release Readiness

Audit date: 2026-08-27

## Scope and verdict

This audit covers the crates.io `anki_forge` Rust Distribution. It deliberately
does not claim that generated APKG artifacts are production-ready.

The repository now contains a release-candidate and publication flow. The crate
can be packaged as a single self-contained product and consumed outside the
workspace. The CI and review defects recorded on the release PR are closed, and
deep repository interfaces are available only through the unsupported, hidden
`internal-tools` feature. Item-level documentation enforcement and source
provenance review remain explicit publication blockers instead of being treated
as satisfied by packaging success.

Production publication is still blocked until maintainers verify the external
crates.io/GitHub controls and the hosted Tier 1 workflow passes on the release
commit. The initial `cargo publish` is intentionally not performed by this
implementation.

## Implemented controls

| Requirement | Repository state | Result |
| --- | --- | --- |
| One public crate | Authoring and writer cores are private modules inside `anki_forge`; `contract_tools` is `publish = false` | Implemented |
| Self-contained runtime | Deterministic bundle `0.3.0` is embedded and loaded with `RuntimeMode::Installed` | Implemented |
| Crate/bundle mapping | Public version functions, README, changelog, metadata check, and Release Record carry both versions | Implemented |
| Registry identity | Description, MIT license, repository, homepage, docs.rs URL, keywords, categories, README, and changelog are present | Implemented |
| Explicit payload | Cargo `include` allowlist plus required/forbidden path audit | Implemented |
| Hermetic package | `cargo package --locked --offline` verifies committed package contents | Implemented |
| Packaged consumer | Fresh external project uses only extracted package source and builds an APKG through the supported facade offline after dependency prefetch | Implemented |
| Documentation | Crate guide, compiling doctest, errors/concurrency notes, warning-free rustdoc, and a compile boundary that hides repository internals by default are present; broad `missing_docs` exemptions still prevent item-level completeness from being enforced | Partial; publication blocker |
| API surface | Default consumers receive only `prelude`, root `Deck`/`Project`/`Severity`, and version inspection; unpublished tooling explicitly enables hidden `internal-tools` modules | Implemented |
| Source provenance | The package excludes `docs/source`; the compatibility schema implementation still requires maintainer/legal provenance review before an MIT publication | External review required |
| Dependency policy | `cargo-deny` blocks advisories, unapproved licenses, wildcard registry dependencies, unknown sources, and unreviewed duplicate-version splits | Implemented |
| Security remediation | Vulnerable locked versions of `anyhow`, `url`/`idna`, `rand`, and `tar` were upgraded without advisory exceptions | Implemented |
| API compatibility | `cargo-semver-checks` is required after the initial 0.1.0 release establishes a compatible crates.io baseline | Implemented |
| Tier 1/MSRV/stable | CI, rehearsal, and tag workflows cover four Tier 1 runners with Rust 1.92.0 and current stable; publish waits for the tag matrix | Implemented; hosted result required |
| Release authority | Only protected `anki-forge-vX.Y.Z` tag workflow can publish; tag/manifest/changelog/bundle mapping are validated | Implemented |
| Trusted publication | OIDC crates.io authentication, no long-lived registry token, and `crates-io` environment approval | Implemented; external configuration required |
| Release evidence | Evidence is created only after Tier 1 passes; the candidate `.crate` checksum is matched against the exact registry download before the GitHub evidence release | Implemented |
| Rehearsal and recovery | Manual non-publishing full rehearsal plus fix-forward, yank, advisory, and evidence-preservation runbook | Implemented |

## Verified locally

The following checks passed during implementation:

- workspace/all-target compilation after consolidating the crate topology;
- deterministic contract package reproduction;
- package payload audit and release metadata/tag validation;
- warning-free rustdoc and compiling public quick-start doctest;
- extracted `.crate` verification and fresh packaged-consumer execution;
- current `cargo-deny` advisory, license, bans, and source checks;
- targeted runtime, authoring, writer, and package regression tests.

The full workspace suite is run once at implementation completion. Linux,
Windows, and both macOS architectures cannot all be proven by one local machine;
their required evidence comes from `.github/workflows/rust-crate-ci.yml`.

## Remaining publication blockers

The remaining items require source review, external authority, or hosted state:

1. Remove the broad default-surface `missing_docs` exemptions and document every supported public item.
2. Confirm that the packaged compatibility schema source has provenance compatible with the intended MIT distribution; do not rely only on excluding the upstream mirror from the Cargo payload.
3. Confirm that the crates.io `anki_forge` name is available/owned by the intended maintainers.
4. Configure the crates.io Trusted Publisher for this repository and workflow.
5. Configure the protected GitHub `crates-io` environment with required reviewers.
6. Protect release tags and require the Rust crate CI checks on the exact release commit.
7. Observe a green Tier 1/MSRV/stable matrix and review candidate evidence.
8. Obtain explicit human approval before creating the first authoritative tag.

Until those are satisfied, the correct action is rehearsal (`cargo publish
--dry-run`), not publication.

## Blocker handling policy

- Code, packaging, documentation, security, or compatibility failures are fixed
  in the release PR; they are not waived by publishing manually.
- Security exceptions are allowed only when unavoidable and must include owner,
  rationale, and expiry in the Release Record. Current duplicate-dependency
  policy exceptions are recorded in `docs/dependency-policy-exceptions.json`.
- External configuration failures stop before the OIDC publish step.
- A defect discovered after publication is fixed in a higher immutable version;
  severe versions may be yanked, and release tags are never moved or reused.

Operational details are in `docs/rust-release-runbook.md`.
