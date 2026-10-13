#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work_root="$(mktemp -d "${TMPDIR:-/tmp}/anki-forge-dependency-policy.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT
if [[ "$#" -gt 1 ]]; then
  echo "usage: $0 [policy-file]" >&2
  exit 2
fi
policy="${1:-$repo_root/docs/dependency-policy-exceptions.json}"

jq -e '
  def valid_date:
    type == "string"
    and test("^[0-9]{4}-[0-9]{2}-[0-9]{2}$")
    and (. as $date
      | try ((. + "T00:00:00Z" | fromdateiso8601 | strftime("%Y-%m-%d")) == $date)
        catch false);
  all(.[];
    (.owner | type == "string" and length > 0)
    and (.reason | type == "string" and length > 0)
    and (.expires | valid_date))
' \
  "$policy" >/dev/null

today="$(date -u +%F)"
if jq -e --arg today "$today" 'any(.[]; .expires <= $today)' "$policy" >/dev/null; then
  echo "dependency policy exception is expired as of $today" >&2
  exit 1
fi

sed -n 's/.*{ name = "\([^"]*\)", version = "\([^"]*\)" }.*/\1 \2/p' \
  "$repo_root/deny.toml" | sort >"$work_root/deny.txt"
jq -r '.[] | select(.advisory == null) | "\(.crate) \(.version)"' "$policy" | sort >"$work_root/policy.txt"
diff -u "$work_root/deny.txt" "$work_root/policy.txt"

python3 - "$repo_root" "$policy" <<'PY'
import json
from pathlib import Path
import sys
import tomllib

root = Path(sys.argv[1])
policy = json.loads(Path(sys.argv[2]).read_text())
configuration = tomllib.loads((root / "deny.toml").read_text())
ignored = configuration.get("advisories", {}).get("ignore", [])
actual = {entry if isinstance(entry, str) else entry["id"] for entry in ignored}
records = [entry for entry in policy if entry.get("advisory")]
expected = {entry["advisory"] for entry in records}
if actual != expected:
    raise SystemExit("ignored advisories must exactly match documented policy exceptions")
locked = tomllib.loads((root / "Cargo.lock").read_text())["package"]
for entry in records:
    versions = {package["version"] for package in locked if package["name"] == entry["crate"]}
    if versions != {entry["version"]}:
        raise SystemExit(f"advisory exception version differs from Cargo.lock: {entry['crate']}")
PY
