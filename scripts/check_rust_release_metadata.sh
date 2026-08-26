#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="$repo_root/anki_forge/Cargo.toml"
changelog="$repo_root/anki_forge/CHANGELOG.md"
release_tag="${1:-}"

crate_version="$(awk '
  /^\[package\]$/ { in_package = 1; next }
  /^\[/ { in_package = 0 }
  in_package && /^version = / { gsub(/version = |"/, ""); print; exit }
' "$manifest")"
bundle_asset="$(find "$repo_root/anki_forge/assets/contracts" -maxdepth 1 \
  -name 'anki-forge-contract-bundle-*.tar.gz' -print)"
if [[ "$(printf '%s\n' "$bundle_asset" | sed '/^$/d' | wc -l | tr -d ' ')" != "1" ]]; then
  echo "expected exactly one embedded contract bundle asset" >&2
  exit 1
fi
bundle_filename="$(basename "$bundle_asset")"
bundle_version="${bundle_filename#anki-forge-contract-bundle-}"
bundle_version="${bundle_version%.tar.gz}"

if [[ -z "$crate_version" || -z "$bundle_version" ]]; then
  echo "failed to resolve crate or embedded bundle version" >&2
  exit 1
fi

expected_asset="$bundle_asset"
[[ -f "$expected_asset" ]] || {
  echo "embedded bundle asset is missing: $expected_asset" >&2
  exit 1
}

grep -Fq "## [$crate_version]" "$changelog" || {
  echo "CHANGELOG.md has no release entry for $crate_version" >&2
  exit 1
}

grep -Fq "bundle \`$bundle_version\`" "$repo_root/anki_forge/README.md" || {
  echo "crate README does not record embedded bundle $bundle_version" >&2
  exit 1
}

cmp "$repo_root/LICENSE" "$repo_root/anki_forge/LICENSE"

if [[ -n "$release_tag" ]]; then
  expected_tag="anki-forge-v$crate_version"
  [[ "$release_tag" == "$expected_tag" ]] || {
    echo "release tag mismatch: expected $expected_tag, got $release_tag" >&2
    exit 1
  }
fi

printf 'crate_version=%s\nbundle_version=%s\n' "$crate_version" "$bundle_version"
