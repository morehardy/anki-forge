#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work_root="$(mktemp -d "${TMPDIR:-/tmp}/anki-forge-package-smoke.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT

package_target="$work_root/package-target"
consumer_root="$work_root/consumer"
consumer_target="$work_root/consumer-target"
dirty_args=()
if [[ "${ANKI_FORGE_ALLOW_DIRTY_PACKAGE:-0}" == "1" ]]; then
  dirty_args+=(--allow-dirty)
fi

cd "$repo_root"
CARGO_TARGET_DIR="$package_target" \
  cargo package -p anki_forge "${dirty_args[@]}" --locked --offline

package_dir="$(find "$package_target/package" \
  -mindepth 1 -maxdepth 1 -type d -name 'anki_forge-*' -print -quit)"
if [[ -z "$package_dir" ]]; then
  echo "packaged anki_forge source directory was not created" >&2
  exit 1
fi

mkdir -p "$consumer_root/src"
cat >"$consumer_root/Cargo.toml" <<EOF
[package]
name = "anki_forge_packaged_consumer"
version = "0.0.0"
edition = "2021"
rust-version = "1.92"

[dependencies]
anki_forge = { path = "$package_dir" }
EOF

cat >"$consumer_root/src/main.rs" <<'EOF'
use anki_forge::runtime::{
    embedded_bundle_version, load_default_writer_stack, RuntimeMode,
};

fn main() {
    assert_eq!(anki_forge::facade_api_version(), "0.1.0");
    assert_eq!(embedded_bundle_version(), "0.3.0");

    let (runtime, _writer_policy, _build_context) =
        load_default_writer_stack().expect("load embedded default writer stack");
    assert_eq!(runtime.mode, RuntimeMode::Installed);
    assert_eq!(runtime.bundle_version, "0.3.0");
}
EOF

CARGO_TARGET_DIR="$consumer_target" \
  cargo generate-lockfile --manifest-path "$consumer_root/Cargo.toml"
CARGO_TARGET_DIR="$consumer_target" \
  cargo fetch --manifest-path "$consumer_root/Cargo.toml" --locked
CARGO_TARGET_DIR="$consumer_target" \
  cargo run --manifest-path "$consumer_root/Cargo.toml" --offline --locked
