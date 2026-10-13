# Task workflows for public Anki Forge 0.3.0

Read the user's language reference and only the relevant sections here. Linked
task guides illustrate Rust; use the matching [Node API](https://ankiforge.dev/docs/node-api/)
or [Python API](https://ankiforge.dev/docs/python-api/) for binding syntax.

## Basic and Cloze

- Add each note to `Project` under a stable source-record key. Basic has front
  and back; use `Note::basic` in Rust or `Note.basic` in Node/Python.
- Use `Note::cloze` / `Note.cloze` with `{{c1::answer}}` or
  `{{c1::answer::hint}}`. Each supported distinct deletion number generates a
  card; numbers are 1–500. One Basic plus a Cloze with c1 and c2 means two notes
  and three cards. Changing deletion numbers may change card identity.
- Ordinary strings are escaped Text, including Cloze syntax. Use `Content::html`
  / `Content.html` only for intentional markup. Built-in Cloze extra uses
  field key `back_extra`.

Read [Basic/Cloze](https://ankiforge.dev/docs/cards/) and
[identity concepts](https://ankiforge.dev/docs/concepts/).

## Media: images, audio, video, fonts and batches

- Import a real file or bytes with the correct MIME. A successful import owns
  an immutable snapshot; later source edits do not change it.
- Use typed `.image()` / `.sound()` content to retain media automatically.
  Sound content accepts audio/video; a raw filename string does not import bytes.
- When filenames matter, set `with_export_name` (Rust/Python) or `withExportName`
  (Node) before creating references. Use literal portable filenames without paths.
  Case or Unicode-normalization variants conflict; renamed clones do not alter
  earlier content nodes. MIME, identifiable bytes and export extensions must agree.
- For HTML/CSS/script/font references register explicit model assets or project
  `add_asset` / `addAsset`; arbitrary markup does not load its dependencies.
- Public batch import is `Media::files(paths)`, `await Media.files(paths)` or
  `Media.files(paths)`. It retains input order and applies limits per asset.
- Inspect media counts and names; test playback on the user's target clients.

Read [media](https://ankiforge.dev/docs/media/) and
[source media guide](https://github.com/morehardy/anki-forge/blob/main/docs/media.md).

## Custom note types, templates and CSS

- Complete `NoteType.builder` / `NoteType::builder` with `.build()` before
  creating notes through `model.note()`.
- Declare stable model, field and template keys separately from display names.
  Field assignments, template references and generation conditions use keys.
- Mark required/sort fields and choose generation conditions according to the
  matching language API; do not infer property names from Rust snippets.
- Register CSS/font/raw HTML assets explicitly with the model. For reuse, load
  the documented template-bundle-v2 format and its owned assets.
- Keep templates and CSS separate from user text. Review later schema changes
  through the update policy, even if their identity keys remain stable.

Read [custom models](https://ankiforge.dev/docs/custom-notetypes/) and
[template bundles](https://ankiforge.dev/docs/templates/).

## Image Occlusion

Input is a decodable owned image plus keyed rectangles `(key, x, y, width, height)`
in finite display pixels after EXIF orientation. Confirm dimensions and bounds;
width/height must be positive, mask keys unique and stable.

Rust/Python use `Note::image_occlusion` / `Note.image_occlusion`; Node uses
`Note.imageOcclusion`. Add masks with `Mask::rect` / `Mask.rect` and finish the
builder. Default HideAllGuessOne and optional HideOneGuessOne each create one
card per keyed mask; binding mode names differ, so consult the language API.
Set `header`, `back_extra` or `comments` on the completed note; `image` and
`occlusion` are reserved generated fields.

Inspect note/card/media counts and test rendering in Anki. Stable mask keys
preserve update ordinals, with an allocation limit of 1–500.
Read [Image Occlusion](https://ankiforge.dev/docs/image-occlusion/).

## Compare and update

Keep the previous **original distribution APKG**, its source revision, namespace
and all identity keys. An Anki re-export lacks the required identity evidence.
Build the candidate to a separate path so the archived baseline survives.

- Compare using `project.compare(CompareOptions::against(path))` in Rust,
  `await project.compare(CompareOptions.against(path))` in Node or the
  synchronous Python equivalent.
- A completed comparison can still block publication. Inspect findings and its
  policy decision; do not automatically allow every returned risk code.
- Build with `BuildOptions::to(next).update_from(previous)` (Rust),
  `BuildOptions.to(next).updateFrom(previous)` (Node), or
  `BuildOptions.to(next).update_from(previous)` (Python).
- Changing an answer under the same note key preserves logical identity.
  Missing baseline evidence and namespace mismatch are hard errors.
- Omitting notes is not deletion synchronization. Client import settings and
  newer learner edits still affect field updates; stable identity alone does
  not guarantee scheduling or every schema merge outcome.

Read [updates](https://ankiforge.dev/docs/updates/) and
[build guarantees](https://ankiforge.dev/docs/build-guarantees/).
