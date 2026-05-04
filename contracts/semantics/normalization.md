---
asset_refs:
  - schema/authoring-ir.schema.json
  - schema/normalized-ir.schema.json
  - schema/normalization-result.schema.json
---

# Normalization Semantics

Normalization turns writer-authoring input into a writer-ready normalized
artifact. This contract only describes the stock lanes that are grounded in the
local source anchors listed below, and it stays away from packaging or storage
details that are not modeled by the schemas.

Source anchors:

- `docs/source/rslib/src/notetype/stock.rs`
- `docs/source/rslib/src/image_occlusion/notetype.rs`
- `docs/source/rslib/src/media/files.rs`

The stock notetype lanes are limited to the variants exposed in the local
`stock.rs` source. In normalized output, downstream/native-facing kinds stay
aligned with Anki's real model and only expose `normal` and `cloze`:

- stock Basic resolves to `kind = "normal"` with `original_stock_kind =
  "basic"`, the stock `Front` and `Back` fields, and the standard single-card
  template shape.
- stock Cloze resolves to `kind = "cloze"` with `original_stock_kind =
  "cloze"`, the stock `Text` and `Back Extra` fields, and the standard cloze
  question format derived from the `Text` field.
- stock Image Occlusion resolves to `kind = "cloze"` with
  `original_stock_kind = "image_occlusion"`, the source-defined occlusion,
  image, header, back-extra, and comments fields, and the image-occlusion CSS
  from the source module.

When authoring input already includes explicit lowered notetype payloads
(`fields`, `templates`, optional `css`, and optional `field_metadata`),
normalization preserves that lowered shape instead of re-expanding the stock
lane. The normalized output carries through:

- lowered notetype identity fields such as `kind`, `original_stock_kind`, and
  `original_id`
- field ordinals and config metadata such as `ord`, `config_id`, `tag`, and
  `prevent_deletion`
- template ordinals and config metadata such as `ord`, `config_id`,
  `browser_question_format`, `browser_answer_format`, `target_deck_name`,
  `browser_font_name`, and `browser_font_size`
- `field_metadata` entries including `field_name`, `label`, and `role_hint`

This explicit-lowered bridge allows upstream product authoring to preserve
stock-compatible payloads and custom `normal` notetype declarations without
inventing a separate downstream kind taxonomy.

Authoring media declarations may reference relative local paths or small inline
byte payloads. Path sources are resolved against explicit normalization options,
not the process working directory. Inline byte payloads are authoring-only and
must be ingested into the media store before normalized IR is produced.

Normalized media is represented by `media_objects`, `media_bindings`, and
`media_references`. It must not contain `data_base64` or inline byte payloads.
`media_objects` describe CAS-backed content, `media_bindings` describe APKG
export filenames, and `media_references` describe resolved, missing, or skipped
static references discovered in notes/templates.

CAS ingest writes original bytes to a unique temporary file, computes BLAKE3,
SHA-1, size, and MIME sample while streaming, then atomically persists with
no-clobber semantics. If the final object already exists, normalize verifies the
existing bytes and discards the temporary file.

Normalization owns semantic media diagnostics. It emits default media errors for
unsafe desired/export filenames, unsafe source paths, missing or unreadable path
sources, non-regular source files, invalid inline base64, inline payloads above
policy size, CAS write or verification failures, duplicate media ids, duplicate
or conflicting export filenames, unsafe static references, missing static local
references, high-confidence declared MIME mismatches, and object or total media
size limits. Policy-controlled media diagnostics cover unused bindings and
unknown MIME results. Informational media diagnostics may report deduped objects
when multiple bindings resolve to the same CAS object.

Media diagnostic reports should identify the relevant object, binding, or
reference location where possible, including owner/location selectors for
references and stage/operation context for ingest, MIME, CAS, and policy
failures. External URLs, `data:` URIs, protocol-relative URLs, and dynamically
computed references are skipped or recorded as skipped references rather than
reported as missing media.

Normalization must not invent unsupported stock templates, unsupported field
names, or final media storage names that are not backed by the local source.
