# Anki-Karten mit Python erzeugen

Erstelle und aktiviere eine virtuelle Umgebung und installiere anschließend das [veröffentlichte PyPI-Paket](https://pypi.org/project/ankiforge/):

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

Unter Windows aktivierst du die Umgebung mit `.venv\Scripts\Activate.ps1`. Verwende gewöhnliches CPython ab Version 3.11 und ein passendes Wheel für deine Plattform. CPython 3.11–3.14 sind durch dokumentierte Release-Prüfungen abgedeckt; siehe [geprüfte Umgebungen](/docs/compatibility/).

<!-- section: export-a-deck -->
## Einen Stapel exportieren

Speichere Folgendes als `main.py`:

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

Öffne die dauerhaft gespeicherte Datei `spanish.apkg` in Anki. Sie enthält **hola → hello** im Stapel **Spanish**. Behalte den Namensraum `spanish` und den Notizschlüssel `es:hola` für spätere Änderungen bei. Für den Export muss Anki nicht installiert sein.

<!-- section: continue -->
## Nächste Schritte

Für eine Vokabeltabelle folge [CSV zu Anki](/docs/csv-to-anki/). Um Kartenentwürfe aus Aufzeichnungen zu erstellen, beginne mit [Karten mit KI-Unterstützung erstellen](/docs/ai-anki-cards/).

Eine vollständige Aufgabe mit Medien und Aktualisierung findest du unter [JSON zu Anki](/docs/json-to-anki/#python).

- [Bilder und Audio](/docs/media/) und [Python-Medien-API](/docs/python-api/#media-and-occlusion)
- [Eigene Notiztypen](/docs/custom-notetypes/) und [Vorlagenpakete](/docs/templates/)
- [Bildverdeckung](/docs/image-occlusion/)
- [Vergleichen und aktualisieren](/docs/updates/)
- [Python-API](/docs/python-api/) und [Diagnostik](/docs/python-diagnostics/)
- [Von genanki umsteigen](/docs/genanki-migration/)

Für native Builds aus dem Quellcode verwende [Aus dem Quellcode bauen](/docs/development/).
