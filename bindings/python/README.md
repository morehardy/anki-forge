# AnkiForge Python

The Python SDK calls the default public Rust API through a PyO3 extension. `Project` is the sole authoring container. Choose a stable namespace and a stable key for every note; display names and content can change independently.

```python
from anki_forge import Project, Note, BuildOptions
project = Project('biology-course', name='Biology', default_deck='Science::Biology')
project.add('cell', Note.basic('What is a cell?', 'A unit of life'))
output = project.build(BuildOptions.to('biology.apkg'))
print(output.artifact.path, output.report.counts.notes)
```

Strings always mean plain text. Use `Content.html(...)` for HTML, and `Content.sequence(...)` to combine text, HTML, images and sounds while retaining their dependencies.

```python
from anki_forge import Media, Content
image = Media.file('cell.png')
project.add('cell-image', Note.basic(Content.sequence(['Cell: ', image.image()]), '细胞'))
```

Media owns a snapshot immediately; deleting the input file later is safe. `Media.bytes(data, mime_type)` accepts bytes with an explicit MIME type, including resources larger than 64 KiB. Both constructors accept `limits=MediaLimits(max_bytes=...)`. To use a fixed filename, call `media.with_export_name('cell.png')` before creating content. Existing references retain the earlier name. Assets referenced by raw HTML, CSS or scripts must be declared using `Project.add_asset(media)` or `NoteTypeBuilder.asset(media)`.

Custom models are completed and validated before creating notes:

```python
from anki_forge import Field, Template, NoteType
model = (NoteType.builder('vocab').name('词汇')
    .field(Field('front', name='正面'))
    .field(Field('back', name='背面'))
    .template(Template('recognition', '{{front}}', '{{FrontSide}}<hr>{{back}}', name='识别'))
    .build())
project.add('word:cell', model.note().field('front', 'cell').field('back', '细胞'))
```

Field and template keys are explicit and are used in template expressions. Models, notes, content and media are immutable reusable values. Builder methods return updated values. Models are collected automatically when adding a note. `NoteType.from_bundle(path, limits=MediaLimits(max_bytes=256 << 20))` loads the new `template-bundle-v2` format and snapshots its assets. The per-asset budget may be lowered or raised before reading. Bundle errors expose typed budget and nested error observations in `source_details`.

For image occlusion, use `Note.image_occlusion(media).mask(Mask.rect('nucleus', 2, 2, 4, 4)).build()`. Mask keys are required and preserve card identity when masks are reordered. `OcclusionMode.HIDE_ALL_GUESS_ONE` is the default; `HIDE_ONE_GUESS_ONE` is also supported. Add header/back_extra/comments through normal `Note.field` calls after building the IO note.

Updates use the original prior distribution APKG, which contains complete identity evidence:

```python
from anki_forge import CompareOptions
next_project = Project('biology-course', name='Biology', default_deck='Science::Biology')
next_project.add('cell', Note.basic('What is a cell?', 'The basic unit of life'))
comparison = next_project.compare(CompareOptions.against('biology.apkg'))
output = next_project.build(BuildOptions.to('biology-v2.apkg').update_from('biology.apkg'))
```

A complete comparison returns a `ComparisonReport` even if its policy blocks publication. Update builds raise `BuildError` on that same blocking policy. The default `UpdatePolicy()` blocks High and Critical findings. After reviewing a particular risk category, explicitly accept it with `UpdatePolicy().allow('RISK.NOTE_REMOVED')`; this preserves the original findings and evidence. Unknown codes and hard errors cannot be accepted. Apply a policy with `.update_policy(policy)` on BuildOptions or CompareOptions. Explicit update policies on Create requests are configuration errors. `InspectLimits` applies independently to each inspected baseline and candidate.

Successful builds always return `BuildOutput.artifact`. `BuildReport` contains observations only. `output.snapshot()`, `output.report.snapshot()` and `BuildError.snapshot()` return JSON-serializable copies that do not own files. With `BuildOptions.temporary()`, retain an artifact handle while using the file; close it or release all copies to delete the temporary output. `artifact.persist_to(path)` returns a new persistent handle. Failures preserve domain exception types, `kind`, `code`, source-chain text in `causes`, and publication facts where relevant.

Development: run `maturin develop --manifest-path bindings/python/native/Cargo.toml`, build the independent observer with `cargo build -p anki_forge_python_native --example python_parity`, then run `python -m pytest bindings/python/tests` and `python -m mypy --config-file bindings/python/pyproject.toml bindings/python/src/anki_forge`.

Native handles belong to their creating process. After `os.fork()`, inherited handles reject operations with `BINDING.FORKED_OBJECT`; dropping them cannot remove parent-owned snapshots. Create new values in the child for child-side work. Those new values retain normal cleanup.
