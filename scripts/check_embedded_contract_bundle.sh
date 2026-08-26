#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work_root="$(mktemp -d "${TMPDIR:-/tmp}/anki-forge-contract-bundle.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT

bundle_version="$(sed -n 's/^const EMBEDDED_BUNDLE_VERSION: &str = "\([^"]*\)";/\1/p' \
  "$repo_root/anki_forge/src/runtime/embedded.rs")"
[[ -n "$bundle_version" ]] || {
  echo "failed to resolve embedded bundle version" >&2
  exit 1
}
expected="$repo_root/anki_forge/assets/contracts/anki-forge-contract-bundle-$bundle_version.tar.gz"

cd "$repo_root"
cargo run --quiet -p contract_tools -- package \
  --manifest "$repo_root/contracts/manifest.yaml" \
  --out-dir "$work_root" >/dev/null

cmp "$expected" "$work_root/$(basename "$expected")"
