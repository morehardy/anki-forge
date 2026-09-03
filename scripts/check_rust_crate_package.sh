#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work_root="$(mktemp -d "${TMPDIR:-/tmp}/anki-forge-package-smoke.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT

package_target="$work_root/package-target"
consumer_root="$work_root/consumer"
consumer_target="$work_root/consumer-target"
package_args=(-p anki_forge --locked --offline)
if [[ "${ANKI_FORGE_ALLOW_DIRTY_PACKAGE:-0}" == "1" ]]; then
  package_args+=(--allow-dirty)
fi

cd "$repo_root"
CARGO_TARGET_DIR="$package_target" \
  cargo package "${package_args[@]}"

package_dir=""
for candidate in "$package_target"/package/anki_forge-*; do
  [[ -d "$candidate" ]] || continue
  if [[ -n "$package_dir" ]]; then
    echo "multiple packaged anki_forge source directories were created" >&2
    exit 1
  fi
  package_dir="$candidate"
done
if [[ -z "$package_dir" ]]; then
  echo "packaged anki_forge source directory was not created" >&2
  exit 1
fi

# Cargo resolves dependency paths relative to the consumer manifest. Keeping
# this relative avoids leaking an MSYS /tmp path into Cargo.toml on Windows.
package_dependency_path="../package-target/package/$(basename "$package_dir")"

mkdir -p "$consumer_root/src"
cat >"$consumer_root/Cargo.toml" <<EOF
[package]
name = "anki_forge_packaged_consumer"
version = "0.0.0"
edition = "2021"
rust-version = "1.92"

[dependencies]
anki_forge = { path = "$package_dependency_path" }
EOF

cat >"$consumer_root/src/main.rs" <<'EOF'
use anki_forge::prelude::*;

fn main() {
    assert_eq!(anki_forge::facade_api_version(), "0.1.0");
    assert!(!anki_forge::embedded_contract_version().is_empty());

    let apkg = std::env::temp_dir().join(format!(
        "anki-forge-packaged-consumer-{}.apkg",
        std::process::id()
    ));
    let mut deck = Deck::new("Packaged Consumer");
    deck.basic()
        .note("front", "back")
        .stable_id("packaged:consumer")
        .add()
        .expect("add packaged note");
    deck.write_apkg(&apkg)
        .expect("build through embedded contracts")
        .ensure_success()
        .expect("successful packaged build");
    assert!(apkg.is_file());
    std::fs::remove_file(apkg).expect("remove packaged consumer artifact");
}
EOF

CARGO_TARGET_DIR="$consumer_target" \
  cargo generate-lockfile --manifest-path "$consumer_root/Cargo.toml"
CARGO_TARGET_DIR="$consumer_target" \
  cargo fetch --manifest-path "$consumer_root/Cargo.toml" --locked
CARGO_TARGET_DIR="$consumer_target" \
  cargo run --manifest-path "$consumer_root/Cargo.toml" --offline --locked
