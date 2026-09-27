# Development and contract tools

This guide is for working on anki-forge itself. For deck authoring, start with
the [Rust quick start](../README.md#quick-start), [Node SDK](../bindings/node/README.md),
or [Python package](../bindings/python/README.md).

Run the following commands from the repository root.

## Requirements

- Rust 1.92.0 and Cargo; the checkout pins this toolchain in
  [rust-toolchain.toml](../rust-toolchain.toml).
- Git and Make for the shared verification entry points.
- Node.js 22.13+ and npm for the Node SDK and full verification.
- Python 3.11+ with `pytest` for Python tests and full verification.
- `jq` for the JSON contract walkthrough below.
- A local `docs/source/anki` checkout and `protoc` for the optional roundtrip
  oracle only.

Install the repository toolchain if needed:

```sh
rustup toolchain install 1.92.0
```

## Verification

Before marking a PR ready, fetch the comparison branch and run the shared
verification script:

```sh
git fetch origin main
make verify-ci
```

For the shorter local Rust checks:

```sh
make verify-fast
```

Both entry points require `origin/main`. `verify-fast` checks Rust formatting,
contract governance, Clippy, workspace tests, and whitespace. `verify-ci` also
runs capability checks, Node SDK tests, Python tests, and contract packaging.
Use a Python environment with `pytest` installed and its `python3` on `PATH`.
See [the script](../scripts/verify-ci.sh) and
[contract CI](../.github/workflows/contract-ci.yml) for the exact commands;
a PR is ready after local verification and the remote `contract-ci / verify`
check pass.

## Rust examples

The supported API examples run with default features:

```sh
cargo run -q -p ankiforge --example target_api_basic
cargo run -q -p ankiforge --example target_api_custom_notetype
cargo run -q -p ankiforge --example target_api_media
```

They write `spanish.apkg`, `jp-core.apkg`, and `spanish-media.apkg`, respectively.
Repository-only examples require the `internal-tools` feature:

```sh
cargo run -q -p ankiforge --features internal-tools --example minimal_flow
```

`product_basic_flow` also uses the default public API.

`internal-tools` exposes curated unsupported tools operations for contract and
conformance work. Downstream applications should use the default public API.

## Contract validation and packaging

The lower-level flow is `Authoring IR → normalize → build → inspect → diff`.
`contract_tools` is an unpublished repository tool.


```bash
cargo run -q -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -q -p contract_tools -- summary --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -q -p contract_tools -- package --manifest "$(pwd)/contracts/manifest.yaml" --out-dir "$(pwd)/dist"
```

- `verify`: validates contracts and executable gates
- `summary`: prints bundle version and component summaries
- `package`: writes versioned artifacts into `dist/`

## Normalize → build → inspect → diff

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

## Node and Python development

See the [Node SDK development commands](../bindings/node/README.md#develop-and-verify)
for building the native addon and testing installed packages.

The [Python setup guide](../bindings/python/README.md#from-a-source-checkout)
builds the native extension with Maturin. Use a CPython 3.11/3.12 virtual
environment; PYTHONPATH alone does not build the extension.

```sh
maturin develop --manifest-path bindings/python/native/Cargo.toml --locked
cargo build -p anki_forge_python_native --example python_parity --locked
python -m pytest bindings/python/tests -q
python -m mypy --config-file bindings/python/pyproject.toml bindings/python/src/anki_forge
```

The native SDK calls the same Rust public API. See [Python coverage](../bindings/python/COVERAGE.md) for installed-wheel,
source-distribution and platform verification. User guide editing and executable
snippets are documented in [documentation maintenance](documentation.md).

## Manual Anki Desktop validation

Generate all manual verification APKGs, or select a scenario:

```sh
./scripts/run_manual_desktop_scenarios.sh
./scripts/run_manual_desktop_scenarios.sh S05_basic_audio
```

Outputs include `tmp/manual-desktop-v1/<scenario>/package.apkg` and
`tmp/manual-desktop-v1/<scenario>/apkg.inspect.json`. Follow the
[scenario checklist](../contracts/fixtures/phase3/manual-desktop-v1/README.md)
to check the files in Anki Desktop.

## Roundtrip oracle

This optional check requires `docs/source/anki/rslib/Cargo.toml` and `protoc`
on `PATH`:

```sh
./scripts/run_roundtrip_oracle.sh
```

## Troubleshooting

| Failure | Action |
| --- | --- |
| Cannot discover `contracts/manifest.yaml` | Run repository tools from this checkout, with an explicit manifest. The normal Rust API embeds its contracts. |
| Python native extension unavailable | Install a matching native wheel or run `maturin develop` in a venv. See [Python setup](../bindings/python/README.md#from-a-source-checkout). |
| Missing upstream Anki crate | Provide the local Anki source checkout for the roundtrip oracle. |
| `protoc is required on PATH` | Install `protoc` before running the roundtrip oracle. |

## Design and releases

- [Contract change policy](process/contract-change-policy.md)
- [Architecture decisions](adr/README.md) and [RFCs](rfcs/README.md)
- [Rust release readiness](rust-crate-release-readiness.md) and
  [release runbook](rust-release-runbook.md)
- [Node release procedure](../bindings/node/RELEASING.md)
- [Benchmark methodology and reproduction](../benchmarks/README.md)
