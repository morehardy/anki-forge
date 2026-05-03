---
asset_refs:
  - schema/inspect-report.schema.json
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

`Phase 5A` inspect output includes three additional structured observation
buckets beyond the existing core note/card/media data:

- `field_metadata` for field labels and role hints
- `browser_templates` for browser-specific template appearance declarations
- `template_target_decks` for template deck declarations with resolved deck ids

Deck routing observations expose `deck_name` on note and card reference entries.
For staging sources, note deck names come directly from normalized IR and card
deck names are computed as `template.target_deck_name ?? note.deck_name`. For
APKG sources, the original note-level import deck is not stored separately in
Anki's collection schema, so inspection reconstructs `notes[].deck_name` from
the first existing card deck, matching Anki's text export behavior.
