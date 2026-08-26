# ADR 0010: Use Tag-Driven Trusted Publishing

Production publication needs one auditable authority without developer-machine state or long-lived registry secrets. A protected `anki-forge-vX.Y.Z` tag whose version matches `Cargo.toml` is the only formal release trigger; CI publishes to crates.io through Trusted Publishing, and local `cargo publish` is not part of the release process.

## Consequences

- Release permissions are enforced through tag protection and CI environment controls.
- The workflow fails before publication when the tag and manifest versions differ.
- Maintainers do not store crates.io publication tokens in developer environments or repository secrets.
