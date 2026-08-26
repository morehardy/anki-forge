# Rust Crate Release Readiness

Audit date: 2026-08-26

## Scope and verdict

This audit covers the crates.io `anki_forge` Rust Distribution. It deliberately
does not claim that generated APKG artifacts are production-ready.

The repository now contains a complete release-candidate and publication flow.
The crate can be packaged as a single self-contained product and consumed
outside the workspace. Repository-level P0 blockers from the original audit are
closed.

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
| Packaged consumer | Fresh external project uses only extracted package source and runs the embedded writer stack offline after dependency prefetch | Implemented |
| Documentation | Crate guide, compiling doctest, errors/concurrency notes, missing-docs deny lint, and warning-free rustdoc gate | Implemented |
| Dependency policy | `cargo-deny` blocks advisories, unapproved licenses, wildcard registry dependencies, and unknown sources; duplicate versions are visible warnings | Implemented |
| Security remediation | Vulnerable locked versions of `anyhow`, `url`/`idna`, `rand`, and `tar` were upgraded without advisory exceptions | Implemented |
| API compatibility | `cargo-semver-checks` is required by CI and release workflows | Implemented |
| Tier 1/MSRV/stable | Packaged consumer matrix covers four Tier 1 runners with Rust 1.92.0 and current stable | Implemented; hosted result required |
| Release authority | Only protected `anki-forge-vX.Y.Z` tag workflow can publish; tag/manifest/changelog/bundle mapping are validated | Implemented |
| Trusted publication | OIDC crates.io authentication, no long-lived registry token, and `crates-io` environment approval | Implemented; external configuration required |
| Release evidence | `.crate` checksum, CycloneDX SBOM, changelog, tag/commit/version mapping, and JSON Release Record | Implemented |
| Recovery | Fix-forward, yank, advisory, and evidence-preservation runbook | Implemented |

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

These items require external authority or hosted state and cannot be completed
by source changes alone:

1. Confirm that the crates.io `anki_forge` name is available/owned by the intended maintainers.
2. Configure the crates.io Trusted Publisher for this repository and workflow.
3. Configure the protected GitHub `crates-io` environment with required reviewers.
4. Protect release tags and require the Rust crate CI checks on the exact release commit.
5. Observe a green Tier 1/MSRV/stable matrix and review candidate evidence.
6. Obtain explicit human approval before creating the first authoritative tag.

Until those are satisfied, the correct action is rehearsal (`cargo publish
--dry-run`), not publication.

## Blocker handling policy

- Code, packaging, documentation, security, or compatibility failures are fixed
  in the release PR; they are not waived by publishing manually.
- Security exceptions are allowed only when unavoidable and must include owner,
  rationale, and expiry in the Release Record. There are no such exceptions in
  the current candidate.
- External configuration failures stop before the OIDC publish step.
- A defect discovered after publication is fixed in a higher immutable version;
  severe versions may be yanked, and release tags are never moved or reused.

Operational details are in `docs/rust-release-runbook.md`.
