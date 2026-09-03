#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 4 ]]; then
  echo "usage: $0 <release-tag> <crate-path> <sbom-path> <output-dir>" >&2
  exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
release_tag="$1"
crate_path="$2"
sbom_path="$3"
output_dir="$4"

[[ "${ANKI_FORGE_TIER1_VERIFIED:-0}" == "1" ]] || {
  echo "release evidence requires a completed Tier 1 matrix" >&2
  exit 1
}

bash "$repo_root/scripts/check_rust_release_metadata.sh" "$release_tag" >/dev/null
bash "$repo_root/scripts/check_dependency_policy_exceptions.sh"
[[ -f "$crate_path" ]] || { echo "crate archive is missing: $crate_path" >&2; exit 1; }
[[ -f "$sbom_path" ]] || { echo "SBOM is missing: $sbom_path" >&2; exit 1; }

mkdir -p "$output_dir"
cp "$crate_path" "$output_dir/"
cp "$sbom_path" "$output_dir/anki_forge.cdx.json"
cp "$repo_root/anki_forge/CHANGELOG.md" "$output_dir/CHANGELOG.md"
cp "$repo_root/docs/dependency-policy-exceptions.json" "$output_dir/"

if command -v sha256sum >/dev/null 2>&1; then
  checksum="$(sha256sum "$crate_path" | cut -d ' ' -f1)"
else
  checksum="$(shasum -a 256 "$crate_path" | cut -d ' ' -f1)"
fi
printf '%s  %s\n' "$checksum" "$(basename "$crate_path")" >"$output_dir/SHA256SUMS"

version="${release_tag#anki-forge-v}"
bundle_version="$(bash "$repo_root/scripts/check_rust_release_metadata.sh" "$release_tag" \
  | sed -n 's/^bundle_version=//p')"
commit="$(git -C "$repo_root" rev-parse HEAD)"
ci_evidence="${GITHUB_SERVER_URL:-https://github.com}/${GITHUB_REPOSITORY:-morehardy/anki-forge}/actions/runs/${GITHUB_RUN_ID:-local-rehearsal}"

jq -n \
  --arg tag "$release_tag" \
  --arg commit "$commit" \
  --arg crate_version "$version" \
  --arg bundle_version "$bundle_version" \
  --arg package_sha256 "$checksum" \
  --arg ci_evidence "$ci_evidence" \
  --slurpfile exceptions "$repo_root/docs/dependency-policy-exceptions.json" \
  '{schema_version:"anki-forge-rust-release-v1",tag:$tag,commit:$commit,crate_version:$crate_version,bundle_version:$bundle_version,msrv:"1.92.0",stable_verified:true,tier1_platforms:["x86_64-unknown-linux-gnu","x86_64-pc-windows-msvc","x86_64-apple-darwin","aarch64-apple-darwin"],package_sha256:$package_sha256,sbom:"anki_forge.cdx.json",changelog:"CHANGELOG.md",ci_evidence:$ci_evidence,dependency_policy_exceptions:$exceptions[0]}' \
  >"$output_dir/release-record.json"
