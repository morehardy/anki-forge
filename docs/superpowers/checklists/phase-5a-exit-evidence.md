# Phase 5A Exit Evidence

The real upstream importer oracle additionally expects a local Anki source tree at
`docs/source/anki`, Rust `1.92.0`, and `protoc` on `PATH`.

Run the oracle explicitly with `./scripts/run_roundtrip_oracle.sh`; it is not part of the default
`cargo test` suite.

- `cargo test -p anki_forge --test product_model_tests -v`
- `cargo test -p anki_forge --test product_lowering_tests -v`
- `cargo test -p anki_forge --test product_helper_tests -v`
- `cargo test -p anki_forge --test product_bundler_tests -v`
- `cargo test -p anki_forge --test product_pipeline_tests -v`
- `cargo test -p anki_forge --test product_portability_tests -v`
- `cargo test -p authoring_core --test normalization_pipeline_tests -v`
- `cargo test -p writer_core --test build_tests -v`
- `cargo test -p writer_core --test inspect_tests -v`
- `cargo test -p contract_tools --test schema_gate_tests -v`
- `./scripts/run_roundtrip_oracle.sh`
- `cargo run -p anki_forge --example product_basic_flow`
