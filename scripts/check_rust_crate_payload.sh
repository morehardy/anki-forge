#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work_root="$(mktemp -d "${TMPDIR:-/tmp}/anki-forge-payload.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT
dirty_args=()
if [[ "${ANKI_FORGE_ALLOW_DIRTY_PACKAGE:-0}" == "1" ]]; then
  dirty_args+=(--allow-dirty)
fi

cd "$repo_root"
cargo package -p anki_forge "${dirty_args[@]}" --locked --offline --list >"$work_root/files.txt"

required=(
  "Cargo.toml"
  "Cargo.lock"
  "README.md"
  "CHANGELOG.md"
  "LICENSE"
  "src/lib.rs"
  "assets/contracts/anki-forge-contract-bundle-0.3.0.tar.gz"
  "assets/rslib/storage/schema11.sql"
  "tests/packaged_contract_tests.rs"
)

for path in "${required[@]}"; do
  grep -Fxq "$path" "$work_root/files.txt" || {
    echo "required package file is missing: $path" >&2
    exit 1
  }
done

for forbidden in "../" "contract_tools/" "docs/source/" "target/"; do
  if grep -Fq "$forbidden" "$work_root/files.txt"; then
    echo "forbidden package payload entry contains: $forbidden" >&2
    exit 1
  fi
done
