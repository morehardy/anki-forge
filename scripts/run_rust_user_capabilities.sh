#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/run_rust_user_capabilities.sh [--keep-artifacts] [<scenario> ...]
  scripts/run_rust_user_capabilities.sh --manual-desktop <scenario>

Runs ignored Rust user API capability scenarios.
Automated all-scenario mode expects the complete first matrix. During incremental implementation, pass named scenarios.
USAGE
}

mode="automated"
keep_artifacts="false"
manual_scenario=""
selected=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --keep-artifacts)
      keep_artifacts="true"
      shift
      ;;
    --manual-desktop)
      mode="manual-desktop"
      shift
      if [[ $# -eq 0 ]]; then
        usage >&2
        exit 2
      fi
      manual_scenario="$1"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --*)
      usage >&2
      exit 2
      ;;
    *)
      selected+=("$1")
      shift
      ;;
  esac
done

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

list_file="$(mktemp "${TMPDIR:-/tmp}/anki-forge-rust-capabilities-list.XXXXXX")"
if ! cargo test -p anki_forge --features internal-tools \
  --test rust_user_capability_matrix -- --ignored --list >"$list_file"; then
  printf 'fail harness %s kept\n' "$list_file" >&2
  exit 2
fi

scenarios=()
while IFS= read -r scenario; do
  scenarios+=("$scenario")
done < <(awk -F: '/: test$/ { print $1 }' "$list_file" | grep -E '^[a-z0-9_]+$' || true)
if [[ "${#scenarios[@]}" -lt 1 ]]; then
  printf 'No capability scenarios discovered. Raw list: %s\n' "$list_file" >&2
  exit 2
fi
if [[ "${#scenarios[@]}" -lt 23 && "$mode" == "automated" && "${#selected[@]}" -eq 0 ]]; then
  printf 'Discovered %s scenarios, expected at least 23 after the full matrix lands. Raw list: %s\n' "${#scenarios[@]}" "$list_file" >&2
  exit 2
fi

contains_scenario() {
  local name="$1"
  local item
  for item in "${scenarios[@]}"; do
    [[ "$item" == "$name" ]] && return 0
  done
  return 1
}

if [[ "$mode" == "manual-desktop" ]]; then
  if [[ "${#selected[@]}" -eq 0 ]]; then
    selected=("$manual_scenario")
  else
    selected=("$manual_scenario" "${selected[@]}")
  fi
elif [[ "${#selected[@]}" -eq 0 ]]; then
  selected=("${scenarios[@]}")
fi

for scenario in "${selected[@]}"; do
  if ! contains_scenario "$scenario"; then
    printf 'Unknown scenario: %s\n' "$scenario" >&2
    printf 'Known scenarios:\n' >&2
    printf '  %s\n' "${scenarios[@]}" >&2
    rm -f "$list_file"
    exit 2
  fi
done

if [[ "$mode" == "manual-desktop" && "${#selected[@]}" -ne 1 ]]; then
  printf 'Manual desktop mode accepts exactly one scenario; got %s\n' "${#selected[@]}" >&2
  printf 'Known scenarios:\n' >&2
  printf '  %s\n' "${scenarios[@]}" >&2
  rm -f "$list_file"
  exit 2
fi

rm -f "$list_file"

if [[ "$mode" == "manual-desktop" ]]; then
  if ! command -v shasum >/dev/null 2>&1 && ! command -v sha256sum >/dev/null 2>&1 && ! command -v openssl >/dev/null 2>&1; then
    printf 'manual Desktop mode requires shasum, sha256sum, or openssl for package SHA-256\n' >&2
    exit 2
  fi
fi

sha256_file() {
  local path="$1"
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$path" | awk '{ print $1 }'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$path" | awk '{ print $1 }'
  else
    openssl dgst -sha256 "$path" | awk '{ print $NF }'
  fi
}

finalize_manual_artifacts() {
  local artifact_dir="$1"
  local package="$artifact_dir/package.apkg"
  local checklist="$artifact_dir/manual-checklist.md"
  local inspect="$artifact_dir/apkg.inspect.json"
  if [[ ! -f "$package" ]]; then
    printf 'manual Desktop scenario did not write expected package: %s\n' "$package" >&2
    return 1
  fi
  if [[ ! -f "$checklist" ]]; then
    printf 'manual Desktop scenario did not write expected checklist: %s\n' "$checklist" >&2
    return 1
  fi
  if [[ ! -f "$inspect" ]]; then
    printf 'manual Desktop scenario did not write expected inspect JSON: %s\n' "$inspect" >&2
    return 1
  fi
  if ! grep -q 'ANKI_FORGE_SHA256_PENDING' "$checklist"; then
    printf 'manual Desktop checklist missing SHA-256 placeholder: %s\n' "$checklist" >&2
    return 1
  fi
  local sha256
  if ! sha256="$(sha256_file "$package")" || ! [[ "$sha256" =~ ^[0-9a-fA-F]{64}$ ]]; then
    printf 'manual Desktop failed to compute package SHA-256: %s\n' "$package" >&2
    return 1
  fi
  local tmp
  if ! tmp="$(mktemp "$artifact_dir/manual-checklist.XXXXXX")"; then
    printf 'manual Desktop failed to create checklist temp file in: %s\n' "$artifact_dir" >&2
    return 1
  fi
  if ! sed "s/ANKI_FORGE_SHA256_PENDING/$sha256/g" "$checklist" >"$tmp"; then
    rm -f "$tmp"
    printf 'manual Desktop failed to finalize checklist SHA-256: %s\n' "$checklist" >&2
    return 1
  fi
  if ! mv "$tmp" "$checklist"; then
    rm -f "$tmp"
    printf 'manual Desktop failed to replace checklist: %s\n' "$checklist" >&2
    return 1
  fi
  if grep -q 'ANKI_FORGE_SHA256_PENDING' "$checklist"; then
    printf 'manual Desktop checklist still contains SHA-256 placeholder after finalization: %s\n' "$checklist" >&2
    return 1
  fi
}

run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"
passed=0
failed=0

for scenario in "${selected[@]}"; do
  if [[ "$mode" == "manual-desktop" ]]; then
    artifact_display_dir="tmp/manual-desktop-rust-api-v1/$scenario"
    artifact_dir="$repo_root/$artifact_display_dir"
    rm -rf "$artifact_dir"
  else
    artifact_display_dir="target/tmp/rust-user-capabilities/$run_id/$scenario"
    artifact_dir="$repo_root/$artifact_display_dir"
  fi
  mkdir -p "$artifact_dir"

  if ANKI_FORGE_CAPABILITY_MODE="$mode" \
    ANKI_FORGE_CAPABILITY_ARTIFACT_DIR="$artifact_dir" \
    cargo test -p anki_forge --features internal-tools \
      --test rust_user_capability_matrix "$scenario" -- --ignored --exact --nocapture
  then
    if [[ "$mode" == "manual-desktop" ]] && ! finalize_manual_artifacts "$artifact_dir"; then
      failed=$((failed + 1))
      printf 'fail %s %s kept\n' "$scenario" "$artifact_display_dir"
      printf 'summary total=%s passed=%s failed=%s skipped=0\n' "${#selected[@]}" "$passed" "$failed"
      exit 1
    fi
    passed=$((passed + 1))
    if [[ "$mode" == "automated" && "$keep_artifacts" == "false" ]]; then
      rm -rf "$artifact_dir"
      printf 'ok %s %s cleaned\n' "$scenario" "$artifact_display_dir"
    else
      printf 'ok %s %s kept\n' "$scenario" "$artifact_display_dir"
    fi
  else
    failed=$((failed + 1))
    printf 'fail %s %s kept\n' "$scenario" "$artifact_display_dir"
    printf 'summary total=%s passed=%s failed=%s skipped=0\n' "${#selected[@]}" "$passed" "$failed"
    exit 1
  fi
done

printf 'summary total=%s passed=%s failed=%s skipped=0\n' "${#selected[@]}" "$passed" "$failed"
