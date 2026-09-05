#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# CI supplies the PR base SHA or push-before SHA. Local branches compare with
# their merge-base against origin/main so unrelated main commits do not count.
if [[ -n "${CONTRACT_BASE_REF:-}" ]]; then
  baseline="$(git rev-parse --verify --end-of-options "${CONTRACT_BASE_REF}^{commit}")"
else
  baseline="$(git merge-base origin/main HEAD)"
fi

work_root="$(mktemp -d "${TMPDIR:-/tmp}/anki-forge-governance.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT
git archive "$baseline" contracts | tar -x -C "$work_root"

cargo run --quiet -p contract_tools -- verify \
  --manifest "$repo_root/contracts/manifest.yaml" \
  --source-root "$repo_root" \
  --baseline-manifest "$work_root/contracts/manifest.yaml"
