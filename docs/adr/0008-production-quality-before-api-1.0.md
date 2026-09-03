# ADR 0008: Require Production Quality Before API 1.0

Operational readiness and long-term API stability are separate commitments. The first crates.io release is `anki_forge` 0.1.0: it must satisfy the full production release gates, while breaking public API changes remain possible through pre-1.0 minor-version increments; 1.0 follows validation by real downstream consumers.

## Consequences

- A 0.x version does not relax packaging, testing, documentation, or runtime requirements.
- Breaking changes increment the 0.x minor version and are documented.
- The 1.0 milestone requires evidence beyond successful publication.
