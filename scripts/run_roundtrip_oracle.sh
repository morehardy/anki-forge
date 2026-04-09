#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/.." && pwd)
WORKTREE_ANKI_DIR="$REPO_ROOT/docs/source/anki"
ANKI_RSLIB_DIR="$WORKTREE_ANKI_DIR/rslib"
MANIFEST_PATH="$REPO_ROOT/scripts/roundtrip_oracle/Cargo.toml"
CREATED_LINK=""
TMP_ROOT=""

resolve_shared_repo_root() {
  case "$REPO_ROOT" in
    */.worktrees/*)
      printf '%s\n' "${REPO_ROOT%%/.worktrees/*}"
      ;;
    *)
      return 1
      ;;
  esac
}

cleanup() {
  if [[ -n "$TMP_ROOT" ]]; then
    rm -rf "$TMP_ROOT"
  fi
  if [[ -n "$CREATED_LINK" ]]; then
    rm -f "$CREATED_LINK"
  fi
}

trap cleanup EXIT

if [[ ! -f "$ANKI_RSLIB_DIR/Cargo.toml" ]]; then
  if SHARED_REPO_ROOT=$(resolve_shared_repo_root 2>/dev/null); then
    SHARED_ANKI_DIR="$SHARED_REPO_ROOT/docs/source/anki"
    if [[ -f "$SHARED_ANKI_DIR/rslib/Cargo.toml" && ! -e "$WORKTREE_ANKI_DIR" ]]; then
      mkdir -p "$(dirname "$WORKTREE_ANKI_DIR")"
      ln -s "$SHARED_ANKI_DIR" "$WORKTREE_ANKI_DIR"
      CREATED_LINK="$WORKTREE_ANKI_DIR"
    fi
  fi
fi

if [[ ! -f "$ANKI_RSLIB_DIR/Cargo.toml" ]]; then
  echo "missing vendored upstream Anki crate at $ANKI_RSLIB_DIR" >&2
  exit 1
fi

if ! command -v protoc >/dev/null 2>&1; then
  echo "protoc is required on PATH for the local roundtrip oracle" >&2
  exit 1
fi

TMP_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/roundtrip-oracle.XXXXXX")
PREPARED_INPUT="$TMP_ROOT/prepared-input.json"

cargo run --locked -p anki_forge --example product_roundtrip_oracle_prepare -- "$PREPARED_INPUT" "$@"
cargo run --manifest-path "$MANIFEST_PATH" --locked -- "$PREPARED_INPUT"
