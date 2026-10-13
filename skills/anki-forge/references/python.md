# Python: public Anki Forge 0.3.0

Requires ordinary CPython 3.11+ and a matching native wheel. Inspect the project's
Python dependency files and environment first. For a new application:

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.0
```

Windows activation: `.venv\Scripts\Activate.ps1`. Check
[verified interpreters and platforms](https://ankiforge.dev/docs/compatibility/)
if pip cannot find a matching wheel; do not silently substitute a source build.
Save as `main.py`:

<!-- source: bindings/python/examples/quickstart.py -->
```python
from ankiforge import Project, Note, BuildOptions

project = Project('spanish', default_deck='Spanish')
project.add('es:hola', Note.basic('hola', 'hello'))
output = project.build(BuildOptions.to('spanish.apkg'))
print(output.artifact.path)
```
<!-- /source -->

```sh
python -m py_compile main.py
python main.py
python -m pip show ankiforge
```

Expect `spanish.apkg` in the application's working directory, with one note,
one Basic card (**hola → hello**) and no media. Check that the file exists;
`output.report.counts` exposes `notes`, `cards` and `media` for assertions.
Import into Anki when the user wants client verification; export needs no Anki.

Keep namespace `spanish` and note key `es:hola` stable. Operations are synchronous.
`BuildOptions.to(path)` produces persistent output. Artifact `.close()` or context
manager exit releases an owner without deleting a persistent file. The last owner
of temporary output removes it; reports alone do not retain the artifact.

For advanced tasks read [workflows.md](workflows.md) and the Python API. Python
uses snake_case such as `with_export_name`, `add_asset` and `update_from`.
Recreate native objects after a process fork; inherited handles cannot be used.

- [Python quickstart](https://ankiforge.dev/docs/python-quickstart/)
- [Python API](https://ankiforge.dev/docs/python-api/)
- [Python diagnostics](https://ankiforge.dev/docs/python-diagnostics/)
- [Source quickstart](https://github.com/morehardy/anki-forge/blob/main/docs/python/quick-start.md)
