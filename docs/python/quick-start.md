# Your first deck with Python

Create and activate a virtual environment, then install the [public PyPI package](https://pypi.org/project/ankiforge/):

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.2.0
```

On Windows, activate with `.venv\Scripts\Activate.ps1`.
Use an ordinary CPython interpreter and a matching platform wheel; see [verified environments](../compatibility.md).

## Export a deck

Save this as `main.py`:

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
python main.py
```

Open the persistent `spanish.apkg` in Anki. It contains **hola → hello** in **Spanish**.
Keep namespace `spanish` and note key `es:hola` stable for later edits. Exporting does not require Anki to be installed.

## Continue

- [Images and audio](../media.md) and [Python media API](api.md#media-and-occlusion)
- [Custom note types](../custom-notetypes.md) and [template bundles](../template-bundles.md)
- [Image Occlusion](../image-occlusion.md)
- [Compare and update](../updates.md)
- [Python API](api.md) and [diagnostics](diagnostics.md)
- [Move from genanki](genanki-migration.md)

For native source builds, use [source builds](../source-builds.md).
