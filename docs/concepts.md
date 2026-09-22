# Core concepts

Anki Forge builds `.apkg` packages from notes, note types, templates and media.
A package can contain multiple decks. Anki imports the package and manages study scheduling.

## Notes and cards

| Term | Meaning | Example |
| --- | --- | --- |
| Note | One item of learning content, stored in fields | A vocabulary entry with “hola” and “hello” |
| Note type | Declares fields and templates for a kind of note | Basic, Cloze, or your custom vocabulary type |
| Field | A named value on a note | Expression, Meaning, Audio |
| Template | Defines the front and back of a card | Show Expression, then Meaning |
| Card | One reviewable question generated from a note | A forward vocabulary question |
| Deck | A destination that groups cards in Anki | `Spanish::Vocabulary` |
| Project | The authoring container for notes, note types and media | A source dataset exported into one or more decks |

A Basic note normally produces one card. A Cloze note with `c1` and `c2`
produces two cards. Repeating `c1` hides those occurrences together on one card.
A custom normal note type can have multiple templates, each with its own generation rule.

## Choose Deck or Project

| Start with | Use it for | Next step |
| --- | --- | --- |
| `Deck` | A short Basic, Cloze or Image Occlusion workflow | [First Rust application](installation.md) |
| `Project` | Custom fields/templates, explicit text and HTML, validation and update configuration | [Authoring guide](rust-guide.md) |

Rust uses `Project::from(deck)` to move a Deck into a Project. Python provides
`Project.from_deck(deck)` as a snapshot. Node exposes separate Deck and Project
constructors; use its documented authoring methods. The language API guides
describe ownership and concurrency details.

## Field keys and display names

A custom field can have the stable key `expr` and display name `Expression`.
Use the key in authoring code and identity/generation rules. Use the display name
in template HTML: `{{Expression}}`. A template also has a stable key and a display name.

Keep explicit field and template keys across releases. Custom derived identity
can also depend on selected field display names; a stable key alone does not
make every rename identity-preserving. Use explicit note IDs when your source
already supplies durable keys, and compare proposed changes against a baseline.

## Text and HTML

`Note::basic(...)` and `.text(...)` on the Rust Project API escape text.
`.html(...)` preserves HTML. `Note::cloze(...)` preserves HTML and cloze markers;
its extra field is text. Node/Python Project conveniences follow the same core behavior.

The Rust **Deck** Basic/Cloze lanes accept HTML content, as do the bindings' Deck
conveniences. Choose Project text setters when you need literal text escaping.
These interfaces are not interchangeable escape policies.

## Stable identity and release history

Give each logical note a stable ID, such as `es:hola`. Keep it when correcting
that note's wording; use a different ID for a new note. Keep the project's stable
ID across releases as well. Do not put explicit IDs in the reserved `afid:v1:*` namespace.

Identity says which note is being updated. A previous APKG or maintained identity
lockfile provides revision evidence for the update. A plain first export does
not guarantee later imports will update existing content. Follow the
[complete update workflow](updates.md) and verify Anki's import behavior separately.
