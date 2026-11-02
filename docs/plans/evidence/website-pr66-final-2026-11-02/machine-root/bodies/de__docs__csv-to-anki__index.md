# CSV zu Anki: Karten mit Python erzeugen

Canonical page: https://ankiforge.dev/de/docs/csv-to-anki/
Language: de
Description: Wandle eine UTF-8-Vokabeltabelle mit einem vollständigen Python-Skript in ein Anki-APKG um – mit CSV-Beispiel, stabilen Notiz-IDs und Importprüfungen.

Wandle mit Python und Anki Forge eine Vokabeltabelle in eine Anki-Datei im Format `.apkg` um. Dieses Beispiel liest eine UTF-8-CSV-Datei mit den Spalten `id`, `front` und `back`, erstellt Basic-Karten mit Frage und Antwort und exportiert einen Stapel, den du weitergeben kannst. Der CSV-Parser aus Pythons Standardbibliothek liest die Tabelle; Anki Forge erstellt das Paket. Für den Export muss Anki nicht installiert sein.

Für den einmaligen Import einer Tabelle unterstützt Anki bereits [Textdateien mit Trennzeichen](https://docs.ankiweb.net/manual/importing/text-files). Verwende diese Anleitung, wenn du ein wiederholbar ausführbares Skript, eigene Notiz-IDs und ein verteilbares APKG benötigst. Eine Excel-Datei im Format `.xlsx` musst du zunächst als UTF-8-CSV speichern.

## Die CSV-Datei vorbereiten

Speichere Folgendes als `cards.csv` oder [lade das Beispiel herunter](https://ankiforge.dev/generated/recipes/csv-to-anki/cards.csv):

```csv
id,front,back
es:hola,hola,hello
es:gracias,gracias,thank you
es:adios,adiós,goodbye
```

Übernimm die Kopfzeile unverändert und verwende Kommas als Trennzeichen. Setze Felder mit Kommas, doppelten Anführungszeichen oder Zeilenumbrüchen gemäß den [CSV-Regeln](https://docs.python.org/3/library/csv.html) in Anführungszeichen. Das Skript akzeptiert eine UTF-8-Byte-Order-Mark, wie sie manche Tabellenexporte hinzufügen. Es weist doppelte IDs, fehlende Werte, zusätzliche Spalten und leere Eingaben zurück.

Vergib eine ID einmal und behalte sie bei, wenn sich die Frage oder Antwort ändert. Leite IDs nicht aus der Zeilenposition oder dem veränderlichen Kartentext ab. Verwende für unabhängige Datensätze unterschiedliche Namensräume; behalte den Namensraum für aufeinanderfolgende Versionen dieses Stapels bei.

## Anki Forge installieren

Verwende gewöhnliches CPython ab Version 3.11 auf einer [unterstützten Plattform](https://ankiforge.dev/de/docs/compatibility/). Erstelle und aktiviere eine virtuelle Umgebung:

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

Unter Windows aktivierst du sie mit `.venv\Scripts\Activate.ps1`. Für diese Umwandlung benötigst du weder eine Tabellenbibliothek noch einen KI-Dienst.

## Das APKG erzeugen

Speichere Folgendes als `csv_to_anki.py` neben `cards.csv` oder [lade das Skript herunter](https://ankiforge.dev/generated/recipes/csv-to-anki/csv_to_anki.py):

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

Führe es in diesem Verzeichnis aus:

```sh
python csv_to_anki.py cards.csv vocabulary-v1.apkg --namespace spanish-vocabulary --deck Spanish
```

Das Beispiel erzeugt `vocabulary-v1.apkg` mit **drei Notizen und drei Basic-Karten** im Stapel **Spanish**, ohne Medien. Es gibt Folgendes aus:

```text
vocabulary-v1.apkg: 3 notes, 3 Basic cards
```

Wähle einen Ausgabepfad, der noch nicht existiert. Das Skript ersetzt keine vorhandene Datei, damit du das ursprünglich verteilte Paket aufbewahren kannst.

## Importieren, prüfen und erweitern

Öffne `vocabulary-v1.apkg` in Anki. Prüfe, ob der Stapel `hola → hello`, `gracias → thank you` und `adiós → goodbye` enthält. Sieh dir die Kartenvorschau an und prüfe die Akzente. Prüfe den Inhalt vor der Verteilung, besonders wenn eine KI die CSV-Daten entworfen hat.

Diese Anleitung erstellt Basic-Karten mit reinem Text. Für andere Abläufe:

-   [Anki-Karten mit KI-Unterstützung erstellen](https://ankiforge.dev/de/docs/ai-anki-cards/): Entwirf und prüfe CSV-Zeilen aus Aufzeichnungen.
-   [JSON zu Anki](https://ankiforge.dev/de/docs/json-to-anki/): Verwende Python, Node / TypeScript oder Rust, bündle Bilder und Audio und prüfe eine zweite Veröffentlichung.
-   [Basic- und Cloze-Karten](https://ankiforge.dev/de/docs/cards/): Wähle zwischen Frage-Antwort- und Lückentextkarten.
-   [Erzeugte Anki-Stapel aktualisieren](https://ankiforge.dev/de/docs/updates/): Verwende das ursprüngliche APKG der Vorgängerversion als Aktualisierungsbasis und prüfe den Vergleich.

Behalte für spätere Veröffentlichungen den Namensraum und die Notiz-IDs bei und durchlaufe vor der Veröffentlichung den Aktualisierungsablauf. Stabile IDs helfen bei der Zuordnung von Notizen; Ankis Importeinstellungen und neuere lokale Änderungen beeinflussen Aktualisierungen weiterhin.
