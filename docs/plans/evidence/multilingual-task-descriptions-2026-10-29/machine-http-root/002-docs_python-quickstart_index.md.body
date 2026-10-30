# Generate Anki flashcards with Python

Canonical page: https://ankiforge.dev/docs/python-quickstart/
Language: en
Description: Install ankiforge from PyPI, generate Anki flashcards with a short Python script, and export an APKG deck you can import into Anki.

Create and activate a virtual environment, then install the [public PyPI package](https://pypi.org/project/ankiforge/):

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

On Windows, activate with `.venv\Scripts\Activate.ps1`. Use ordinary CPython 3.11 or later and a matching platform wheel; CPython 3.11–3.14 is covered by recorded release checks; see [verified environments](https://ankiforge.dev/docs/compatibility/).

## Export a deck

Save this as `main.py`:

```python
from ankiforge import Project, Note, BuildOptions

project = Project('spanish', default_deck='Spanish')
project.add('es:hola', Note.basic('hola', 'hello'))
output = project.build(BuildOptions.to('spanish.apkg'))
print(output.artifact.path)
```

```sh
python main.py
```

Open the persistent `spanish.apkg` in Anki. It contains **hola → hello** in **Spanish**. Keep namespace `spanish` and note key `es:hola` stable for later edits. Exporting does not require Anki to be installed.

## Continue

For a vocabulary spreadsheet, follow [CSV to Anki](https://ankiforge.dev/docs/csv-to-anki/). To draft cards from source notes, start with [AI-assisted card generation](https://ankiforge.dev/docs/ai-anki-cards/).

For a complete media and update task, follow [JSON to Anki](https://ankiforge.dev/docs/json-to-anki/#python).

-   [Images and audio](https://ankiforge.dev/docs/media/) and [Python media API](https://ankiforge.dev/docs/python-api/#media-and-occlusion)
-   [Custom note types](https://ankiforge.dev/docs/custom-notetypes/) and [template bundles](https://ankiforge.dev/docs/templates/)
-   [Image Occlusion](https://ankiforge.dev/docs/image-occlusion/)
-   [Compare and update](https://ankiforge.dev/docs/updates/)
-   [Python API](https://ankiforge.dev/docs/python-api/) and [diagnostics](https://ankiforge.dev/docs/python-diagnostics/)
-   [Move from genanki](https://ankiforge.dev/docs/genanki-migration/)

For native source builds, use [source builds](https://ankiforge.dev/docs/development/).
