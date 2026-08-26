#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work_root="$(mktemp -d "${TMPDIR:-/tmp}/anki-forge-dependency-policy.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT
policy="$repo_root/docs/dependency-policy-exceptions.json"

jq -e 'all(.[]; (.owner | length > 0) and (.reason | length > 0) and (.expires | length > 0))' \
  "$policy" >/dev/null

today="$(date -u +%F)"
if jq -e --arg today "$today" 'any(.[]; .expires <= $today)' "$policy" >/dev/null; then
  echo "dependency policy exception is expired as of $today" >&2
  exit 1
fi

sed -n 's/.*{ name = "\([^"]*\)", version = "\([^"]*\)" }.*/\1 \2/p' \
  "$repo_root/deny.toml" | sort >"$work_root/deny.txt"
jq -r '.[] | "\(.crate) \(.version)"' "$policy" | sort >"$work_root/policy.txt"
diff -u "$work_root/deny.txt" "$work_root/policy.txt"
