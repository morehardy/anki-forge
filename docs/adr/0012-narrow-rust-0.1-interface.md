# ADR 0012: Narrow the Rust 0.1 Consumer Interface

The first production crate has one supported external seam:
`anki_forge::prelude`, plus root conveniences for `Deck`, `Project`, `Severity`,
and version inspection. Repository contract tooling uses a separate hidden
`internal-tools` interface because exposing normalization, writer, inspection,
and persistence modules would create a much larger compatibility promise than
normal consumers need.

## Consequences

- New consumer capability enters through the facade only after its interface,
  diagnostics, examples, and compatibility commitment are reviewed.
- Internal modules can evolve with the contract implementation without
  expanding the promised Rust surface.
- CI runs both default-feature consumer tests and `--all-features` repository
  tests; the packaged-consumer test must use only the supported facade.
- Enabling `internal-tools` from a downstream application is unsupported and
  may break in any release.
