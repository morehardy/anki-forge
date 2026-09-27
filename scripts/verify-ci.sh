#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/verify-ci.sh [--fast|--ci]

Runs the verification gates expected before a PR is marked ready.

Modes:
  --fast  Rust formatting, contract governance, clippy, workspace tests, and whitespace checks.
  --ci    Full local mirror of .github/workflows/contract-ci.yml. This is the default.
USAGE
}

mode="ci"
case "${1:-}" in
  "" | --ci | ci)
    mode="ci"
    ;;
  --fast | fast)
    mode="fast"
    ;;
  -h | --help | help)
    usage
    exit 0
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

run() {
  printf '\n==> %s\n' "$*"
  "$@"
}

check_branch_whitespace() {
  printf '\n==> git diff --check origin/main...HEAD\n'
  if ! git rev-parse --verify --quiet origin/main >/dev/null; then
    printf 'origin/main is missing. Run `git fetch origin main` before `make verify-ci`.\n' >&2
    exit 1
  fi
  git diff --check origin/main...HEAD
}

check_worktree_whitespace() {
  run git diff --check
  run git diff --cached --check
}

manifest_path="$repo_root/contracts/manifest.yaml"
dist_dir="$repo_root/dist"
python_path="$repo_root/bindings/python/src"

run cargo fmt --all -- --check
run bash ./scripts/check_contract_governance.sh
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace -v
if [[ "$mode" == "ci" ]]; then
  run ./scripts/run_rust_user_capabilities.sh
fi
check_worktree_whitespace
check_branch_whitespace

if [[ "$mode" == "fast" ]]; then
  printf '\nverify-fast passed\n'
  exit 0
fi

run cargo test -p ankiforge --features internal-tools --example conformance_surface
run cargo run -p ankiforge --features internal-tools --example minimal_flow
run npm --prefix bindings/node run setup
run npm --prefix bindings/node run build
run npm --prefix bindings/node run check
run npm --prefix bindings/node run test:product
run npm --prefix bindings/node run test:installed
run npm --prefix bindings/node run example:minimal
run npm --prefix bindings/node run check:package
run cargo build -p contract_tools --release
run python3 -m venv target/python-ci-venv
run target/python-ci-venv/bin/python -m pip install pytest==9.1.1 mypy==2.3.1 maturin==1.15.0
run env "VIRTUAL_ENV=$repo_root/target/python-ci-venv" target/python-ci-venv/bin/maturin develop --manifest-path bindings/python/native/Cargo.toml --locked
run cargo build -p anki_forge_python_native --example python_parity --locked
run target/python-ci-venv/bin/python -m mypy --config-file bindings/python/pyproject.toml bindings/python/src/anki_forge
run env "PYTHONPATH=$python_path" target/python-ci-venv/bin/python -m pytest bindings/python/tests -q
run env "PYTHONPATH=$python_path" target/python-ci-venv/bin/python bindings/python/examples/minimal_flow.py
run target/python-ci-venv/bin/python bindings/python/examples/native_workflow.py target/python-ci-example
run cargo run -p contract_tools -- verify --manifest "$manifest_path"
run cargo run -p contract_tools -- summary --manifest "$manifest_path"
run cargo run -p contract_tools -- package --manifest "$manifest_path" --out-dir "$dist_dir"

printf '\nverify-ci passed\n'
