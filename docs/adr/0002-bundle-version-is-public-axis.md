# ADR 0002: Bundle Version Is the Public Axis

## Context

The bundle contains internal component versions, but compatibility within the
contract set needs one public coordinate that stays stable across those assets.

## Decision

Within the contract bundle, `bundle_version` is the only public compatibility axis. The
`compatibility.public_axis` field must remain `bundle_version`, and other
component versions are internal bundle bookkeeping.

## Consequences

- Compatibility discussions stay centered on a single public version.
- Internal asset evolution can continue without inventing new public axes.
- Tooling can reject manifests that try to promote a different axis.

The separately published Rust crate has its own SemVer compatibility axis, as
recorded in ADR 0007.
