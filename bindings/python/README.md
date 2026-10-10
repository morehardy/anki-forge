<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/morehardy/anki-forge/main/docs/assets/brand/ankiforge-dark.svg">
    <img src="https://raw.githubusercontent.com/morehardy/anki-forge/main/docs/assets/brand/ankiforge.svg" alt="Anki Forge logo: stacked cards with a folded corner and card loop" width="96" height="96">
  </picture>
</p>

<h1 align="center">anki-forge</h1>

<p align="center">
  <a href="https://github.com/morehardy/anki-forge/actions/workflows/contract-ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/morehardy/anki-forge/contract-ci.yml?branch=main&label=tests%20passing" alt="Tests passing"></a>
  <a href="https://pypi.org/project/ankiforge/"><img src="https://img.shields.io/pypi/v/ankiforge?logo=pypi" alt="PyPI version"></a>
  <a href="https://ankiforge.dev/docs/"><img src="https://img.shields.io/badge/docs-ankiforge.dev-blue" alt="Documentation"></a>
  <a href="https://github.com/morehardy/anki-forge/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

<p align="center">Python SDK</p>

Build Basic, Cloze, Image Occlusion and custom cards, package media, and check updates before distributing a new deck.

## Install

Use an ordinary CPython interpreter with a matching platform wheel; CPython 3.11/3.12 are verified in the recorded package checks. See [verified environments](https://ankiforge.dev/docs/compatibility/).
The public package is [`ankiforge` 0.3.0](https://pypi.org/project/ankiforge/).

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.0
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
- [Build SDKs from source](https://ankiforge.dev/docs/development/)
