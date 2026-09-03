# ADR 0007: Separate Crate and Bundle Versions

Publishing `anki_forge` introduces a Rust API compatibility promise in addition to the contract compatibility governed by ADR 0002. The crate's SemVer governs its public Rust API and behavior, while `bundle_version` governs its embedded contract set; they evolve independently, and every crate release identifies the bundle version it carries.

## Consequences

- Crate and bundle version numbers do not need to match.
- Release metadata must expose the crate-to-bundle version mapping.
- Compatibility checks must evaluate the Rust API and contract set on their respective axes.
