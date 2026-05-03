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

Normalization preserves `notes[].deck_name` independently from
`notetypes[].templates[].target_deck_name`. The note deck represents the deck
selected by authoring/import input for new cards from that note, while the
template target deck represents Anki's per-template Deck Override. Normalization
must not copy a note deck into template target deck fields or copy a template
target deck into note deck fields.

This explicit-lowered bridge allows upstream product authoring to preserve
stock-compatible payloads and custom `normal` notetype declarations without
inventing a separate downstream kind taxonomy.

Authoring notes may reference media entries inline, and normalized output keeps
those media records inline as well for this contract scope. The media source
module shows that Anki normalizes filenames during filesystem insertion, but
this contract does not model the filesystem rewrite or uniquification process.
It only preserves the declared media records as part of the normalized payload.

Normalization must not invent unsupported stock templates, unsupported field
names, or final media storage names that are not backed by the local source.
