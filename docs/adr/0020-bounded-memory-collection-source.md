# ADR 0020: Bound the temporary collection source in memory

## Decision

Build the private SQLite source in memory when the estimated collection fits
within 64 MiB. The estimate adds field bytes and 2 KiB per note for ordinary
identity metadata, cards and indexes. It selects a storage strategy; it is not
a memory guarantee. In particular, wide models, many cards and unusually large
identities can exceed the estimate.

Enforce a separate limit of 8,192 source database pages at 8 KiB per page using
SQLite's `max_page_count`. If initialization reaches that limit, discard the
memory connection and rebuild once in a private file. Inputs whose estimate
already exceeds the budget start in a file. Other initialization errors retain
their existing failure behavior. Each file source has a unique name and is
removed after its connection closes, including during unwinding or compaction
failure.

Both strategies use the same schema, transactions and row population, followed
by `VACUUM INTO` a private file. Compression, actual APKG inspection, output
synchronization and publication retain their existing behavior. No public
option, format change, contract bundle update or global SQLite setting is added.

## Rationale and consequences

The source is disposable and does not need durable intermediate writes.
Removing those writes improves common text exports, with moderate extra memory
accepted for this optimization round. Limiting source pages keeps large exports
from retaining an entire additional database indefinitely. SQLite bookkeeping,
rollback journals, compaction and the caller's input have separate memory costs;
64 MiB is not a bound on total RSS.

A field-only estimate caused 50,000 short notes to fill the memory database and
then repeat population on disk. Including ordinary per-note overhead avoids
that measured regression. Unexpectedly large models can still require one
retry; correctness does not depend on the estimate being exact.

A temporary-file SQLite connection with a 32 MiB cache was also measured. It
used more peak memory than the memory source on the long-field workload, so it
was not selected.

## Validation

Tests compare exact compacted bytes for memory, forced capacity fallback and
direct file storage. They cover the page limit with unusually large identities,
non-capacity SQL failures, private-file cleanup, transaction rollback and a
successful retry after a failed public build. Existing native-path, inspection
budget, artifact-preservation and contract gates continue to apply.

Paired end-to-end timings, independent peak RSS samples and large-input checks
are recorded in the [implementation report](../superpowers/specs/2026-09-08-round3-build-performance.md).
