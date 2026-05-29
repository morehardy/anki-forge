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

`project.media.add_file(path, export_as="sound.mp3")` registers a file path but does not check that the file exists or is readable. Missing or unreadable files are reported by Rust build diagnostics. `add_bytes(source_label="diagram.png", data=data, export_as="diagram.png")` is the immediate in-memory option when you already have bytes.
