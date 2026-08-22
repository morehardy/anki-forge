---
asset_refs:
  - schema/product-document-v3.schema.json
  - schema/template-bundle.schema.json
  - schema/diagnostic-item.schema.json
---

# Template and Card Planning Semantics

Rust is the semantic authority. ProductDocument lowering, the Rust Project API,
Python, Node, and the CLI must converge on `product-build`; language bindings may
validate basic argument types but must not implement a second template or Cloze
engine.

## Supported template syntax

The v1 compiler recognizes Anki-style `{{...}}` expressions:

- field replacement and filter chains;
- `#` conditional and `^` inverted sections with matching `/` close sections;
- comments beginning with `!`;
- filters `cloze`, `hint`, `text`, and `type`;
- special fields `Card`, `CardFlag`, `Deck`, `FrontSide`, `Subdeck`, `Tags`,
  and `Type`.

Unknown fields, unmatched delimiters, and mismatched sections are errors.
Unknown syntactically valid filters are portability warnings. HTML, CSS,
JavaScript, third-party filter execution, and browser rendering correctness are
outside the compiler contract.

## Custom Cloze

A custom Cloze note type declares exactly one stable field key and exactly one
template. Its front template must apply `cloze` to the declared field. Card
ordinals come from complete `{{cN::body}}` deletions in that field:

- `N` is a positive decimal integer;
- repeated `N` values produce one card;
- distinct values are sorted and map from `cN` to zero-based card ordinal
  `N - 1`;
- no valid deletion is an error;
- zero, missing delimiters, missing close braces, empty bodies, and nested
  numbered deletions are malformed errors.

The same CardPlanner supplies writer materialization, BuildReport card counts,
and inspect observations.

## Normal card requirements

ProductDocument v3 and typed Project normal templates compile front-side card
generation into the same requirement stored in the Anki NoteType:

- a statically visible front uses `none` (no non-empty field is required);
- alternatives consisting of direct field replacements use `any`;
- a single positive conditional path uses `all`;
- an explicit `all` or `any` generation rule overrides default inference.

If the rendered-front predicate cannot be represented by one Anki `none`,
`all`, or `any` requirement, the build reports
`TEMPLATE.GENERATION_RULE_REQUIRED`. It does not persist an approximation that
would change card creation after import. Product v2 retains its existing normal
custom-note-type interpretation.

## External template bundle

`template-bundle-v1` is a directory containing `anki-template.yaml`. All
referenced paths are relative to that directory. Absolute paths, parent
traversal, symlink escape, non-regular files, oversized files, and invalid UTF-8
template text are rejected. Note type and media changes are committed only after
the entire bundle has loaded and validated.

Stable note type, field, and template keys are explicit; display names, file
names, and declaration order are not identity substitutes. Product v2 remains
normal-only. Product v3 is required for custom Cloze semantics.

Normal bundle templates may declare `generation_rule` with kind
`anki_default`, `all`, or `any`; `all` and `any` name one or more stable field
keys. Custom Cloze bundles must not declare a normal generation rule. A bundle
field cannot be both `required` and `optional`, and at most one field may be the
Anki sort field.
