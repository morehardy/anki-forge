Verwende Rust, Node / TypeScript oder Python, um aus deinen Daten Anki-Stapel zu erzeugen. Mit allen drei Sprachen kannst du eine dauerhaft gespeicherte Datei `spanish.apkg` exportieren, ohne Anki zu installieren.

| Sprache | Veröffentlichtes Paket | Installation | Vollständiges erstes Beispiel |
| --- | --- | --- | --- |
| Rust | [ankiforge 0.3.1](https://crates.io/crates/ankiforge) | `cargo add ankiforge@0.3.1` | [Rust-Schnellstart](/docs/quickstart/) |
| Node / TypeScript | [ankiforge 0.3.1](https://www.npmjs.com/package/ankiforge) | `npm install --include=optional ankiforge@0.3.1` | [Node-Schnellstart](/docs/node-quickstart/) |
| Python | [ankiforge 0.3.1](https://pypi.org/project/ankiforge/) | `python -m pip install ankiforge==0.3.1` | [Python-Schnellstart](/docs/python-quickstart/) |

Rust benötigt mindestens Rust 1.92. Node benötigt mindestens Node 22.13 und eine passende native optionale Abhängigkeit. Python verwendet ein natives Wheel für einen gewöhnlichen CPython-Interpreter; CPython 3.11–3.14 wurden in dokumentierten Release-Prüfungen verifiziert. Siehe [geprüfte Umgebungen](/docs/compatibility/).

<!-- section: next-tasks -->
## Nächste Aufgaben

Erzeuge und aktualisiere einen Stapel aus [gemeinsamen JSON-, Bild- und Audiodaten](/docs/json-to-anki/) oder vergleiche [Anki Forge und genanki](/docs/anki-forge-vs-genanki/) für deinen Arbeitsablauf.

Ergänze [Bilder und Audio](/docs/media/), erstelle [eigene Notiztypen](/docs/custom-notetypes/) oder [vergleiche und aktualisiere einen bereits verteilten Stapel](/docs/updates/).

Für TypeScript findest du die [Compiler-Konfiguration](/docs/node-api/#typescript) in der API-Referenz. Für genanki gibt es eine [Migrationsanleitung](/docs/genanki-migration/). Die Entwicklung im Repository und Funktionen, die nur im Quellcode verfügbar sind, haben eigene [Anweisungen zum Bauen aus dem Quellcode](/docs/development/#source-builds).
