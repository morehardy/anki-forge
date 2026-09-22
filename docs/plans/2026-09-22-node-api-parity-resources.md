# Node parity resource measurements

Date: 2026-09-22. Host: macOS arm64, Node 24.19.0, Rust 1.92.0, core 0.1.0,
Node candidate 0.2.0 / binding protocol 2. These are local diagnostic observations,
not release benchmarks or cross-platform limits.

## Retained Rust state after Deck builds

The private probe copies the native adapter into a temporary crate, reuses the
repository's diagnostic System allocator counter and exposes allocation counters
only in that probe. Production native packages and the core contain no new
allocator or instrumentation. This measures Rust global-allocator requested live
bytes, excluding V8 and independent C allocators such as SQLite. It does not infer
leaks from RSS.

The controlled variant changes only BuildTask to the previous persistent
`context.project = Project::from(deck.clone())` dispatch. The direct variant
calls Deck.build(). Both otherwise use the same current source and workload:
1,000 notes with 8,192-byte Back fields, plus 64 registered 16 KiB SVG documents
and corresponding image notes. Three builds run consecutively, with artifact
owners closed and JS collection requested before each sample.

| Phase | Persistent Project control, live Rust bytes | Direct Deck, live Rust bytes |
| --- | ---: | ---: |
| Addon loaded | 42,903 | 42,903 |
| Authored | 10,535,481 | 10,535,481 |
| First build closed | 22,219,229 | 11,318,988 |
| Second build closed | 22,219,229 | 11,318,988 |
| Third build closed | 22,219,229 | 11,318,988 |
| Deck collected | 827,066 | 827,066 |

The direct path retains 10,900,241 fewer requested Rust bytes after this build.
Both variants stabilize across repeated builds and return to the same warmed
runtime allocation level after Deck collection. This comparison isolates the
removed persistent copy; it is not a timing claim against a released version.
Instrumented source hashes, allocator peaks and RSS samples are preserved in
[control](evidence/node-api-parity-2026-09-22/persistent-project-control.json) and
[direct](evidence/node-api-parity-2026-09-22/direct-deck.json).

Reproduce from the repository root (provide a Node 22.13+ executable):

```sh
python3 bindings/node/scripts/resource-probe.py --node /absolute/path/to/node --output /tmp/node-resource-evidence
```

## Finished-archive transfer

Each workload runs in a separate process, registers random WAV bytes via
addBuffer, and writes to a Writable with highWaterMark 1,024 that never retains
chunks. A sample after the first chunk separates completed Rust generation from
transfer. Builds and user media staging can still have their own memory costs.

| Input | APKG bytes | Writes | Largest chunk | ArrayBuffer bytes at transfer start after GC | Observed peak ArrayBuffer bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 MiB | 1,104,940 | 17 | 65,536 | 274,683 | 1,379,623 |
| 16 MiB | 16,833,941 | 257 | 65,536 | 274,683 | 8,270,075 |
| 64 MiB | 67,166,741 | 1,025 | 65,536 | 274,683 | 9,187,579 |

All targets remained open. The SDK did not allocate a whole-archive transfer
Buffer. Observed peaks include chunks awaiting GC, so they are not a hard heap
limit; SDK read/write buffer bounds are separately tested. A caller that retains
chunks can still consume memory proportional to the archive. Raw phase records:
[1 MiB](evidence/node-api-parity-2026-09-22/stream-1mib.json),
[16 MiB](evidence/node-api-parity-2026-09-22/stream-16mib.json),
[64 MiB](evidence/node-api-parity-2026-09-22/stream-64mib.json).

```sh
ANKI_FORGE_NATIVE_PATH=/absolute/path/to/anki-forge.node node --expose-gc bindings/node/scripts/profile-resources.mjs stream 64
```
