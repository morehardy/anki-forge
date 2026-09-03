# ADR 0006: Make the Rust Distribution Self-Contained

The public crate must behave consistently when installed from crates.io, but repository-relative contract discovery makes behavior depend on the caller's working directory and source layout. `anki_forge` therefore carries its default contract resources within the published package and does not require a source checkout or separately installed contract bundle for normal use.

## Consequences

- A fresh downstream project can use the public API without repository setup.
- Release verification must exercise the packaged crate outside the workspace.
- Changes to embedded contract resources are part of a crate release.
