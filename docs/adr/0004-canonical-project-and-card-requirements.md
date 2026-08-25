# ADR 0004: Canonical Project Lowering and Persisted Card Requirements

## Context

The typed Rust Project and template-bundle paths previously converted custom
note types through the legacy Product model. That conversion discarded required
and sort field semantics. CardPlanner also evaluated `AnkiDefault` by rendering
the front template, while the writer independently stored an `any` requirement
over every field. The exported cards and the model Anki used for later edits
could therefore disagree.

## Decision

Builder-backed Projects lower through canonical ProductDocument v3 semantics.
Stable field keys, required and sort flags, identity, browser templates, target
decks, CSS, and generation rules cross that single boundary without a legacy
down-conversion.

For Product v3 normal custom note types, Rust compiles the front template into
one persisted `none`, `all`, or `any` card requirement. CardPlanner and the
writer consume that same requirement. If default front-side logic cannot be
represented without approximation, the build requires an explicit `all` or
`any` generation rule. Product v2 keeps its existing interpretation.

Template bundles may declare the same normal generation rules as Product v3.
Custom Cloze continues to derive card ordinals from its declared Cloze field.

## Consequences

- Rust Project, template bundle, and Product v3 inputs preserve equivalent field
  and template semantics.
- Optional fields are completed as empty values and required fields fail before
  artifact publication.
- Anki's stored model requirements match initial card planning for Product v3.
- Some complex `AnkiDefault` templates must opt into an explicit generation
  rule instead of receiving an unsafe approximation.
- Contract bundle 0.3.0 publishes the additive bundle schema and diagnostics.
