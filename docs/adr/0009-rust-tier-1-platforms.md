# ADR 0009: Define Four Tier 1 Rust Targets

The public crate needs a finite, testable support promise across the main desktop environments. Linux x86_64, Windows x86_64, macOS x86_64, and macOS ARM64 are Tier 1 and must pass release CI; other Rust targets are best-effort, and WebAssembly is not a Tier 1 commitment.

## Consequences

- A release is blocked when a Tier 1 target fails its required checks.
- CI must exercise all four target and operating-system combinations.
- Functionality on other targets does not imply an ongoing compatibility guarantee.
