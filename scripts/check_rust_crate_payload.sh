#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work_root="$(mktemp -d "${TMPDIR:-/tmp}/anki-forge-payload.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT
package_args=(-p anki_forge --locked --offline --list)
if [[ "${ANKI_FORGE_ALLOW_DIRTY_PACKAGE:-0}" == "1" ]]; then
  package_args+=(--allow-dirty)
fi

cd "$repo_root"
cargo package "${package_args[@]}" \
  | sed 's/\r$//' >"$work_root/files.txt"

diff -u "$repo_root/anki_forge/PACKAGE_FILES.txt" "$work_root/files.txt"

required=(
  "Cargo.toml"
  "Cargo.lock"
  "README.md"
  "CHANGELOG.md"
  "LICENSE"
  "PACKAGE_FILES.txt"
  "src/lib.rs"
  "assets/contracts/anki-forge-contract-bundle-0.3.0.tar.gz"
  "tests/packaged_contract_tests.rs"
)

for path in "${required[@]}"; do
  grep -Fxq "$path" "$work_root/files.txt" || {
    echo "required package file is missing: $path" >&2
    exit 1
  }
done

for forbidden in "../" "assets/rslib/" "contract_tools/" "docs/source/" "target/"; do
  if grep -Fq "$forbidden" "$work_root/files.txt"; then
    echo "forbidden package payload entry contains: $forbidden" >&2
    exit 1
  fi
done
