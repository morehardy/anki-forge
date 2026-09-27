# Python quickstart

The native Python SDK wraps the Rust public API. Use Python 3.11 or newer and install the platform wheel:

```sh
python -m pip install anki-forge
```

Choose a stable project namespace and stable note keys from your data. Names and content can then change without turning each edit into a new note.

```python
from anki_forge import Project, Note, BuildOptions

project = Project('biology-course', name='Biology', default_deck='Science::Biology')
project.add('cell', Note.basic('What is a cell?', 'The basic unit of life'))
project.add('dna', Note.cloze('DNA stores {{c1::genetic information}}'))
output = project.build(BuildOptions.to('biology.apkg'))
print(output.artifact.path)
print(output.report.counts.notes)
```

`Project` is the only container. A note can override its deck with `.deck('Science::Revision')`; no separate Deck object is needed. Note methods return new values, so retain the result of each configuration call.

## Text, HTML and owned media

Strings are always text, including in Cloze notes. Use `Content.html` for explicit HTML. Image and sound content owns media dependencies and adds them to the project automatically.

```python
from anki_forge import Content, Media

image = Media.file('cell.png')
project.add('cell-picture', Note.basic(
    Content.sequence(['Identify: ', image.image()]),
    Content.html('<strong>A cell</strong>'),
))
```

`Media.file` snapshots the bytes immediately. The original file may be changed or deleted after this call. Reuse the Media value in any project. `Media.bytes(data, 'image/png')` accepts an owned snapshot with an explicit MIME type. To choose a filename, call `.with_export_name('cell.png')` before creating content references. Assets referenced only in handwritten HTML, CSS or scripts are declared with `project.add_asset(media)`.

## Complete a custom model

```python
from anki_forge import Field, Template, NoteType

model = (NoteType.builder('vocab').name('词汇')
    .field(Field('front', name='正面', required=True))
    .field(Field('back', name='背面'))
    .template(Template('recognition', '{{front}}', '{{FrontSide}}<hr>{{back}}', name='识别'))
    .build())
project.add('word:cell', model.note().field('front', 'cell').field('back', '细胞'))
```

The builder validates fields, templates and their references. The completed model is immutable. Note fields and template expressions use stable keys; Anki displays the separate names. Adding a note collects its model automatically and atomically. `NoteType.from_bundle(path)` loads the same immutable model from a `template-bundle-v2` bundle.

## Compare and update

```python
from anki_forge import CompareOptions

next_project = Project('biology-course', name='Biology', default_deck='Science::Biology')
next_project.add('cell', Note.basic('What is a cell?', 'The basic structural unit of life'))
next_project.add('dna', Note.cloze('DNA stores {{c1::genetic information}}'))
comparison = next_project.compare(CompareOptions.against('biology.apkg'))
print(comparison.findings)
next_project.build(BuildOptions.to('biology-v2.apkg').update_from('biology.apkg'))
```

Use the previous original distribution as the baseline. It carries complete identity evidence inside the APKG. A high-risk comparison is a completed analysis; its `allows_publication` may be false. A build using the same blocking policy raises `BuildError` before publishing. See [diagnostics and update policy](diagnostics.md) for explicit risk acceptance and inspection budgets.

For temporary output, use `BuildOptions.temporary()` and retain `output.artifact` for as long as the file is needed. A saved path or JSON snapshot does not retain that file. See the [API reference](api.md) and [complete executable workflow](../../bindings/python/examples/native_workflow.py).
