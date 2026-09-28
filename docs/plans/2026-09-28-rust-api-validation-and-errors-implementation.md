# Rust API validation and errors — implementation evidence

Implements [the approved design](2026-09-28-rust-api-validation-and-errors-design.md)
against baseline `4d324bb353241cbf8883d90dc46829d5c4cdadec` on `main`.
Implementation and independent Standards/Spec reviews used GPT-6 Astra.

## Delivered behavior

- Project titles are removed from Rust, Node, Python and tool input; model,
  field and template names and deck selection remain. Recipes require v2 and
  reject v1 and any top-level name, including null.
- Typed image/sound nodes validate retained MIME at addition. Sound supports
  audio and video. Constructors, imports, raw HTML, explicit assets and build's
  existing independent sniffing/extension validation retain their boundaries.
- Addition errors carry original identifiers, target paths, UTF-8 ranges and
  typed conflict/usage facts without resource ownership. Dependency collection
  remains atomic. Both SDKs expose typed details and preserve source contexts.
- Node native protocol is 5; Python rejects native metadata with any contract
  other than 2.0.0. The incompatible contract bundle is 2.0.0 with exact inventory,
  migration notes and rebuilt embedded archive. Identity-v1 and build/report/
  comparison DTO versions are unchanged; BUILD.NAME_INVALID is deprecated.

Release preflight on 2026-09-28 found crates.io's published ankiforge maximum
at 0.1.0 and no published package at the queried npm/PyPI endpoints. Existing
unreleased 0.2.0 package versions are retained; bundle versioning is independent.

## Executed checks

Host: macOS aarch64, Rust 1.92.0 (repository MSRV), Node 24.19.0, Python 3.11
for wrapper/type/installed tests; the abi3 wheel build also used CPython 3.14.

- Targeted red/green checks covered missing error context, previously accepted
  incompatible media and the recipe version boundary. After implementation,
  all 10 addition-context/media tests passed. Real MP4/WebM sound references
  and decompressed media payload bytes were verified.
- All 6 media snapshot lifecycle tests passed, including retaining addition
  errors after rejected media and project owners are dropped.
- Workspace suite: 664 passed, 23 ignored by existing test configuration.
  Public external consumers cover new nameable types and removed methods;
  the repository's Rust user-capability gates passed.
- Rust formatting, workspace Clippy with warnings denied, and all-feature
  rustdoc with warnings denied passed.
- Node product suite: 26 passed, 1 environment-dependent skip. Installed ESM/CJS,
  read-only node_modules, four README examples, TypeScript positive/negative
  consumers and same-version old protocol rejection passed.
- Python installed wheel suite: 62 passed, 1 environment-dependent skip.
  Typed context narrowing, removed title negative consumers and same-version
  contract-1.0.0 rejection passed. The source distribution was built using the
  repository backend and rebuilt with locked/offline dependencies outside the
  checkout; its installed wheel, examples and typing checks passed.
- Rust package payload allowlist, reproducible embedded bundle and repository-
  external packaged consumer passed. The consumer inspected actual APKG
  SQLite data and media, not just file existence.
- Documentation synchronization/link/version checks passed. All 16 complete
  documentation example executions and 22 independently inspected APKG outputs
  passed.
- Every command required by `make verify-ci` passed across the initial run and
  the resumed environment-limited step. Python workspace suite: 62 passed,
  1 environment-dependent skip; final contract verify/summary/package passed.
- Contract governance against the fixed baseline passed, including the exact
  2.0.0 change inventory. Independent Standards review: 0 findings. Independent
  Spec review: 0 findings.

The initial verify-ci run reached Node installed-package testing after the Rust
and Node product checks; its localhost registry was blocked by sandbox EPERM.
The unchanged remaining commands were resumed from verify-ci.sh line 81 with
local networking allowed. Separate wheel verification needed network access to
install pytest in its fresh external virtual environment; no project dependency
or test was relaxed to bypass either environment limitation.

## Verification boundary

This run supplies local macOS/MSRV, package, SDK and APKG-content evidence. It
does not claim fresh Windows/Linux CI results or a new Anki desktop/mobile manual
import. Existing client-import evidence was not reclassified as a new run.
