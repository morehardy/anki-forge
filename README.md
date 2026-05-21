# anki-forge

`anki-forge` is a Rust-first toolkit for building Anki `.apkg` artifacts, with
advanced contract tooling and Node/Python bindings for lower-level workflows.

Most users should start with the typed Rust API:

- `Deck` for the shortest path from notes to an APKG
- `Project` for long-term decks that need stable IDs, custom note types, media,
  validation, and build reports

## 1. Requirements

- Rust `1.92.0` (see `rust-toolchain.toml`)
- `cargo`
- `jq` for advanced contract-tool examples
- Optional: Node.js `18+` for Node binding examples/tests
- Optional: Python `3.11+` for Python binding examples/tests
- Optional: `protoc` + local `docs/source/anki` for the roundtrip oracle only

Suggested one-time setup from the repository root:

```bash
rustup toolchain install 1.92.0
rustup override set 1.92.0
```

## 2. Quick Start: Deck First

```rust
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

Run the same flow locally:

```bash
cargo run -q -p anki_forge --example target_api_basic
```

This writes `spanish.apkg` in the current directory.

## 3. Project For Long-Term Decks

```rust
use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_note(Note::basic("食べる", "to eat").stable_id("jp:taberu"))?;
    project.write_apkg("jp-core.apkg")?.ensure_success()?;
    Ok(())
}
```

`BuildReport` includes the artifact path, note/card/media counts, diagnostics,
warning count, inspect summary, and duration. `inspect.observation_status` is
writer-layer reporting metadata passed through from the inspection step.

Custom note types, stable field/template keys, and project media are shown in:

```bash
cargo run -q -p anki_forge --example target_api_custom_notetype
cargo run -q -p anki_forge --example target_api_media
```

`Note::cloze(...)` intentionally stores the cloze `Text` field as explicit HTML
so Anki receives raw `{{cN::...}}` markers. Do not assume cloze text is escaped
like `Note::basic(...)` text.

### 3.1 Media Troubleshooting

Media export names must be helper-safe bare filenames such as `taberu.mp3`.
Avoid path components, absolute paths, URL escapes, and unsafe characters.
Register files with `project.media_mut().add_file(...).export_as("taberu.mp3")`;
inline examples can use `project.media_mut().add_bytes(...).export_as(...)`.

Common media diagnostics:

- Filename collision: the same export filename is bound to different bytes.
  Choose a unique `export_as(...)` name and update local note, template, or CSS
  references to match.
- Missing media reference: Product content refers to a local filename that is
  not registered. Register it or change the local filename in the HTML/CSS.
- CSS missing reference: CSS scanning is conservative. A local
  `url("icon.svg")` should be registered, changed to an external URL, or removed
  if the CSS rule is unused.
- CSS import reference: a local import such as `url("theme.css")` must be
  registered as packaged media, changed to an external URL, or removed if unused.
- Unused media binding: a registered file is not referenced by any note,
  template, or CSS. Remove the registration or add the intended local reference;
  this is a warning under the strict default.
- Unsafe media reference: packaged media references must be bare local
  filenames. Remove path components, absolute paths, escapes, or unsafe
  characters.
- MIME mismatch: the export filename or declared MIME does not match the
  observed source bytes. Change the export filename/declared MIME, or replace
  the source file.

`anki-forge` does not automatically rewrite filenames, HTML, or CSS because
those edits can change deck behavior and hide the authoring intent. Keep the
registered `export_as(...)` filename and local references in sync yourself.
`BuildReport::pretty_report()` is a human-facing summary; structured
machine-readable report export is not currently exported.

## 4. Advanced: Contract Tools And Runtime

The lower-level contract flow is:

`Authoring IR -> normalize -> build -> inspect -> diff`

### 4.1 PR Verification

Before a PR, sync the base branch and run the same full verification entry point
used by GitHub Actions:

```bash
git fetch origin main
make verify-ci
```

For faster local development:

```bash
make verify-fast
```

`make verify-ci` mirrors `.github/workflows/contract-ci.yml`; a PR is ready only
after both local `make verify-ci` and the remote `contract-ci / verify` pass.

### 4.2 Contract Validation And Packaging

```bash
cargo run -q -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -q -p contract_tools -- summary --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -q -p contract_tools -- package --manifest "$(pwd)/contracts/manifest.yaml" --out-dir "$(pwd)/dist"
```

- `verify`: validates contracts and executable gates
- `summary`: prints bundle version and component summaries
- `package`: writes versioned artifacts into `dist/`

### 4.3 Normalize -> Build -> Inspect -> Diff

```bash
mkdir -p tmp/readme-basic

cargo run -q -p contract_tools -- normalize \
  --manifest "$(pwd)/contracts/manifest.yaml" \
  --input "$(pwd)/contracts/fixtures/phase3/inputs/basic-authoring-ir.json" \
  --output contract-json > "$(pwd)/tmp/readme-basic/normalize.result.json"

jq -e '.normalized_ir' "$(pwd)/tmp/readme-basic/normalize.result.json" > "$(pwd)/tmp/readme-basic/normalized-ir.json"

cargo run -q -p contract_tools -- build \
  --manifest "$(pwd)/contracts/manifest.yaml" \
  --input "$(pwd)/tmp/readme-basic/normalized-ir.json" \
  --writer-policy default \
  --build-context default \
  --artifacts-dir "$(pwd)/tmp/readme-basic/artifacts" \
  --output contract-json > "$(pwd)/tmp/readme-basic/build.result.json"

cargo run -q -p contract_tools -- inspect \
  --staging "$(pwd)/tmp/readme-basic/artifacts/staging/manifest.json" \
  --output contract-json > "$(pwd)/tmp/readme-basic/staging.inspect.json"

cargo run -q -p contract_tools -- inspect \
  --apkg "$(pwd)/tmp/readme-basic/artifacts/package.apkg" \
  --output contract-json > "$(pwd)/tmp/readme-basic/apkg.inspect.json"

cargo run -q -p contract_tools -- diff \
  --left "$(pwd)/tmp/readme-basic/staging.inspect.json" \
  --right "$(pwd)/tmp/readme-basic/apkg.inspect.json" \
  --output contract-json > "$(pwd)/tmp/readme-basic/diff.result.json"
```

Main outputs:

- `tmp/readme-basic/artifacts/package.apkg`
- `tmp/readme-basic/staging.inspect.json`
- `tmp/readme-basic/apkg.inspect.json`
- `tmp/readme-basic/diff.result.json`

### 4.4 Rust Examples

```bash
cargo run -q -p anki_forge --example target_api_basic
cargo run -q -p anki_forge --example target_api_custom_notetype
cargo run -q -p anki_forge --example target_api_media
cargo run -q -p anki_forge --example deck_basic_flow
cargo run -q -p anki_forge --example product_basic_flow
cargo run -q -p anki_forge --example minimal_flow
```

- `target_api_basic`: shortest `Deck` API path; writes `spanish.apkg`
- `target_api_custom_notetype`: `Project` with custom note type; writes `jp-core.apkg`
- `target_api_media`: `Project` media helpers, template/CSS references, and
  pretty media report; writes `spanish-media.apkg`
- `deck_basic_flow`: broader Rust Deck API scenario
- `product_basic_flow`: lower-level product authoring example
- `minimal_flow`: file-driven runtime example

### 4.5 Stable Note Identity

New Basic, Cloze, and Image Occlusion notes use AFID (`afid:v1:*`) as stable
note IDs by default instead of legacy `generated:*` IDs. AFID comes from the
normalized identity payload: Basic defaults to the front field, Cloze uses the
cloze structure and text skeleton, and Image Occlusion uses image content,
dimensions, mode, and sorted mask geometry.

Explicit `stable_id` still wins and is saved as an explicit identity snapshot.
If callers explicitly pass a `generated:*` prefix, it is kept as an ordinary
explicit stable ID. The `afid:v1:*` namespace is reserved and cannot be passed
as an explicit stable ID.

Basic notes can choose identity fields through the typed API:

```rust
use anki_forge::{BasicIdentityField, BasicIdentityOverride, BasicIdentitySelection, BasicNote, Deck};

let mut deck = Deck::builder("Spanish")
    .basic_identity(BasicIdentitySelection::new([BasicIdentityField::Back])?)
    .build();
deck.add(BasicNote::new("hola", "hello"))?;

let override_cfg = BasicIdentityOverride::new(
    [BasicIdentityField::Front, BasicIdentityField::Back],
    "sense-disambiguation",
)?;
deck.basic()
    .note("banco", "bank / bench")
    .identity_override(override_cfg)
    .add()?;
```

`validate_report()` preserves legacy stable ID diagnostics for blank IDs,
missing/generated legacy IDs, unknown media, empty Image Occlusion masks, and
duplicate IDs. It also returns a `NoteLevelIdentityOverrideUsed` warning when a
note uses note-level identity override. AFID duplicate payloads, hash
collisions, and stable ID duplicates are blocking add-time or load-time rebuild
errors.

Serialization preserves resolved identity snapshots (`stable_id`, `recipe_id`,
`provenance`, `canonical_payload`, and `used_override`). Deserialization rebuilds
runtime indexes and validates that snapshots still match note IDs, payload
hashes, payload duplicates, and collisions.

### 4.6 Node Bindings

```bash
npm --prefix bindings/node install
npm --prefix bindings/node run example:minimal
npm --prefix bindings/node test
```

### 4.7 Python Bindings

```bash
PYTHONPATH=bindings/python/src python3.11 bindings/python/examples/minimal_flow.py
PYTHONPATH=bindings/python/src python3.11 -m unittest discover -s bindings/python/tests -p "test_*.py"
```

The target Product API shape sketches are documented in
`bindings/python/examples/target_api_custom.py` and
`bindings/python/examples/target_api_media.py`.

## 5. Manual Anki Desktop Scenarios

Generate every manual verification APKG:

```bash
./scripts/run_manual_desktop_scenarios.sh
```

Generate one scenario:

```bash
./scripts/run_manual_desktop_scenarios.sh S05_basic_audio
```

Output paths:

- `tmp/manual-desktop-v1/<scenario>/package.apkg`
- `tmp/manual-desktop-v1/<scenario>/apkg.inspect.json`

## 6. Roundtrip Oracle

Use this optional flow only when validating roundtrip behavior against a local
Anki upstream checkout.

Requirements:

- `docs/source/anki/rslib/Cargo.toml` exists
- `protoc` is available on `PATH`

Run:

```bash
./scripts/run_roundtrip_oracle.sh
```

## 7. FAQ

- Error: `failed to discover contracts/manifest.yaml from workspace path`
  - Confirm the current directory is inside this repository, or pass an explicit
    `cwd` from bindings.
- Error: `missing vendored upstream Anki crate ... docs/source/anki/rslib`
  - The roundtrip oracle is missing local Anki source. Normal flows are not
    affected.
- Error: `protoc is required on PATH`
  - Install `protoc` and retry. This is required only for the roundtrip oracle.

## 8. Related Docs

- `bindings/node/README.md`
- `bindings/python/README.md`
- `contracts/fixtures/phase3/manual-desktop-v1/README.md`
