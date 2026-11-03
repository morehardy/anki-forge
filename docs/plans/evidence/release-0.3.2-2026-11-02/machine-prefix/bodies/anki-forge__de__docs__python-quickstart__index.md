# Anki-Karten mit Python erzeugen

Canonical page: https://morehardy.github.io/anki-forge/de/docs/python-quickstart/
Language: de
Description: Installiere ankiforge über PyPI, erzeuge Karten mit einem kurzen Python-Skript und exportiere einen APKG-Stapel zum Import in Anki.

Erstelle und aktiviere eine virtuelle Umgebung und installiere anschließend das [veröffentlichte PyPI-Paket](https://pypi.org/project/ankiforge/):

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

Unter Windows aktivierst du die Umgebung mit `.venv\Scripts\Activate.ps1`. Verwende gewöhnliches CPython ab Version 3.11 und ein passendes Wheel für deine Plattform. CPython 3.11–3.14 sind durch dokumentierte Release-Prüfungen abgedeckt; siehe [geprüfte Umgebungen](https://morehardy.github.io/anki-forge/de/docs/compatibility/).

## Einen Stapel exportieren

Speichere Folgendes als `main.py`:

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

Öffne die dauerhaft gespeicherte Datei `spanish.apkg` in Anki. Sie enthält **hola → hello** im Stapel **Spanish**. Behalte den Namensraum `spanish` und den Notizschlüssel `es:hola` für spätere Änderungen bei. Für den Export muss Anki nicht installiert sein.

## Nächste Schritte

Für eine Vokabeltabelle folge [CSV zu Anki](https://morehardy.github.io/anki-forge/de/docs/csv-to-anki/). Um Kartenentwürfe aus Aufzeichnungen zu erstellen, beginne mit [Karten mit KI-Unterstützung erstellen](https://morehardy.github.io/anki-forge/de/docs/ai-anki-cards/).

Eine vollständige Aufgabe mit Medien und Aktualisierung findest du unter [JSON zu Anki](https://morehardy.github.io/anki-forge/de/docs/json-to-anki/#python).

-   [Bilder und Audio](https://morehardy.github.io/anki-forge/de/docs/media/) und [Python-Medien-API](https://morehardy.github.io/anki-forge/de/docs/python-api/#media-and-occlusion)
-   [Eigene Notiztypen](https://morehardy.github.io/anki-forge/de/docs/custom-notetypes/) und [Vorlagenpakete](https://morehardy.github.io/anki-forge/de/docs/templates/)
-   [Bildverdeckung](https://morehardy.github.io/anki-forge/de/docs/image-occlusion/)
-   [Vergleichen und aktualisieren](https://morehardy.github.io/anki-forge/de/docs/updates/)
-   [Python-API](https://morehardy.github.io/anki-forge/de/docs/python-api/) und [Diagnostik](https://morehardy.github.io/anki-forge/de/docs/python-diagnostics/)
-   [Von genanki umsteigen](https://morehardy.github.io/anki-forge/de/docs/genanki-migration/)

Für native Builds aus dem Quellcode verwende [Aus dem Quellcode bauen](https://morehardy.github.io/anki-forge/de/docs/development/).
