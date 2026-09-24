# Moving from genanki concepts

AnkiForge uses stable data keys, validated immutable models, owned media snapshots and explicit build/update operations. This is a new authoring model; it does not emulate genanki objects or the removed AnkiForge Deck/lockfile interfaces.

| genanki concept | AnkiForge equivalent |
| --- | --- |
| Model with numeric ID | `NoteType.builder(stable_key)...build()` |
| Field names used as identifiers | Stable Field keys plus separate display names |
| Note carrying field strings | `model.note().field(key, content)` with an owned model |
| Deck as a note container | Project with a default deck; optional note-level `.deck(name)` |
| Package media file paths | Owned `Media.file(path)` values, automatically collected through typed content |
| Package write | `project.build(BuildOptions.to(path))` |
| GUID derived from field contents | Explicit stable namespace and note key |

```python
from anki_forge import Project, NoteType, Field, Template, BuildOptions

model = (NoteType.builder('vocabulary').name('Vocabulary')
    .field(Field('word', name='Word', required=True))
    .field(Field('meaning', name='Meaning'))
    .template(Template('recognition', '{{word}}', '{{FrontSide}}<hr>{{meaning}}'))
    .build())
project = Project('vocabulary-course', default_deck='Languages::Vocabulary')
project.add('dictionary:cell', model.note().field('word', 'cell').field('meaning', '细胞'))
project.build(BuildOptions.to('vocabulary.apkg'))
```

Choose keys from persistent business identifiers. Renaming a display name or changing a definition's text must not change those keys. Do not hash current field content unless content-addressed identity is an explicit requirement of your importer. For source data without keys, assign and persist them outside the SDK.

Template expressions use Field keys. Display names can be translated independently, and the library compiles key references into Anki field names. Model completion rejects missing or conflicting declarations before a note can be added. Adding notes collects model dependencies automatically and rejects conflicting definitions under one model key.

Strings are plain text; wrap intentional HTML with `Content.html(...)`. Use `media.image()` / `media.sound()` and `Content.sequence(...)` to compose content. File snapshots survive removal or edits to the original path. Explicitly declare dependencies referenced by raw HTML, CSS or scripts using `.asset(media)` on the model builder or `project.add_asset(media)`.

For later releases, build a new Project with the same namespace and data keys and call `BuildOptions.to(next_path).update_from(previous_original_apkg)`. The original APKG supplies complete identity evidence. This workflow does not recover identity from arbitrary older packages or promise automatic conversion of existing genanki numeric IDs/GUIDs. Treat the first new-model distribution as a deliberate identity boundary unless a separately validated data migration establishes correspondence.

See [quickstart](quick-start.md), [API reference](api.md) and [update policy](diagnostics.md).
