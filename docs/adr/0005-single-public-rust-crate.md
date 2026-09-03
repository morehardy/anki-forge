# ADR 0005: Publish a Single Rust Crate

The Rust workspace separates its public API from authoring and writing cores, but publishing those cores would expose internal boundaries and couple multiple release versions and ordering. crates.io therefore publishes only `anki_forge`; authoring and writing remain internal implementation boundaries of that package, and internal verification tools are not release products.

## Consequences

- Downstream consumers depend on one public version.
- Packaging must not require unpublished workspace crates at registry resolution time.
- Internal boundaries may evolve without creating additional public compatibility promises.
