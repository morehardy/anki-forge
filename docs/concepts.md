# Core concepts

## Project and note

A `Project` is one publication with a stable namespace and a human-readable
name. It can place notes in several Anki decks: `default_deck` supplies the
fallback and `Note::deck` overrides it. A deck is a destination, not a second
container for authoring, media or builds.

A `Note` carries its model and content. `project.add(key, note)` assigns a
required stable source key and atomically collects dependencies. Duplicate keys,
unknown fields, conflicting models or media fail without partially adding state.
The project namespace and record keys must survive edits and source reordering.
Display names and text are not identity keys.

## Models, fields and templates

`NoteType::builder(key)` accumulates declarations; `.build()` validates and
returns a shareable immutable model. `model.note()` creates a note holding it.
Built-in Basic, Cloze and Image Occlusion notes carry their own models.

Fields and templates have stable keys and optional display names. Author
templates with field keys, for example `{{front}}`, even when the Anki display
name is `正面`. Completion validates and compiles references, including sections,
filters, Cloze and browser templates. Assignments also use keys, never a guessed
key or display-name fallback.

## Text and HTML

Ordinary strings always mean Text, across Basic, Cloze and custom notes. Text
escapes HTML; `Content::html` intentionally includes markup. Cloze syntax such as
`{{c1::answer}}` is preserved in Text without making embedded HTML trusted.
`Content::sequence` combines text, HTML, images and sounds into one field.

Typed image and sound content retain media dependencies until export. Do not
render media to strings before adding it unless you also declare the asset
explicitly for your hand-written HTML/CSS/script.

## Media ownership

`Media` is an owned snapshot with a deterministic content-derived filename.
Import succeeds only after reading and validating the input under its budget.
The source file may then change or disappear. Clones share storage and can be
used in multiple projects. `with_export_name` selects a fixed portable filename
without changing existing clones or content nodes.

Typed references collect media automatically. Explicit assets on a model or
project are included even if static analysis cannot identify their use. The
library does not discover file contents from arbitrary HTML or scripts.

## Build and update

A successful build returns an output with a guaranteed APKG. A report contains
observations; a snapshot is JSON data and owns no files. Temporary artifacts
remain alive only while at least one artifact owner remains.

An update uses complete evidence embedded in the previous original distribution
APKG. Independent comparison separates findings from a publication policy.
High-risk findings block updates by default; hard errors cannot be allowed.
Anki import settings, local edits and card scheduling still require client
validation. See [updates](updates.md) and [build guarantees](build-guarantees.md).
