# AnkiForge Python

The Python SDK calls the default public Rust API through a PyO3 extension. `Project` is the sole authoring container. Choose a stable namespace and a stable key for every note; display names and content can change independently.

The distribution name and import package are both `ankiforge`. Once the release
is available on PyPI, install it with `python -m pip install ankiforge`.
See the [release status](../../docs/compatibility.md) before choosing a version.

```python
from ankiforge import Project, Note, BuildOptions
project = Project('biology-course', default_deck='Science::Biology')
project.add('cell', Note.basic('What is a cell?', 'A unit of life'))
output = project.build(BuildOptions.to('biology.apkg'))
print(output.artifact.path, output.report.counts.notes)
```

Strings always mean plain text. Use `Content.html(...)` for HTML, and `Content.sequence(...)` to combine text, HTML, images and sounds while retaining their dependencies.

```python
from ankiforge import Media, Content
image = Media.file('cell.png')
project.add('cell-image', Note.basic(Content.sequence(['Cell: ', image.image()]), '细胞'))
```

Media owns a snapshot immediately; deleting the input file later is safe. `Media.bytes(data, mime_type)` accepts bytes with an explicit MIME type, including resources larger than 64 KiB. Both constructors accept `limits=MediaLimits(max_bytes=...)`. To use a fixed filename, call `media.with_export_name('cell.png')` before creating content. Existing references retain the earlier name. Assets referenced by raw HTML, CSS or scripts must be declared using `Project.add_asset(media)` or `NoteTypeBuilder.asset(media)`.

Custom models are completed and validated before creating notes:

```python
from ankiforge import Field, Template, NoteType
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
from ankiforge import CompareOptions
next_project = Project('biology-course', default_deck='Science::Biology')
next_project.add('cell', Note.basic('What is a cell?', 'The basic unit of life'))
comparison = next_project.compare(CompareOptions.against('biology.apkg'))
output = next_project.build(BuildOptions.to('biology-v2.apkg').update_from('biology.apkg'))
```

A complete comparison returns a `ComparisonReport` even if its policy blocks publication. Update builds raise `BuildError` on that same blocking policy. The default `UpdatePolicy()` blocks High and Critical findings. After reviewing a particular risk category, explicitly accept it with `UpdatePolicy().allow('RISK.NOTE_REMOVED')`; this preserves the original findings and evidence. Unknown codes and hard errors cannot be accepted. Apply a policy with `.update_policy(policy)` on BuildOptions or CompareOptions. Explicit update policies on Create requests are configuration errors. `InspectLimits` applies independently to each inspected baseline and candidate.

Successful builds always return `BuildOutput.artifact`. `BuildReport` contains observations only. `output.snapshot()`, `output.report.snapshot()` and `BuildError.snapshot()` return JSON-serializable copies that do not own files. With `BuildOptions.temporary()`, retain an artifact handle while using the file; close it or release all copies to delete the temporary output. `artifact.persist_to(path)` returns a new persistent handle. Failures preserve domain exception types, `kind`, `code`, source-chain text in `causes`, and publication facts where relevant.

Development: run `maturin develop --manifest-path bindings/python/native/Cargo.toml`, build the independent observer with `cargo build -p anki_forge_python_native --example python_parity`, then run `python -m pytest bindings/python/tests` and `python -m mypy --config-file bindings/python/pyproject.toml bindings/python/src/ankiforge`.

Native handles belong to their creating process. After `os.fork()`, inherited handles reject operations with `BINDING.FORKED_OBJECT`; dropping them cannot remove parent-owned snapshots. Create new values in the child for child-side work. Those new values retain normal cleanup.

### Addition diagnostics

Projects have a stable namespace and optional default deck, with no title.
Image and sound constructors remain infallible; adding a note checks retained
MIME categories: image/* for images, audio/* or video/* for sound references.
Other categories raise AddError with code NOTE.MEDIA_USAGE_INVALID. Raw HTML and
explicit assets retain their existing behavior; category checking does not promise
decodability. Build still performs its independent MIME/extension checks.

`AddError.details` is described by exported `AddErrorDetails` TypedDict, with
`context` and nullable `detail`, alongside `error_kind`, `causes` and
`source_details`. Context includes `note_key`, `model_key` and `target`; field
targets include `field_key`, `content_path` and `byte_range`.
Native metadata must report embedded contract 2.1.0, even when binding/core
versions both match 0.2.0.
A field path is null/None for the field, [] for root content, or zero-based
indices into the original sequences. Byte ranges use UTF-8 offsets in the
original text/HTML leaf. Nested source details preserve the same facts.

### Prepare, review and publish

```python
from ankiforge import Project, Note, BuildOptions, BuildError
project = Project('review-once').add('one', Note.basic('Question', 'Answer'))
previous = project.build(BuildOptions.to('previous.apkg'))
previous.artifact.close()
prepared = project.prepare_publication(
    BuildOptions.to('next.apkg').update_from('previous.apkg')
)
try:
    report = prepared.report  # observations; does not own the candidate
    if report.comparison is None or report.comparison.allows_publication:
        output = prepared.publish()
        output.artifact.close()  # persistent next.apkg remains
except BuildError as error:
    print(error.snapshot()['result'])  # actual publication/durability facts
    raise
finally:
    prepared.close()  # idempotent; deletes an unused candidate
```

One publish attempt consumes the owner, including policy or I/O failure. Later
calls raise `PreparedPublicationStateError` with code `BUILD.PREPARED_UNAVAILABLE`
and reason `closed` or `consumed`; close does not cancel an already running
publication. Relative paths bind at preparation invocation, and Project changes
cannot modify the reviewed APKG. Reports survive close without retaining files.
Change options or retry by preparing again. Duration excludes review waiting.
Late errors can mean replacement succeeded with unconfirmed durability; inspect
publication facts before deciding what to do. Native binding protocol 1 and
contract bundle 2.1.0 are required; older same-version binaries are rejected.
