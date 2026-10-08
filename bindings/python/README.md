# Anki Forge for Python

Build Basic, Cloze, Image Occlusion and custom cards, package media, and check updates before distributing a new deck.

## Install

Use an ordinary CPython interpreter with a matching platform wheel; CPython 3.11/3.12 are verified in the recorded package checks. See [verified environments](https://ankiforge.dev/docs/compatibility/).
The public package is [`ankiforge` 0.2.0](https://pypi.org/project/ankiforge/).

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.2.0
```

On Windows, activate the environment with `.venv\Scripts\Activate.ps1`.

## Your first deck

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

Open the persistent `spanish.apkg` in Anki. It contains one Basic card in **Spanish**, with **hola → hello**. Keep `spanish` and `es:hola` stable when editing this publication.

## Continue

- [Complete quickstart](https://ankiforge.dev/docs/python-quickstart/)
- [Images and audio](https://ankiforge.dev/docs/media/)
- [Custom note types](https://ankiforge.dev/docs/custom-notetypes/) and [template bundles](https://ankiforge.dev/docs/templates/)
- [Image Occlusion](https://ankiforge.dev/docs/image-occlusion/)
- [Compare and update](https://ankiforge.dev/docs/updates/)
- [Python API](https://ankiforge.dev/docs/python-api/)
- [Source development](https://ankiforge.dev/docs/development/)
