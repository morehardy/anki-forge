# Build and output guarantees

`Project::build` returns `BuildOutput` only after a successful operation, with a
guaranteed APKG artifact. A failed operation returns `BuildError` with structured
kind/code, observations, underlying causes and actual publication facts.

## Temporary and persistent artifacts

Choose the ownership model explicitly:

| Request | Ownership |
| --- | --- |
| `BuildOptions::to(path)` | Persistent destination; survives handle cleanup |
| `BuildOptions::temporary()` | Temporary file; deleted with its last artifact owner |
| `artifact.persist_to(path)` | New persistent owner; original owner remains usable |

Keep the `BuildOutput` or a cloned artifact alive while reading a temporary
path. A report or JSON snapshot does not extend file lifetime. Rust drops owners
normally; Node exposes `artifact.close()` for deterministic release. Copying the
path into application state does not create an owner.

`BuildReport::snapshot()` contains only observations. `BuildOutput::snapshot()`
and `BuildError::snapshot()` additionally state the real outcome. A warning can
appear in a successful snapshot. Do not infer outcome from diagnostic severity,
a path's existence, or whether a report has comparison findings.

## Baselines and publication

For updates, retain the previous original distribution APKG as an immutable
baseline and choose a separate output destination. Baseline evidence and
candidate content are checked before publication. Policy rejection does not
replace the output. Missing or corrupt evidence and exhausted inspection budgets
are hard failures.

Publication uses atomic replacement of the APKG. This is a per-file guarantee;
writing an application's separate JSON report is a separate operation. The
ordinary build does not write an additional identity file: complete versioned
evidence is embedded in the distribution package.

A late error can occur after publication, for example while confirming
persistence. Error snapshots therefore record `PublicationSnapshot` facts:
`path`, `stage` (`not_published` or `published`), `temporary`, and durability
(`confirmed` or `unconfirmed`). Check these facts before retrying or announcing
that no file was written. `PersistError` also preserves its publication fact and
actual I/O source. Atomic replacement alone does not establish durability after
a crash.

Snapshot serialization preserves native paths without accessing the filesystem.
Unicode paths remain JSON strings. Non-UTF-8 Unix paths use
`{"encoding":"unix_bytes","bytes":[...]}`; Windows paths with unpaired UTF-16
surrogates use `{"encoding":"windows_wide","units":[...]}`. The arrays contain
the exact native bytes or code units. Success artifact paths, failure publication
paths, and binding error-path details all use this representation. These values
describe paths and do not retain artifact ownership. A filesystem may reject a
particular native path, but its publication failure facts remain serializable.

## Inspection limits

Finite archive, entry-count, expansion, decoded collection/media and zstd-window
budgets apply independently to baseline and candidate. Start with
`InspectLimits::default()` and deliberately raise a relevant limit when needed.
A limit of zero is not unlimited. These counters are not a process-wide memory
or CPU sandbox; media imports have a separate earlier per-asset budget.

## Concurrent work

Rust builds borrow immutable project state. Owned models and media can be shared
across projects. The Node binding captures a project snapshot when build or
compare is invoked, so later additions do not change that request. Retain the
artifact owner until all readers have finished; closing the last temporary owner
while a separate consumer still needs its path is an application lifetime error.

Package inspection cannot establish every client-side behavior. Test rendering,
playback and update imports with the intended Anki versions and settings. See
[updates](updates.md) for scheduling and schema-change limits.
