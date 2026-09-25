#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
release_tag="${1:-}"

cd "$repo_root"
cargo fmt --all -- --check
cargo test -p ankiforge --lib \
  --test custom_notetype_api_tests \
  --test project_value_tests \
  --test packaged_contract_tests \
  --test stable_facade_boundary_tests \
  --test public_surface_contract_tests \
  --test media_snapshot_lifecycle_tests \
  --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo test -p ankiforge --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc -p ankiforge --all-features --no-deps --locked
bash scripts/check_embedded_contract_bundle.sh
bash scripts/check_rust_crate_payload.sh
bash scripts/check_rust_release_metadata.sh "$release_tag"
bash scripts/check_dependency_policy_exceptions.sh
