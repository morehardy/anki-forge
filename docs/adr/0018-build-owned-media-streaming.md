# ADR 0018: Stream build-owned media into private candidates

Status: implemented and locally validated (2026-09-07)

## Decision

Default temporary Project/Deck builds own a private media archive. Registration
still validates the source immediately. During each build, one streamed source
read checks the registered fingerprint, computes SHA-1/BLAKE3 and MIME evidence,
and encodes zstd. Up to four workers claim tasks from a shared queue inside a
fixed window of twice the worker count plus one. Each job owns a capacity-one
reply channel; the consumer accepts results in canonical order. This balances
mixed file sizes while limiting all active and queued payloads together to nine
at four workers. Encoded payloads share a build-local pool of at most 72 fixed
64 KiB buffers (4.5 MiB). Buffers are allocated lazily and returned for reuse as
the consumer writes them directly into the candidate ZIP. A larger payload may
use otherwise idle capacity. If no buffer is available, that payload spills to
a private file without waiting for another job or the ordered consumer. The
transfer releases its buffers only after all held bytes have been written.
Errors, cancellation and unwinding also return outstanding buffers.

The 4.5 MiB limit includes active, queued and idle buffers, and fixed-size boxes
avoid geometric capacity growth. Pool metadata, codec contexts, allocator
overhead and the input model have separate memory costs. This bounds encoded
payload buffers, not total RSS or inline-input allocations. Thread failure
becomes a build diagnostic.

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

For packages reaching 256 KiB, an optional hashing thread consumes final bytes
in order while writing continues. One active block, one queued block and one
caller block bound additional payload buffers to 768 KiB. Smaller packages and
thread-creation failures use serial hashing. Completion joins the worker before
returning the fingerprint; dropping a failed writer also closes and joins it.
The final output flush, sync and publication requirements are unchanged.

The fixed schema11 upgrade-message collection is packaged as a generated asset;
a test regenerates and compares its exact bytes. Actual note databases are still
built per export. SHA-1 remains SHA-1, with RustCrypto 0.11's runtime hardware
selection and portable fallback; no platform FFI or global allocator is added.

Inspection reuses decoder workspace with a fresh session per frame; exact
window checks and cumulative ZIP/decoded budgets remain in force.
For at least 16 media entries, inspection may hash decoded payloads in ordered
256 KiB blocks on up to four scoped workers, including files larger than a block.
Archive reading, CRC/format checks
and all resource counters stay on the inspecting thread. Each worker has only
one queued block and one active block, and releases bytes before acknowledging
completion. Including the reader's current block, extra decoded payload buffers
are bounded by 2.25 MiB. Unavailable workers leave the serial path usable without
a payload buffer. Hash results are collected in the original media order,
and all workers finish before inspection returns, including on read failures.
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

The [streaming follow-up report](../../benchmarks/results/20260907-streaming-followup/implementation.md)
records chunked inspection, pipelined package hashing and bounded dynamic media
preparation, including the rejected SQLite-cache and identity-reuse experiments.

See the [current interface and benchmark report](../superpowers/specs/2026-09-07-internal-media-registration-report.md)
for individual-registration results and limits. The [earlier implementation report](../superpowers/specs/2026-09-07-media-performance-implementation-report.md)
retains the historical batch-registration evidence.
