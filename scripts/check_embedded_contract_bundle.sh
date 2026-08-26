#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work_root="$(mktemp -d "${TMPDIR:-/tmp}/anki-forge-contract-bundle.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT

expected="$repo_root/anki_forge/assets/contracts/anki-forge-contract-bundle-0.3.0.tar.gz"

cd "$repo_root"
cargo run --quiet -p contract_tools -- package \
  --manifest "$repo_root/contracts/manifest.yaml" \
  --out-dir "$work_root" >/dev/null

cmp "$expected" "$work_root/$(basename "$expected")"
