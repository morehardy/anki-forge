# Local implementation review

- Fixed-size owned blocks cap total pool capacity, including free buffers.
- Exhaustion spills immediately; ordered output never waits for pool credit.
- CRC/length advance only for bytes accepted by Write. A failed spill keeps the
  original blocks until the temporary-file transfer succeeds.
- Output consumes payloads and releases blocks only after write_all completes.
  Early errors, panics, dropped replies and worker exits release remaining leases.
- Media ordering, registered source fingerprints, compression frames, resource
  limits, final integrity checks and sync/publication remain unchanged.
- No new public API, dependency, allocator, persistent cache or debug hook.
- Unit tests exercise shared exhaustion, concurrent producers, short writes,
  zero budget, spilling, cancellation and unwinding. Paired real exports cover
  all 29 inputs; full checks and Anki imports bind every measured artifact.
