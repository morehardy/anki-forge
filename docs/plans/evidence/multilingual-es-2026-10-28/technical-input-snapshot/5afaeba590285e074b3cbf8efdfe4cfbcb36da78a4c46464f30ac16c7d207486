# Convert CSV to Anki flashcards with Python

Turn a vocabulary spreadsheet into an Anki `.apkg` file with Python and Anki
Forge. This example reads a UTF-8 CSV with `id`, `front` and `back` columns,
creates Basic question-and-answer cards, and exports a deck you can share.
Python's standard-library CSV parser reads the spreadsheet; Anki Forge builds
the package. Exporting does not require Anki to be installed.

For a one-off spreadsheet import, Anki already supports
[delimited text files](https://docs.ankiweb.net/manual/importing/text-files).
Use this recipe when you want a repeatable script, your own note IDs and a
distributable APKG. An Excel `.xlsx` file must first be saved as UTF-8 CSV.

<!-- section: prepare-csv -->
## Prepare the CSV

Save this as `cards.csv`, or [download the sample](/generated/recipes/csv-to-anki/cards.csv):

<!-- source: docs/recipes/csv-to-anki/cards.csv -->
```csv
id,front,back
es:hola,hola,hello
es:gracias,gracias,thank you
es:adios,adiós,goodbye
```
<!-- /source -->

Keep the header exactly as shown and use commas as separators. Quote fields
containing commas, double quotes or line breaks using
[CSV quoting rules](https://docs.python.org/3/library/csv.html).
The script accepts a UTF-8 byte-order mark, as some spreadsheet exports include one.
It rejects duplicate IDs, missing values, extra columns and empty input.

Assign an ID once and keep it when the question or answer changes. Do not derive
IDs from a row's position or editable card text. Use a different namespace for
unrelated datasets; keep the namespace stable for successive versions of this deck.

<!-- section: install -->
## Install Anki Forge

Use ordinary CPython 3.11 or later on a
[supported platform](compatibility.md). Create and activate a virtual environment:

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

On Windows, activate with `.venv\Scripts\Activate.ps1`.
No spreadsheet library or AI service is needed for this conversion.

<!-- section: convert -->
## Generate the APKG

Save the following as `csv_to_anki.py` beside `cards.csv`, or
[download the script](/generated/recipes/csv-to-anki/csv_to_anki.py):

<!-- source: bindings/python/examples/csv_to_anki.py -->
```python
"""Convert a UTF-8 id,front,back CSV into a persistent Basic-card APKG."""
import argparse
import csv
from pathlib import Path

from ankiforge import BuildOptions, Note, Project


def build_csv(source: Path, destination: Path, namespace: str, deck: str) -> int:
    if destination.exists():
        raise ValueError('Choose a new output path; keep previously distributed APKGs')
    project = Project(namespace, default_deck=deck)
    seen = set()
    with source.open(encoding='utf-8-sig', newline='') as handle:
        rows = csv.DictReader(handle, strict=True)
        if rows.fieldnames != ['id', 'front', 'back']:
            raise ValueError('Expected the header id,front,back in that order')
        for row in rows:
            if None in row or any(value is None or not value.strip() for value in row.values()):
                raise ValueError(f'Line {rows.line_num}: expected three nonempty fields')
            key = row['id'].strip()
            if key in seen:
                raise ValueError(f'Line {rows.line_num}: duplicate id {key!r}')
            project.add(key, Note.basic(row['front'], row['back']))
            seen.add(key)
    if not seen:
        raise ValueError('The CSV must contain at least one note')
    project.build(BuildOptions.to(destination))
    return len(seen)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('destination', type=Path)
    parser.add_argument('--namespace', default='csv-vocabulary')
    parser.add_argument('--deck', default='Vocabulary')
    args = parser.parse_args()
    count = build_csv(args.source, args.destination, args.namespace, args.deck)
    print(f'{args.destination}: {count} notes, {count} Basic cards')
```
<!-- /source -->

Run it from that directory:

```sh
python csv_to_anki.py cards.csv vocabulary-v1.apkg --namespace spanish-vocabulary --deck Spanish
```

The sample produces `vocabulary-v1.apkg` with **three notes and three Basic
cards** in the **Spanish** deck, with no media. It prints:

```text
vocabulary-v1.apkg: 3 notes, 3 Basic cards
```

Choose an output path that does not exist. The script refuses to replace a
previous file so you can retain the original distribution package.

<!-- section: verify-and-extend -->
## Import, check and extend

Open `vocabulary-v1.apkg` in Anki. Check that the deck contains `hola → hello`,
`gracias → thank you` and `adiós → goodbye`; preview the cards and verify accents.
Check the content before distributing it, especially if an AI drafted the CSV.

This recipe creates text-only Basic cards. For other workflows:

- [AI-assisted Anki card generation](ai-anki-cards.md): draft and review CSV rows from source notes.
- [JSON to Anki](json-to-anki.md): use Python, Node / TypeScript or Rust, package images and audio, and review a second release.
- [Basic and Cloze cards](cards.md): choose question-and-answer or fill-in-the-blank cards.
- [Update generated Anki decks](updates.md): use the previous original APKG as an update baseline and inspect the comparison.

For a later release, preserve the namespace and note IDs and follow the update
workflow before publishing. Stable IDs help identify notes; Anki's import
settings and newer local edits still affect updates.
