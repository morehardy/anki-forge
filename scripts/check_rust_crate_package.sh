#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work_root="$(mktemp -d "${TMPDIR:-/tmp}/anki-forge-package-smoke.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT

package_target="$work_root/package-target"
consumer_root="$work_root/consumer"
consumer_target="$work_root/consumer-target"
package_args=(-p ankiforge --locked --offline)
if [[ "${ANKI_FORGE_ALLOW_DIRTY_PACKAGE:-0}" == "1" ]]; then
  package_args+=(--allow-dirty)
fi

cd "$repo_root"
CARGO_TARGET_DIR="$package_target" \
  cargo package "${package_args[@]}"

package_dir=""
for candidate in "$package_target"/package/ankiforge-*; do
  [[ -d "$candidate" ]] || continue
  if [[ -n "$package_dir" ]]; then
    echo "multiple packaged ankiforge source directories were created" >&2
    exit 1
  fi
  package_dir="$candidate"
done
if [[ -z "$package_dir" ]]; then
  echo "packaged ankiforge source directory was not created" >&2
  exit 1
fi

# Cargo resolves dependency paths relative to the consumer manifest. Keeping
# this relative avoids leaking an MSYS /tmp path into Cargo.toml on Windows.
package_dependency_path="../package-target/package/$(basename "$package_dir")"

mkdir -p "$consumer_root"
cp -R "$repo_root/scripts/packaged_consumer/." "$consumer_root/"
cat >"$consumer_root/Cargo.toml" <<EOF
[package]
name = "anki_forge_packaged_consumer"
version = "0.0.0"
edition = "2021"
rust-version = "1.92"

[workspace]

[dependencies]
ankiforge = { path = "$package_dependency_path", default-features = false }
anyhow = "1"
serde_json = "1"
prost = "0.13"
rusqlite = { version = "0.32", features = ["bundled"] }
zip = { version = "2", default-features = false, features = ["deflate"] }
zstd = "0.13"
tempfile = "3"
EOF

# Runtime working directory and every fixture are outside the repository.
# Dependency pins are resolved from the already populated offline cache.
(
  cd "$consumer_root"
  CARGO_TARGET_DIR="$consumer_target" cargo generate-lockfile --offline
  CARGO_TARGET_DIR="$consumer_target" cargo run --offline --locked --bin anki_forge_packaged_consumer
)
