# Rust Distribution Release Runbook

This runbook applies only to the crates.io `anki_forge` Rust Distribution. It
does not certify generated APKG semantics or publish the contract bundle as a
separate product.

## Repository gates

A release candidate is blocked until all of the following are green:

1. `cargo fmt`, warning-free Clippy, workspace tests, doctests, and warning-free rustdoc.
2. The committed embedded contract archive byte-for-byte matches a fresh deterministic build.
3. `cargo deny` accepts advisories, licenses, dependency sources, and the reviewed duplicate graph.
4. `cargo-semver-checks` accepts compatibility with the latest compatible crates.io release.
5. `cargo package --locked` forms and verifies the package, the payload audit passes, and a fresh external consumer runs using only the extracted package.
6. The packaged consumer passes on Linux x86_64, Windows x86_64, macOS x86_64, and macOS ARM64 with Rust 1.92.0 and current stable.

The authoritative event is a protected tag named `anki-forge-vX.Y.Z`. The tag,
`anki_forge/Cargo.toml`, `anki_forge/CHANGELOG.md`, and the crate/bundle mapping
must agree. Never reuse or move a release tag.

## External prerequisites

Before the first release, a repository administrator must verify the following
external state:

- the crates.io `anki_forge` name and ownership;
- a crates.io Trusted Publisher scoped to this repository and release workflow;
- a GitHub environment named `crates-io` with designated-maintainer approval;
- branch/tag rules that require the Rust crate CI workflow.

These controls cannot be proven by repository files alone. Their absence is a
publication blocker even when local tests pass.

## Publication

1. Merge the release PR with the version, changelog, and bundle mapping.
2. Confirm the main-branch CI run is green on the exact commit.
3. Create the protected `anki-forge-vX.Y.Z` tag on that commit.
4. Review candidate package, checksum, CycloneDX SBOM, and release record.
5. Approve the protected `crates-io` environment. The isolated publish job
   obtains a short-lived OIDC token and runs `cargo publish`; no long-lived
   registry token is stored.
6. A separately retryable, read-only post-publication job downloads the exact
   crates.io archive, matches it to the candidate checksum, exercises it in a
   fresh consumer, and confirms docs.rs builds.
7. Only after that verification succeeds, a dedicated write-scoped job creates
   or completes the evidence release on GitHub. A retry may add a missing
   asset, but it refuses to replace an existing asset whose bytes differ.

Actual first publication remains an explicit release operation; development
and rehearsal must use `cargo publish --dry-run` and must not create the tag.

## Recovery

crates.io versions are immutable. Do not overwrite a package or reuse a tag.

- For a normal defect, fix forward and publish a higher patch version.
- For a severe unusable or unsafe release, a crate owner may run
  `cargo yank --version X.Y.Z anki_forge`, record the reason publicly, then
  publish a corrective version. Yanking is not deletion and existing lockfiles
  can continue to resolve the version.
- For a security defect, coordinate disclosure, publish a RustSec-compatible
  advisory when appropriate, yank affected versions when necessary, and link
  the advisory from the corrective release record.
- Preserve the original checksum, SBOM, logs, tag, and commit for investigation.

Every exception to dependency security policy must identify an owner, a reason,
and an expiry date in the release record; permanent silent exceptions are not
allowed.
