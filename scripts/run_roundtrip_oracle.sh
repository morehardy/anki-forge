#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/.." && pwd)
MANIFEST_PATH="$SCRIPT_DIR/roundtrip_oracle/Cargo.toml"
if [[ $# -gt 1 ]]; then
  echo "usage: scripts/run_roundtrip_oracle.sh [evidence-directory]" >&2
  exit 2
fi
if [[ ! -f "$REPO_ROOT/docs/source/anki/rslib/Cargo.toml" ]]; then
  echo "missing local upstream Anki source at docs/source/anki/rslib" >&2
  exit 1
fi
if ! command -v protoc >/dev/null 2>&1; then
  echo "protoc is required for the real Anki oracle" >&2
  exit 1
fi
EVIDENCE_DIR="${1:-$REPO_ROOT/target/native-roundtrip-oracle}"
mkdir -p "$EVIDENCE_DIR"
EVIDENCE_DIR=$(cd -- "$EVIDENCE_DIR" && pwd)
cd "$REPO_ROOT"
cargo run --offline --locked -p ankiforge --no-default-features \
  --example native_roundtrip_oracle_prepare -- "$EVIDENCE_DIR/prepared-input.json"
cargo run --offline --locked --manifest-path "$MANIFEST_PATH" \
  --bin roundtrip_oracle -- "$EVIDENCE_DIR/prepared-input.json" "$EVIDENCE_DIR/report.json" \
  > "$EVIDENCE_DIR/oracle.log"
echo "Real Anki import/update evidence: $EVIDENCE_DIR/report.json"
