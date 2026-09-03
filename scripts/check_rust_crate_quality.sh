#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
release_tag="${1:-}"

cd "$repo_root"
cargo fmt --all -- --check
cargo test -p anki_forge --lib \
  --test custom_notetype_api_tests \
  --test deck_project_facade_tests \
  --test packaged_contract_tests \
  --test stable_facade_boundary_tests \
  --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo test -p anki_forge --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc -p anki_forge --all-features --no-deps --locked
bash scripts/check_embedded_contract_bundle.sh
bash scripts/check_rust_crate_payload.sh
bash scripts/check_rust_release_metadata.sh "$release_tag"
bash scripts/check_dependency_policy_exceptions.sh
