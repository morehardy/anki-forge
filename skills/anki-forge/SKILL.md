---
name: anki-forge
description: Build and maintain Anki decks with the public Anki Forge 0.3.1 Rust, Node/TypeScript, or Python APIs. Use when a user mentions Anki Forge or ankiforge, or asks to use this library for APKG generation, Basic/Cloze cards, media packaging, custom note types, Image Occlusion, stable note IDs, or deck updates.
---

# Anki Forge

Use the published Anki Forge 0.3.1 package to create runnable deck-building
applications. This Skill provides guidance; it does not call an AI service.

## Start with the application's language

Inspect its manifest and existing dependency before editing. Select one reference:

- Rust: [references/rust.md](references/rust.md).
- Node / TypeScript: [references/node.md](references/node.md).
- Python: [references/python.md](references/python.md).

Default to a complete persistent `.apkg` example. State the files to add or edit,
exact setup and run commands, output path, and expected contents. Proceed with
work the user has requested; ask focused questions only when language, input
shape, output behavior, or stable identity cannot be inferred.

## Public API and identity rules

- Use public APIs documented for 0.3.1. Do not invent methods or silently switch
  to git/path dependencies, internal tools, source-only APIs, or unreleased features.
  If an existing dependency uses another version, explain the mismatch before
  choosing a documented migration or a version-specific reference.
- Keep the project namespace and source-record note keys stable. Do not derive
  identity from mutable answers, random values, or a record's current row number.
- Keep model, field, template and mask keys stable; distinguish keys from display names.
- Preserve media export filenames and MIME declarations. Raw HTML/CSS assets
  need explicit registration; a filename string does not import a file.
- Preserve the previous original distribution APKG as an update baseline.
  An Anki re-export does not retain the required identity evidence.
- Do not publish, upload, delete decks, or modify an Anki collection unless the
  user explicitly requests that action. Building a local APKG is the normal task.

## Load only the relevant task guidance

Read [references/workflows.md](references/workflows.md) for the relevant section:

- Basic or Cloze: text escaping, deletion numbers, counts and stable keys.
- Images, audio, video or fonts: owned snapshots, MIME, export names and batch import.
- Custom fields/templates/CSS: completed models, field keys and explicit assets.
- Image Occlusion: image input, pixel rectangles, keyed masks and output counts.
- Revised decks: original APKG baseline, comparison policy and update build.

For installation, native loading or authoring errors, read
[references/troubleshooting.md](references/troubleshooting.md).
Consult the matching language API linked there before translating Rust method
names into another language.

## Verify the result

Run the smallest relevant formatter, type check, build or example command.
Inspect the resulting APKG when the project provides an inspection command.
Report the path and note/card/media counts when available. A successful export
does not establish client playback, preserved scheduling or every import outcome;
use the documented client limits for the user's intended scenario.
