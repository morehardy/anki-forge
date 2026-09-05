---
asset_refs:
  - schema/inspect-report.schema.json
  - errors/error-registry.yaml
---

# Inspect

Inspection reports are stable observation models. They summarize what was
observed from staging or packaged output without collapsing into a raw byte
dump.

The report boundary includes the observation model version, source identity,
fingerprint, observation completeness, missing domains, degradation reasons,
and the structured observation buckets required by the schema.

Inspection must preserve compatibility-relevant structure and avoid packaging
noise that does not help compare writer outputs.

Bundle 0.5.0 emits observation model `phase3-inspect-v2` for the numeric model-ID
and full-content revision evidence below. Saved `phase3-inspect-v1` reports remain
readable, but comparing v1 and v2 reports is partial and records that observation
model versions differ. Node and Python bindings accept both supported versions,
including mixed-version diff reports, and continue rejecting unknown versions.

Each notetype observation includes its numeric `anki_model_id`. APKG inspection
reads it from the collection, while staging inspection reads the selected model
assignment. It is not inferred from declaration order for new staging artifacts.

Each note reference observation also carries full-content `revision` evidence:
the versioned content digest and effective `mtime_secs`. Inspection recomputes the
digest and reads actual APKG modification times; embedded identity metadata does
not override note storage. Staging uses its selected time, or the legacy default
`1` if no explicit note time exists.

Field storage preserves empty values, including the single-field case where
`notes.flds` is an empty string. It represents one empty field value, not an
absent field map, and contributes the declared field name/value to the digest.

`Phase 5A` inspect output includes three additional structured observation
buckets beyond the existing core note/card/media data:

- `field_metadata` for field labels and role hints
- `browser_templates` for browser-specific template appearance declarations
- `template_target_decks` for template deck declarations with resolved deck ids

Deck routing observations expose `deck_name` on note and card reference entries.
For staging sources, note deck names follow normalized IR and card deck names
are computed as `template.target_deck_name ?? note.deck_name`. These names and
template target names are then expressed using the writer's canonical human
deck names, including component normalization and the selected spelling of
case-insensitive aliases and their parents. APKG inspection converts the native
`U+001F` hierarchy separator back to human-readable `::` and preserves the
spelling stored in the collection.

For APKG sources, the original note-level import deck is not stored separately in
Anki's collection schema, so inspection reconstructs `notes[].deck_name` from
the first existing card deck, matching Anki's text export behavior. This APKG
note deck is an observational reconstruction, not authoritative source recovery:
it may differ from the original note deck when the first card was routed by a
template target deck override or when a note's cards live in multiple decks.

## APKG resource budgets

APKG inspection is a bounded read of untrusted archive content. Every entry point
uses finite defaults, including runtime/CLI inspection, build-result inspection,
comparison, and previous-APKG identity recovery. A Rust caller may explicitly
override them with `InspectLimits`, `BuildOptions::inspect_limits`, or
`Project::diff_against_apkg_with_limits`; there is no automatic unlimited retry.

| Resource | Default maximum |
| --- | --- |
| Input archive bytes | 2 GiB |
| ZIP entries / media-map entries (each independently) | 100,000 |
| Central directory bytes / ZIP64 extended footer (each independently) | 16 MiB |
| Output of one ZIP entry | 1 GiB |
| Cumulative ZIP output | 4 GiB |
| Decoded `meta` | 64 KiB |
| Decoded `media` map | 16 MiB |
| Decoded collection database | 512 MiB |
| Decoded individual media payload | 256 MiB |
| Cumulative decoded output | 4 GiB |
| zstd frame window | 64 MiB |

The ZIP index is bounded before allocating the library's index. Both ordinary
ZIP and ZIP64 counts/extents are checked. Inspection accepts single-disk,
zero-offset archives with contiguous directory/footer metadata ending at EOF;
ambiguous footer signatures in index metadata, trailing bytes, and prefixed
self-extracting layouts are rejected. Payload bytes cannot supply fallback ZIP
footers during indexing. Metadata is snapshotted from the same open file handle.

Advertised ZIP sizes are early checks, not the accounting authority: actual ZIP
output and final decoded output have separate cumulative counters. The latter
includes `meta`, the media map, collection, and media payloads; the former also
includes encoded zstd headers and skippable frames. Unreferenced payloads are not
decoded. A high compression ratio alone is not a reason for rejection.

All concatenated zstd frames share entry and cumulative budgets. Each frame's
window is checked before decoder allocation, including single-segment frames.
Limits are inclusive; streaming may probe one extra byte to identify an excess
but never writes that excess to a collection file or media hash. Small metadata
buffers are bounded, media hashes are streamed, and the collection is streamed
to a private temporary file opened by SQLite read-only and removed on return.
Media-map entry counts are checked before allocating repeated parsed entries.

Exceeding any resource limit terminates that inspection with
`INSPECT.RESOURCE_LIMIT_EXCEEDED`. The typed error identifies the resource, entry
when applicable, limit, and first observed excess (not the unknown complete
size). It must not become ordinary missing-media degradation or a successful
partial inspect report. Ordinary malformed/missing-media behavior is unchanged.

Build reports expose this diagnostic in domain `inspection`, stage `inspect`.
A strict baseline failure blocks the build before publishing. Report-only or
disabled update safety can continue building, but a failed baseline remains
unavailable, with no previous inspection or complete diff. Current-artifact
inspection exceeding its budget prevents publication in every mode. Budgets
are per inspection, not shared between separate current/baseline inspections.

These limits bound archive expansion and decoder windows, not total process
memory, CPU time, SQLite query work, or the size of reconstructed observation
objects. Process isolation/timeouts and further SQLite/model quotas remain
separate hardening work for hostile multi-tenant services. Staging inspection
does not decompress APKG input and is outside this policy.
