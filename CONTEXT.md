# Rust Distribution

This context defines the production distribution boundary for the Rust API.

## Language

**Rust Distribution**:
The versioned `anki_forge` library crate published through crates.io for downstream Rust projects to depend on. Its production-readiness scope excludes generated APKG files and internal verification tools.
_Avoid_: Rust production package, generated package

**Authoritative Release Channel**:
crates.io is the canonical source for production releases of the Rust Distribution.
_Avoid_: GitHub release, source checkout

**Public Crate**:
`anki_forge` is the sole crate published as the Rust Distribution. Authoring and writing cores are internal boundaries, not independently versioned products.
_Avoid_: Public core crates, multi-crate release

**Self-Contained Distribution**:
The Rust Distribution carries the default contract resources required by its public behavior. Normal use does not depend on a source checkout, the process working directory, or separately installed contract files.
_Avoid_: Repository-relative runtime, implicit contract discovery

**Crate Version**:
The SemVer version of the Rust Distribution, governing compatibility of its public Rust API and behavior.
_Avoid_: Bundle version, synchronized version

**Bundle Version**:
The compatibility version of the contract resources embedded in a Rust Distribution release. It evolves independently from the Crate Version, and each release identifies the version it carries.
_Avoid_: Crate version, release version

**Initial Public Release**:
The production-quality `anki_forge` 0.1.0 release on crates.io. It satisfies release and runtime quality gates while retaining pre-1.0 freedom to revise the public API through minor-version changes.
_Avoid_: Stable 1.0, beta-quality package

**Tier 1 Platform**:
A platform on which every Rust Distribution release must compile and pass its required test suite: Linux x86_64, Windows x86_64, macOS x86_64, and macOS ARM64.
_Avoid_: Best-effort target, untested target

**Best-Effort Platform**:
A Rust target outside the Tier 1 set that may work but carries no release-blocking compatibility promise. WebAssembly is best-effort rather than Tier 1.
_Avoid_: Supported platform

**Supported Rust Baseline**:
Rust 1.92.0 is the minimum compiler version promised for the 0.1.x release line. Releases are also verified against the current stable compiler, and any baseline increase is announced through a versioned release.
_Avoid_: stable, latest Rust

**Authoritative Release Event**:
A protected `anki-forge-vX.Y.Z` Git tag whose version matches the crate manifest and triggers the trusted publishing workflow. Local publication and long-lived registry credentials are outside the release process.
_Avoid_: Manual publish, workflow dispatch, local release

**Packaged Consumer Test**:
A release-blocking test in a fresh project outside the repository that depends only on the packaged Rust Distribution and exercises its public API without workspace files, internal crates, or external contract resources.
_Avoid_: Workspace test, source-tree example

**Distribution License**:
MIT is the license for project-owned source shipped in the Rust Distribution. Mirrored or other third-party source is not part of that licensing grant and retains its own license.
_Avoid_: MIT OR Apache-2.0, repository-wide relicensing

**Dependency Policy**:
Production dependencies resolve from crates.io and pass release-blocking vulnerability, license, duplication, and source checks. Git and local path dependencies are excluded from the published dependency graph; any security exception identifies an owner and expiry date.
_Avoid_: Best-effort audit, permanent advisory exception

**Release Gate**:
The mandatory automated checks that must pass before publication: formatting, warning-free linting and documentation, workspace tests and doctests, locked packaging, dependency and platform policies, packaged-consumer behavior, and public API compatibility with the latest compatible release.
_Avoid_: Manual checklist, advisory check

**Compatible Release**:
A patch release within a pre-1.0 minor line that preserves the previously published public Rust API. Breaking API changes begin a new 0.x minor line.
_Avoid_: Any 0.x release, undocumented breakage

**Package Payload**:
The explicit allowlist of source, public examples, required tests, embedded contracts, and release documentation carried by the `.crate` archive. Internal tools, large fixtures, temporary files, and upstream source mirrors are excluded.
_Avoid_: Repository snapshot, implicit package contents

**Public API Documentation**:
The docs.rs documentation and compiling quick-start examples that define supported use of the Rust Distribution, including public errors, thread-safety expectations, Rust baseline, and compatibility policy.
_Avoid_: Source comments, repository-only guide

**Release Record**:
The durable evidence for a Rust Distribution release, linking its tag and commit to crate and bundle versions, supported-toolchain and platform results, package checksum, dependency SBOM, changelog, and any time-bounded security exception.
_Avoid_: CI console output, mutable release notes

**Release Recovery**:
The response to a defective immutable crates.io version: publish a higher corrective version, yank a severely defective version, and issue a security advisory when applicable. Published versions and tags are never overwritten or reused.
_Avoid_: Rebuilt version, replaced tag, silent withdrawal

**Hermetic Package Build**:
A build of the packaged Rust Distribution that succeeds with registry dependencies cached and network access disabled, using only committed package contents. It does not fetch resources or depend on Git state, workspace-external files, or uncommitted generated output.
_Avoid_: Repository build, online build

**Publication Approval**:
The explicit approval by a designated maintainer in a protected CI environment after every Release Gate passes and before crates.io publication begins. Creating the release tag starts a candidate release but does not itself authorize publication.
_Avoid_: Tag-only publication, unattended publish
