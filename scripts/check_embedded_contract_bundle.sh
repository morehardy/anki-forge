#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work_root="$(mktemp -d "${TMPDIR:-/tmp}/anki-forge-contract-bundle.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT

expected="$(find "$repo_root/anki_forge/assets/contracts" -maxdepth 1 \
  -name 'anki-forge-contract-bundle-*.tar.gz' -print)"
if [[ "$(printf '%s\n' "$expected" | sed '/^$/d' | wc -l | tr -d ' ')" != "1" ]]; then
  echo "expected exactly one embedded contract bundle asset" >&2
  exit 1
fi

cd "$repo_root"
cargo run --quiet -p contract_tools -- package \
  --manifest "$repo_root/contracts/manifest.yaml" \
  --out-dir "$work_root" >/dev/null

cmp "$expected" "$work_root/$(basename "$expected")"
