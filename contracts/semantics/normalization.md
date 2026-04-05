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
`stock.rs` source. In normalized output:

- `basic` resolves to the stock Basic notetype with `Front` and `Back` fields
  and the standard single-card template shape.
- `cloze` resolves to the stock Cloze notetype with `Text` and `Back Extra`
  fields and a cloze question format derived from the `Text` field.
- `image_occlusion` resolves to the stock image occlusion notetype with the
  source-defined occlusion, image, header, back-extra, and comments fields, and
  it carries the image-occlusion CSS from the source module.

Authoring notes may reference media entries inline, and normalized output keeps
those media records inline as well for this contract scope. The media source
module shows that Anki normalizes filenames during filesystem insertion, but
this contract does not model the filesystem rewrite or uniquification process.
It only preserves the declared media records as part of the normalized payload.

Normalization must not invent unsupported stock templates, unsupported field
names, or final media storage names that are not backed by the local source.
