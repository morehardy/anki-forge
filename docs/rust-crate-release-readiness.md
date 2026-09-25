# Rust Crate Release Readiness

Audit updated: 2026-09-24 (clean-slate local implementation and acceptance complete)

## Scope and verdict

This audit covers the crates.io `ankiforge` Rust Distribution. It deliberately
does not claim that generated APKG artifacts are production-ready.

The repository now contains a release-candidate and publication flow. The crate
can be packaged as a single self-contained product and consumed outside the
workspace. The CI and review defects recorded on the release PR are closed, and
deep repository interfaces are available only through the unsupported, hidden
`internal-tools` feature. The public interface now enforces item-level documentation on actual definitions.
Source provenance review and hosted platform evidence remain publication conditions.
Implementation status is tracked in the [current plan](plans/2026-09-23-rust-api-clean-slate-design.md#8-实施记录);
current local results are recorded in the [acceptance evidence](plans/evidence/rust-api-clean-slate-2026-09-24/verification.md).

Production publication is still blocked until maintainers verify the external
crates.io/GitHub controls and the hosted Tier 1 workflow passes on the release
commit. The initial `cargo publish` is intentionally not performed by this
implementation.

## Implemented controls

| Requirement | Repository state | Result |
| --- | --- | --- |
| One public crate | Authoring and writer cores are private modules inside `ankiforge`; `contract_tools` is `publish = false` | Implemented |
| Self-contained runtime | Deterministic bundle `1.0.0` is embedded and loaded by the private runtime | Implemented |
| Crate/bundle mapping | Public version functions, README, changelog, metadata check, and Release Record carry both versions | Implemented |
| Registry identity | Description, MIT license, repository, official homepage and Rust guide URLs, keywords, categories, README, and changelog are present | Implemented |
| Explicit payload | Cargo `include` allowlist plus required/forbidden path audit | Implemented |
| Hermetic package | `cargo package --locked --offline` verifies committed package contents | Implemented |
| Packaged consumer | Fresh external project uses only extracted package source and builds an APKG through the supported facade offline after dependency prefetch | Implemented |
| Documentation | Public definitions and impls enforce `missing_docs`; current guides cover owned values, errors, reports, and updates | Passed locally for the current candidate |
| API surface | Root common types and six public domains; internals private; hidden curated `tools` requires `internal-tools` | Implemented |
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

The current production implementation (`399bc19`) passed:

- complete Rust quality and local `verify-ci` gates, including all-feature
  workspace tests, warning-free Clippy/rustdoc, and doctests;
- independent default public contracts (21), capability scenarios (23), and
  positive/negative public-boundary probes;
- deterministic embedded bundle reproduction, exact payload and release metadata;
- offline packaged consumers on Rust 1.92.0 and installed 1.98.1;
- rebuilt Node (17) and Python (47) native tests, independent installation and
  typing checks, including a final sdist-to-wheel build outside the repository;
- documentation execution (16 complete programs, 22 independently inspected
  APKG outputs) and website checks;
- fresh RustSec/cargo-deny advisory, license, bans, and source checks.

A final verification-harness race was fixed and independently reviewed; concurrent
consumer and documentation runs passed. These changes do not alter production
sources. Exact commands, log digests, review findings and limits are in the
[acceptance record](plans/evidence/rust-api-clean-slate-2026-09-24/verification.md).

This is macOS ARM64 evidence. Other Tier 1 platforms and the current stable channel
require `.github/workflows/rust-crate-ci.yml`; installed Rust 1.98.1 is not a claim
that the stable channel was refreshed.

## Remaining publication blockers

The remaining items require source review, external authority, or hosted state:

1. Confirm that the packaged compatibility schema source has provenance compatible with the intended MIT distribution; do not rely only on excluding the upstream mirror from the Cargo payload.
2. Confirm that the crates.io `ankiforge` name is available/owned by the intended maintainers.
3. Configure the crates.io Trusted Publisher for this repository and workflow.
4. Configure the protected GitHub `crates-io` environment with required reviewers.
5. Protect release tags and require the Rust crate CI checks on the exact release commit.
6. Observe a green Tier 1/MSRV/stable matrix and review candidate evidence.
7. Obtain explicit human approval before creating the first authoritative tag.

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
