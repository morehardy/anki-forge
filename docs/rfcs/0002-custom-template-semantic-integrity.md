# RFC 0002: Custom Template Semantic Integrity

Status: implemented for review
Related: ADR 0003, ADR 0004, GitHub issue #27

## Proposal

Complete the custom-template capability introduced in ProductDocument v3 and
template-bundle-v1 by making all high-level inputs converge before lowering.
The canonical mapping preserves field presence rules, sort selection, stable
identity, complete template appearance, target decks, and generation rules.

Compile each Product v3 normal front template to the Anki card requirement that
is persisted with the NoteType. Static visible fronts require no non-empty
field; direct field alternatives compile to `any`; a single positive conditional
path compiles to `all`. Logic that cannot be represented by one Anki requirement
fails with a stable diagnostic until the author supplies an explicit rule.

Add optional `generation_rule` support to normal template bundles. Reject
conflicting required/optional fields and generation rules attached to custom
Cloze bundles before mutating the Project.

## Compatibility

ProductDocument v2 retains its existing normal custom-note-type interpretation.
The schema addition is backward compatible for existing template bundles, while
the Product v3 failure for unrepresentable default generation logic is an
intentional correctness boundary. The public contract bundle version advances
from 0.2.0 to 0.3.0; `bundle_version` remains the only public compatibility axis.

## Validation and rollout

The main automated acceptance seam remains `product-build` through APKG inspect.
Equivalent Rust Project, template bundle, and Product v3 fixtures must preserve
the same observable field, template, card, media, and requirement facts.

Release evidence also includes contract verify/summary/package and documented
Anki Desktop imports for custom normal and custom Cloze bundles. Desktop checks
include creating and editing notes after import so persisted card requirements,
not only initial exported rows, are exercised.
