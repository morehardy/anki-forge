# Python Quick Start

```python
from anki_forge import Note, Project

project = Project("Example")
project.add_note(Note.basic("Front", "Back"))
report = project.write_apkg("example.apkg")
report.ensure_success()
```

`Project`, `NoteType`, `Note`, and `MediaRegistry` are mutable builders and are not thread-safe. Create one project per concurrent task or synchronize mutations yourself.

## Long-Term Projects

Use `Project("Language", stable_id="language-core", default_deck="Language::Core")` for decks that will be rebuilt over time. Stable project and note ids keep Anki identity stable between exports.

Changing `project.default_deck` between two `write_apkg()` calls changes deck resolution for notes that do not set `note.deck(name)`.

## Custom Note Types

```python
from anki_forge import Field, GenerationRule, Note, NoteType, Project, Template

nt = (
    NoteType.custom("jp-vocab")
    .field(Field("Expression", key="expr", identity=True, sort=True, required=True))
    .field(Field("Meaning", key="meaning"))
    .template(Template("Recognition", front="{{Expression}}", back="{{Meaning}}", generate_when=GenerationRule.all(["expr"])))
)
project = Project("Japanese").add_notetype(nt)
project.add_note(Note("jp-vocab").text("expr", "食べる").html("meaning", "<b>to eat</b>"))
```

## Media

```python
audio = project.media.add_file("taberu.mp3", export_as="taberu.mp3")
image = project.media.add_bytes(source_label="diagram.png", data=b"png-bytes", export_as="diagram.png")
project.add_note(Note("jp-vocab").sound("audio", audio).image("picture", image))
```
