# Your first deck with Python

Create one Spanish flashcard using the native Python 0.2 SDK. This guide uses
the source checkout; the [verification record](../../bindings/python/COVERAGE.md)
is not a claim that the candidate has been published to PyPI.

## Install from source

You need Git, Rust 1.92 and ordinary CPython 3.11 or 3.12. From the repository root:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install maturin==1.15.0
maturin develop --manifest-path bindings/python/native/Cargo.toml --locked
python bindings/python/examples/docs_basic.py target/python-basic
```

On Windows, activate `.venv\Scripts\Activate.ps1` in PowerShell instead of the
shell activation command. Select a supported interpreter when creating the venv.
The first Maturin build compiles Rust dependencies. Adding the source directory
to `PYTHONPATH` does not build the native extension.

## Read the complete program

<!-- source: bindings/python/examples/docs_basic.py -->
```python
"""python docs_basic.py [OUTPUT_DIRECTORY]"""
from pathlib import Path
import sys

from anki_forge import Note, Project

output = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
output.mkdir(parents=True, exist_ok=True)
project = Project("Spanish", stable_id="docs-spanish", default_deck="Spanish", base_dir=output)
project.add_note(Note.basic("hola", "hello", stable_id="es:hola"))
project.validate().ensure_success()
report = project.write_apkg("spanish.apkg")
report.ensure_success()
assert report.counts["notes"] == 1 and report.counts["cards"] == 1
print(output / "spanish.apkg")
```
<!-- /source -->

Open `target/python-basic/spanish.apkg` in Anki. The **Spanish** deck contains one
note and one card: **hola → hello**. The assertions check package report counts.
Anki is needed to study the deck, not to generate the package.

## Use it in your own application

Keep the same virtual environment active, save the complete program as
`make_deck.py` in your application directory, and run `python make_deck.py`.
It writes `spanish.apkg` in that directory. For a separate environment, build a
wheel from the checkout and install the matching file:

```sh
maturin build --manifest-path bindings/python/native/Cargo.toml --release --locked --out bindings/python/dist
```

Activate the destination environment and run `python -m pip install` with the
actual absolute path to that wheel. Choose the file for the destination platform;
the installed consumer does not need Cargo, the checkout or contract files.

## Custom note types

Use `NoteType.custom`, `Field` and `Template` to declare your own fields and HTML.
HTML uses display names; field setters and generation rules use stable keys.
See the [Python API](api.md#custom-note-types) and the
[complete native workflow](../../bindings/python/examples/native_workflow.py).

Project additions capture input snapshots. Finish editing a Note/NoteType before
adding it. Construction settings are read-only; build a new Project with the same
stable identities for a revised dataset.

## Media

Run the self-contained media example; it generates a real SVG and one-second WAV:

```sh
python bindings/python/examples/docs_media.py target/python-media
```

Import `target/python-media/media.apkg`. Its two cards reveal a green circle and
play an A4 tone, with two registered media files.

<!-- source: bindings/python/examples/docs_media.py -->
```python
"""Generate real image/audio assets: python docs_media.py [OUTPUT_DIRECTORY]."""
from pathlib import Path
import math
import struct
import sys
import wave

from anki_forge import Note, Project

output = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
output.mkdir(parents=True, exist_ok=True)
(output / "diagram.svg").write_text(
    '<svg xmlns="http://www.w3.org/2000/svg" width="240" height="120">'
    '<rect width="240" height="120" fill="#edf3f1"/>'
    '<circle cx="120" cy="60" r="35" fill="#116e60"/></svg>',
    encoding="utf-8",
)
with wave.open(str(output / "tone.wav"), "wb") as audio:
    audio.setnchannels(1)
    audio.setsampwidth(2)
    audio.setframerate(8000)
    audio.writeframes(b"".join(
        struct.pack("<h", int(4000 * math.sin(2 * math.pi * 440 * i / 8000)))
        for i in range(8000)
    ))

project = Project("Media", stable_id="docs-media", base_dir=output)
picture = project.media.add_file("diagram.svg", export_as="diagram.svg")
sound = project.media.add_file("tone.wav", export_as="tone.wav")
project.add_note(Note.basic("Reveal a green circle", "", stable_id="media:circle").image("back", picture))
project.add_note(Note.basic("Play A4 (440 Hz)", "", stable_id="media:tone").sound("back", sound))
report = project.write_apkg("media.apkg")
report.ensure_success()
assert report.counts["notes"] == 2 and report.counts["cards"] == 2 and report.counts["media"] == 2
print(output / "media.apkg")
```
<!-- /source -->

File registration reads and fingerprints the source immediately. Keep files
available and unchanged through build. For small in-memory content, `add_bytes`
accepts bytes/bytearray up to 64 KiB and rejects empty input. Relative paths use
`base_dir` fixed at construction. See [diagnostics](diagnostics.md).

## Long-term projects

Keep stable project and note IDs. A previous APKG or identity lockfile supplies
revision evidence; see [update-safe builds](diagnostics.md#update-safe-builds).
For bundles, comparison, Artifact ownership and file output, run:

```sh
python bindings/python/examples/native_workflow.py target/python-workflow
```

Each Project/Deck permits one active operation. Independent objects can work
concurrently; mutable authoring inputs require caller synchronization.
Existing Python 0.1 users should read [migration](../../bindings/python/MIGRATION.md).
