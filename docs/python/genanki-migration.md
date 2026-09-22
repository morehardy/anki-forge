# Genanki Migration Guide

anki-forge's Python API is concept-oriented for genanki users. It does not clone genanki classes or model ids. Build a `Project`, add stock or custom notes, then call `write_apkg()`.

## Basic Notes

`Note.basic(front="<b>hi</b>", back="plain")` treats both fields as safe text and escapes the tags. Use explicit HTML fields when you want markup:

```python
Note("basic").html("front", "<b>hi</b>").text("back", "plain")
```

## Cloze Notes

`Note.cloze(text="{{c1::<b>term</b>}}", back_extra="<i>hint</i>")` treats `text` as HTML so cloze markers and markup survive. `back_extra` is safe text and is escaped. If you interpolate untrusted text into the cloze body, escape it before building the cloze string.

## Custom Models

Use `NoteType.custom(id)`, `Field(name)`, and `Template(name, front="{{Expression}}", back="{{Meaning}}")` instead of genanki model ids. Template front/back strings reference Anki display field names such as `{{Expression}}`; generation rules and identity recipes reference stable field keys such as `expr`.

## Media

`project.media.add_file(path, export_as="sound.mp3")` reads and fingerprints the file immediately. Missing or unreadable files raise `MediaError` during registration; build verifies the source again. `add_bytes(source_label="diagram.png", data=data, export_as="diagram.png")` is the immediate in-memory option when you already have bytes.

Project `add_bytes` accepts non-empty payloads up to 64 KiB. Keep source files available until build completes. Inputs are snapshotted at addition; use a new Project with the same stable IDs to produce updates. See [Python 0.1 to 0.2 migration](../../bindings/python/MIGRATION.md) for existing anki-forge projects.
