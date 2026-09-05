# anki_forge

`anki_forge` is a typed Rust library for building Anki decks. The crate ships
its default contract resources, so normal use does not require a source
checkout, a particular working directory, or a separate runtime installation.

```rust,no_run
use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut deck = Deck::new("Spanish");
    deck.basic()
        .note("hola", "hello")
        .stable_id("es:hola")
        .add()?;
    deck.write_apkg("spanish.apkg")?.ensure_success()?;
    Ok(())
}
```

The crate version is `0.1.0`; it embeds contract bundle `0.5.0`. These are
independent compatibility axes. Rust 1.92.0 is the minimum supported compiler
for the 0.1.x line.

## Supported 0.1 interface

The supported consumer interface is intentionally small: import
`anki_forge::prelude::*`, or use the root `Deck`, `Project`, and `Severity`
exports. `facade_api_version()` and `embedded_contract_version()` expose the
two compatibility axes. Normal consumers do not need contract loading,
normalization IR, writer, inspection, or persistence modules.

The `internal-tools` Cargo feature exists only for this repository's
unpublished contract tool and deep conformance tests. Its hidden modules are
not covered by the 0.1 compatibility promise and must not be enabled by
downstream applications.

## Updating distributed decks

For a project with stable project/note identities, build updates using
`BuildOptions::new().output("v2.apkg").compare_to("v1.apkg")`, where `v1.apkg` is
the latest distributed version. Alternatively, use a maintained identity lockfile
with `.update_safe("identity.json").write_identity_lockfile(true)` after the first
`.first_update_safe_build("identity.json")` build. Keep the baseline separate from
the new output and advance it only after verifying the release.

Changed note content advances its baseline modification time; unchanged content
preserves it. This also covers answer-only edits, tags, and content reverts while
keeping same-input/same-baseline builds reproducible. Legacy lockfiles without
revision evidence require a previous APKG for strict migration. Baseline-free
`write_apkg` is a first-release export, not a guarantee that Anki will update
existing notes. Newer local edits remain governed by Anki's import settings.

Report-only builds with missing or unreadable baseline evidence leave the identity
lockfile unchanged, even if writing was requested, and report
`UPDATE.LOCKFILE_WRITE_SKIPPED_UNVERIFIED`. Recover the evidence before retrying;
rejected requested lockfiles are high risk and can be blocked with
`.fail_on(RiskLevel::High)`.

## Errors and concurrency

APKG inspection has finite archive, entry-count, expansion, and zstd-window
budgets. `BuildOptions::inspect_limits(InspectLimits)` applies the same policy to
current and baseline APKGs. Start with `InspectLimits::default()` and explicitly
raise individual fields only for trusted large decks. Resource failures carry
`INSPECT.RESOURCE_LIMIT_EXCEEDED`; report-only baselines remain unavailable, and
current-artifact failures prevent publication. These are decompression budgets,
not a total process memory or CPU sandbox.

High-level `Deck` and `Project` operations return structured errors and build
reports; callers should inspect stable diagnostic codes instead of matching
human-readable messages. File-writing operations are synchronous and may leave
diagnostic evidence in a requested report path when a build fails.

`compare_to(...)` baselines are read-only: output, report, and writable lockfile
paths must not alias them, including through symlinks or hard links. Baselines,
outputs, retained packages, and identity lockfiles must also stay
outside writable staging/media directories, including directory aliases.
Comparison and risk checks use a snapshot captured before building and run before APKG or
lockfile publication. A policy-blocked build preserves existing outputs and
lockfiles and reports diff/risk evidence with no artifact path. Publication is
atomic per file, not transactional across all requested files.
New destinations are rechecked after creation, so a late path-collision error
may leave a valid published APKG but cannot replace it with lockfile/report JSON.
Private candidates follow the artifact workspace's filesystem and are cleaned
up after building. Lockfile publication uses an exclusively reserved temporary
file beside its target, without reusing predictable names that may alias inputs
or outputs.

Values are ordinary owned Rust values and may be moved between threads when
their fields permit it. A single builder or project is not designed for
concurrent mutation; coordinate shared mutation in the calling application.
The embedded read-only contract runtime is initialized once and can be loaded
concurrently.

See the [repository documentation](https://github.com/morehardy/anki-forge)
for custom note types, media, update-safe builds, compatibility policy, and
release operations.

Licensed under MIT.
