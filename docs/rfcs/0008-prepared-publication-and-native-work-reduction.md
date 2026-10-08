# RFC 0008: Prepared publication and native work reduction

Status: implemented locally; verification status is recorded separately.
Decision: [ADR 0026](../adr/0026-prepared-publication-and-native-work-reduction.md).

The review boundary is a real inspected APKG owner. It cannot be reconstructed
from an observation, cloned, retargeted, or retried after a failed publish.
Blocked comparisons remain reviewable; changing allowances requires reprepare.
Destination anchoring and original/current baseline alias checks are mandatory.
Existing file-publication code retains responsibility for replacement, sync and
cross-device fallback. Error observations must survive both early and late failure.

Copy on write is confined to Node's native Project/task boundary. The first edit
while a version is shared pays the copy cost. Tests observe output fields, GUIDs,
decks and media rather than reference counts. Worker cleanup and Python fork
protection must extend to the new owner. Report serialization is unchanged.

The unused manifest and duplicate temporary bytes writes can be omitted only
after preserving their independent validation duties. Complete staging callers
retain canonical bytes and SHA-1. Cache hits still check the current import's
budget and MIME; registration reconciles concurrent misses without waiting by key.

Bundle 2.0.0 → 2.1.0 is additive_compatible: one new SDK state error and normative
semantics, no APKG/report/input schema change. Node protocol 5 → 6 and Python
protocol 1 require matching binaries. Version checks must reject missing protocol
metadata. Public performance claims require the specification's paired batches,
negative controls and independent RSS measurements; isolated gains are not added.
