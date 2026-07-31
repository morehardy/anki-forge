# RFC 0001: ProductDocument v3 and Template Bundle v1

Status: implemented for review  
Related: ADR 0003, GitHub issue #27

## Proposal

Add `product-v3` as the canonical transport for custom note types with explicit
`note_type_kind`, `cloze_field`, stable field/template keys, browser templates,
target deck, CSS, identity, notes, and media. Keep `product-v2` accepted with
unchanged normal semantics.

Add `template-bundle-v1`, a directory rooted at `anki-template.yaml`, for
importing one note type plus CSS, browser templates, target deck, and local
assets. All reads are bounded to the canonical bundle root and committed to a
Project atomically.

## Compatibility

This is additive for Product v2 consumers and increments the single public
`bundle_version` axis from 0.1.1 to 0.2.0. Custom Cloze requires explicit
Product v3 opt-in. Stable diagnostic codes define automation behavior; messages
are descriptive and may evolve.

## Validation and rollout

The manifest publishes both schemas, template semantics, registry additions,
and shared fixtures. Release evidence is:

```text
contract_tools verify --manifest contracts/manifest.yaml
contract_tools summary --manifest contracts/manifest.yaml
contract_tools package --manifest contracts/manifest.yaml --out-dir <temporary-directory>
```

Anki Desktop remains the import/render oracle for release validation.
