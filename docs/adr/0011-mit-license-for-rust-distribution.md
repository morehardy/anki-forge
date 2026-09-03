# ADR 0011: License the Rust Distribution Under MIT

The crates.io release needs explicit downstream usage rights, and the repository previously had no root license for its project-owned Rust distribution code. Project-owned source packaged in `anki_forge` is released under the MIT License; third-party mirrors such as the upstream Anki source are excluded from the crate and retain their own licenses.

## Consequences

- The published package includes the MIT license text and matching manifest metadata.
- Every source and asset included in the crate must be project-owned or compatible with MIT distribution.
- Upstream Anki source remains a behavioral oracle rather than distributed implementation code.
