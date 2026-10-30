# Anki Forge vs genanki

Consider Anki Forge when you are designing a repeatable publishing workflow and
need owned media, structured diagnostics, comparison and policy review before
releasing updates. Keep a proven genanki workflow when its Python scripts,
templates and media already meet your needs and migration adds more cost than value.

| Decision | Anki Forge | genanki |
| --- | --- | --- |
| Authoring language | Rust core; Rust, Python and Node / TypeScript interfaces | Python |
| Shared basics | Basic, Cloze, custom templates, media and offline APKG generation | Basic, Cloze, custom templates, media and offline APKG generation |
| Identity | Explicit namespace and note keys; retain original APKG for updates | Configurable GUIDs and numeric model/deck IDs; supports re-import updates |
| Before release | Compare findings, risk policy and inspected prepared candidate | Your script and tests define release review |
| Media ownership | Typed images/sounds carry owned snapshots | Script supplies package media files and references |
| Programmatic Image Occlusion | Keyed rectangle masks through the public API | Evaluate an appropriate model or extension for your workflow |
| Runtime cost | Native dependencies; use a supported platform and interpreter | Existing Python deployment may be cheaper to retain |
| Migration cost | Explicitly validate old identity mapping, templates and imports | No migration cost for an already working genanki pipeline |

## The same small task

To turn three JSON questions into a deck with one image and sound, both libraries
let Python parse the JSON and author notes. With genanki, choose the model and deck
IDs, assign field values and GUID rules, add media files to the package, then write
the APKG. With Anki Forge, choose a namespace and durable business keys, compose
owned media into notes, build v1, then compare and prepare v2 against the original
v1 APKG before publication. The [complete JSON recipe](json-to-anki.md) includes
Rust, Python and TypeScript implementations and their run commands.

## Existing genanki decks

[genanki supports custom GUIDs](https://github.com/kerrickstaley/genanki#note-guids),
media and updates on re-import. It is incorrect to treat stable identity as an
exclusive Anki Forge capability. The default GUID rule can depend on fields;
choose a deliberate GUID rule when fields will change.

Anki Forge does not automatically recover arbitrary genanki numeric IDs or GUIDs.
Without a separately validated mapping, treat your first Anki Forge distribution
as an identity boundary. Neither this comparison nor a successful APKG build
promises lossless migration or preservation of all review history.
Read the [migration guide](python/genanki-migration.md) before moving an existing deck.

## Historical performance evidence

The 2026-10-08 comparison measured the Rust Project API against genanki 0.13.1 on
an Apple M1 Pro. At 1,000 Basic notes, the five measured Rust workloads used
45.7–53.9% less export time; some media workloads used more peak memory.
[Method, raw results and limitations](https://github.com/morehardy/anki-forge/blob/bef4aeb653fc875f216614e73d8a617be039b9cf/benchmarks/results/20261008-latest-commit-genanki/report.md).
It did not measure current Python or Node host performance and gives no equivalent
speed guarantee for those SDKs or other workloads.

Anki Forge maintainers · reviewed 2026-10-25 against public Anki Forge 0.3.1 APIs,
[genanki's documentation](https://github.com/kerrickstaley/genanki), and the archived
measurement. Check [runtime compatibility](compatibility.md) and
[choose a language](https://ankiforge.dev/docs/languages/) to start a new task.
