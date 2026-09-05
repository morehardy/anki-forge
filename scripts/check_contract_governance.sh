#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

release_mode=false
if [[ "$#" -gt 0 ]]; then
  if [[ "$#" -ne 1 || "$1" != "--release" ]]; then
    echo "usage: $0 [--release]" >&2
    exit 2
  fi
  release_mode=true
fi
if [[ "$release_mode" == true && -z "${CONTRACT_BASE_REF:-}" ]]; then
  echo "release verification requires an explicit previous CONTRACT_BASE_REF" >&2
  exit 1
fi

# CI supplies the PR base SHA or push-before SHA. Local branches compare with
# their merge-base against origin/main so unrelated main commits do not count.
if [[ -n "${CONTRACT_BASE_REF:-}" ]]; then
  baseline="$(git rev-parse --verify --end-of-options "${CONTRACT_BASE_REF}^{commit}")"
else
  baseline="$(git merge-base origin/main HEAD)"
fi

if [[ "$release_mode" == true ]]; then
  current_commit="$(git rev-parse --verify 'HEAD^{commit}')"
  if [[ "$baseline" == "$current_commit" ]] ||
    ! git merge-base --is-ancestor "$baseline" "$current_commit"; then
    echo "release baseline must be a strict ancestor of the current commit: $baseline" >&2
    exit 1
  fi
fi

work_root="$(mktemp -d "${TMPDIR:-/tmp}/anki-forge-governance.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT
git archive "$baseline" contracts | tar -x -C "$work_root"

verify_args=(
  --manifest "$repo_root/contracts/manifest.yaml"
  --source-root "$repo_root"
  --baseline-manifest "$work_root/contracts/manifest.yaml"
)
if [[ "$release_mode" == true ]]; then
  verify_args+=(--release)
fi
cargo run --quiet -p contract_tools -- verify "${verify_args[@]}"
