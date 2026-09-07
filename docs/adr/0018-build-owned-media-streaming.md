# ADR 0018: Stream build-owned media into private candidates

Status: implemented and locally validated (2026-09-07)

## Decision

Default temporary Project/Deck builds own a private media archive. Registration
still validates the source immediately. During each build, one streamed source
read checks the registered fingerprint, computes SHA-1/BLAKE3 and MIME evidence,
and encodes zstd. Four workers at most feed deterministic, capacity-one queues.
Each encoded payload keeps at most 512 KiB of data before spilling to a private
file. The consumer writes final ZIP media entries immediately; codec contexts,
allocator capacity and the input model have separate memory overhead. This is a
bound on in-flight encoded payloads, not a total RSS limit or an inline-input
allocation limit. Thread failure becomes a build diagnostic.

A logical normalized/staging manifest is still produced. Temporary staging does
not need physical media copies because the build owns its verified encoded bytes.
Explicit `artifacts_dir`, `media_store_dir`, `base_dir` and self-contained mode
retain the existing physical CAS/staging flow and integrity checks. The standalone
Phase 3 writer's CAS contract is unchanged. Callers cannot provide a prepared
archive or use it to bypass verification of external CAS files.

The private STORED ZIP writer knows entry lengths and CRCs before writing headers,
hashes final bytes without seeking, and supports ZIP64. Default media packages
write `meta`, numbered media entries, collection entries, then the media map.
Persistent builds retain their existing order. Container hashes can consequently
differ across these storage modes; decoded entries, Anki identities and content
remain equal. Repeated builds in the same mode are deterministic.

The fixed schema11 upgrade-message collection is packaged as a generated asset;
a test regenerates and compares its exact bytes. Actual note databases are still
built per export. SHA-1 remains SHA-1, with RustCrypto 0.11's runtime hardware
selection and portable fallback; no platform FFI or global allocator is added.

Inspection reuses decoder workspace with a fresh session per frame; exact
window checks and cumulative ZIP/decoded budgets remain in force.
Inspection and policy acceptance remain mandatory before publication. The owned
candidate is synced and renamed, with copy-before-replace on cross-device moves.
`persist_to` keeps its copy semantics because its source may have other owners.
All private candidates and spills are removed on failure or ownership release.

## Public API

Keep `deck.media().add(...)` and Project's `add_file(...).export_as(...)` unchanged.
Bounded parallel preparation belongs to the private build pipeline. The initially
added, unreleased `add_many` interface has been removed; benchmarks now measure
the ordinary individual registration loop.

Registration remains synchronous: its full content fingerprint is required for
immediate duplicate-content checks, persisted Deck evidence and later source-change
detection. Queuing hashes until export would change these behaviors. Deck file
registration instead uses 64 KiB reads and extracts dimensions from the first
hashed block when possible. A streaming parser on the same file handle retains
support for long image headers. No background registration work outlives `add`.

Historical batch-registration performance is not a claim for this interface.

## Validation

The implementation report records same-input timings, separate RSS samples,
source/binary identities, exact media and raw collection verification, and pinned
upstream Anki import/render evidence. Successful-path equality alone does not
replace source-change, ownership, overflow/spill, failure and concurrency tests.

See the [current interface and benchmark report](../superpowers/specs/2026-09-07-internal-media-registration-report.md)
for individual-registration results and limits. The [earlier implementation report](../superpowers/specs/2026-09-07-media-performance-implementation-report.md)
retains the historical batch-registration evidence.
