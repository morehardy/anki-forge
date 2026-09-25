# ADR 0012: Narrow the Rust Consumer Interface

Updated 2026-09-24 by the [clean-slate design](../plans/2026-09-23-rust-api-clean-slate-design.md).

The sole authoring container is `Project`. Common owned values are available at
crate root; named domains `note`, `schema`, `media`, `build`, `update`, and
`diagnostics` contain advanced types. Every public return type is nameable without
an extension trait or a repository-only feature.

`prelude`, `Deck`, public implementation modules and `Project` lowering methods
are removed. Internal authoring and writing cores remain private. The hidden
`tools` module requires `internal-tools` and exposes only operations used by
repository contract tooling; its DTOs do not expose whole implementation trees.

Default independent consumers exercise the supported API. Negative compilation
probes verify inaccessible internals and retired entrances. Repository conformance
tests can inspect private algorithms; SDKs use only the default public API.
Packaged-consumer tests use the extracted crate outside this workspace.

The clean-slate implementation makes no promise to preserve the former interface.
Future compatible releases remain governed by the crate's SemVer policy.
