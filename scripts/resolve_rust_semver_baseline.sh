#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="$repo_root/anki_forge/Cargo.toml"
crate_version="$(awk '
  /^\[package\]$/ { in_package = 1; next }
  /^\[/ { in_package = 0 }
  in_package && /^version = / { gsub(/version = |"/, ""); print; exit }
' "$manifest")"

[[ -n "$crate_version" ]] || {
  echo "failed to resolve anki_forge crate version" >&2
  exit 1
}

printf 'crate_version=%s\n' "$crate_version"
if [[ "$crate_version" != "0.1.0" ]]; then
  printf 'required=true\n'
  exit 0
fi

# Version 0.1.0 has no baseline only until the first publication. Afterwards,
# CI must compare against the registry release even if a branch forgot to bump
# its local manifest version.
registry_api="${ANKI_FORGE_CRATES_IO_API_BASE:-https://crates.io/api/v1/crates}"
status="$(curl --silent --show-error --location --retry 3 --retry-all-errors \
  --user-agent 'anki-forge-release-ci/0.1 (https://github.com/morehardy/anki-forge)' \
  --output /dev/null --write-out '%{http_code}' \
  "$registry_api/anki_forge/$crate_version")"
case "$status" in
  200)
    printf 'required=true\n'
    ;;
  404)
    printf 'required=false\n'
    ;;
  *)
    echo "unexpected crates.io response while resolving SemVer baseline: HTTP $status" >&2
    exit 1
    ;;
esac
