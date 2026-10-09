# ADR 0026: Prepared publication and native work reduction

Decision date: 2026-10-03. Specification:
[publication performance](../superpowers/specs/2026-10-03-publication-performance-spec.md).
Discussion: [RFC 0008](../rfcs/0008-prepared-publication-and-native-work-reduction.md).

One core PreparedPublication owns a private inspected candidate, immutable
observations, destination and baseline locations/identity. It is not cloneable
and exposes no candidate path. Publication consumes it and enforces the prepared
policy; SDK close/drop releases unused candidates. Existing build uses this core;
compare still drops its candidate and returns observations only. Preparation
releases the Project and build workspace. Duration excludes review waiting.

SDKs adapt ownership rather than implement update policy. Node takes the owner
before scheduling, and Python before releasing the interpreter. Closed/consumed
errors are BUILD.PREPARED_UNAVAILABLE. Node protocol 6 and Python protocol 1 reject
older binaries even if package versions match. Python fork ownership is retained.
Publication rechecks baseline aliases, including moved original file identities,
and preserves atomic replacement and truthful late failure publication facts.

Node uses immutable shared Project snapshots and copy on write for authoring.
The first edit with a live task or clone may copy the project. Core Rust value
semantics and Python authoring stay unchanged. Bytes retain invocation snapshots.

Native writer calls omit staging manifest materialization while retaining all
validation, media preparation and embedded identity evidence. Internal Tools
Interface calls still materialize manifests; absent output has absent references.
Large Media.bytes checks its own limits/MIME, hashes outside the cache lock and
reuses live storage before temporary writes. Concurrent misses use the existing
registration arbitration. These paths no longer produce failures unique to the
omitted redundant I/O. Other structured errors remain unchanged.

Bundle 2.1.0 records the additive error/semantic contracts. Crate and SDK versions
remain 0.2.0 pending their separately governed release. APKG and existing identity,
report and comparison schemas remain unchanged. ADR 0023/0024 guarantees remain.
No ZIP streaming experiment or production performance switch is introduced.

Performance acceptance and platform gates are evidence requirements, not inferred
from this decision or from historical experimental timings. Package publication
still requires the existing Release Gate and Publication Approval.

The [local implementation evidence](../../benchmarks/results/20261003-publication-implementation/README.md)
records the functional and performance checks. Timing targets passed, but repeated
wide-field RSS increases exceed the original regression gate. This remains an
unreleased review candidate until that acceptance decision is resolved and the
remaining platform completion and release gates pass.
