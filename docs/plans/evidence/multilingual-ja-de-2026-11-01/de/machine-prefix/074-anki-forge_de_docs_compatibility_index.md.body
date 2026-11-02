# Kompatibilität und geprüfte Umgebungen

Canonical page: https://morehardy.github.io/anki-forge/de/docs/compatibility/
Language: de
Description: Prüfe veröffentlichte Pakete, verifizierte Laufzeitumgebungen und Bedingungen für Import und Wiedergabe in Anki-Clients.

## Pakete und geprüfte Umgebungen

Die veröffentlichten Rust-, npm- und PyPI-Pakete haben gemäß Prüfung vom 2026-10-13 die Version `0.3.1`. Beginne mit [Rust](https://morehardy.github.io/anki-forge/de/docs/installation/), [Node / TypeScript](https://morehardy.github.io/anki-forge/de/docs/node-quickstart/) oder [Python](https://morehardy.github.io/anki-forge/de/docs/python-quickstart/).

| Schnittstelle | Paket | Laufzeit und Prüfung |
| --- | --- | --- |
| Rust | [ankiforge 0.3.1](https://crates.io/crates/ankiforge) | Rust 1.92.0 und die aktuelle stabile Version; Paketnutzer auf allen vier unterstützten Plattformen sowie eine neue Anwendung mit dem öffentlichen crates.io-Paket |
| Node / TypeScript | [ankiforge 0.3.1](https://www.npmjs.com/package/ankiforge) | Node 22.13.0 sowie aktuelle Versionen von 22, 24 und 26; 16 Prüfungen der öffentlichen npm-Installation, von ESM/CJS und TypeScript |
| Python | [ankiforge 0.3.1](https://pypi.org/project/ankiforge/) | Gewöhnliches CPython 3.11–3.14; 16 isolierte Installationsprüfungen der öffentlichen PyPI-Wheels |

Die veröffentlichten nativen Pakete decken macOS arm64/x64, Linux x64 GNU/glibc und Windows x64 ab. Allein die Verfügbarkeit eines Pakets bedeutet keinen erfolgreichen Laufzeittest auf jedem System. Die [Node-Abdeckung](https://github.com/morehardy/anki-forge/blob/main/bindings/node/COVERAGE.md), die [Python-Abdeckung](https://github.com/morehardy/anki-forge/blob/main/bindings/python/COVERAGE.md) und die Release-Nachweise dokumentieren die genauen Prüfungen. Die erfolgreichen Release-Läufe für `0.3.1` sind [Rust](https://github.com/morehardy/anki-forge/actions/runs/38371221696), [npm](https://github.com/morehardy/anki-forge/actions/runs/38371222114) und [PyPI](https://github.com/morehardy/anki-forge/actions/runs/38371222047). Andere Python-Implementierungen, Interpreter ohne globalen Interpreter-Lock und Subinterpreter benötigen eine separate Validierung. Browser, Electron, Bun, Alpine/musl und ARM64 Linux/Windows liegen außerhalb der dokumentierten Node-Prüfabdeckung.

Die Anleitungen zu [Medienimporten im Stapel](https://morehardy.github.io/anki-forge/de/docs/media/#batch-media-import) und [vorbereiteter Veröffentlichung](https://morehardy.github.io/anki-forge/de/docs/updates/#build-once-review-then-publish) gelten für die veröffentlichten Pakete der Version `0.3.1`.

## Anki-Import und Wiedergabe

Für einen gewöhnlichen Export muss Anki nicht installiert sein. Import, Darstellung und Medienwiedergabe hängen weiterhin von Version, Codecs und Importeinstellungen des Zielclients ab. Der Benchmark vom 2026-10-08 bestand 40 Prüfungen von Anki-Import, Inhalten und repräsentativer Darstellung – 20 pro Implementierung. Er prüfte nicht jeden Client und jedes Verhalten der Oberfläche oder Audiowiedergabe.

Bewahre für Aktualisierungen das ursprünglich verteilte APKG der Vorgängerversion auf. Erneute Exporte aus Anki erhalten die erforderlichen Identitätsnachweise nicht. Das Weglassen einer Notiz in einem Paket löscht sie nicht aus der Sammlung eines Lernenden. Feld- und Vorlagenänderungen, Sortierfelder, lokale Änderungen und Wiederholungsverlauf erfordern Tests für den jeweiligen Client.

Verwende die [manuellen Clientszenarien](https://github.com/morehardy/anki-forge/blob/main/docs/manual-validation/anki-desktop-v1/TEMPLATE.md) sowie die genaue Anki-Version und Einstellungen jedes Referenztests, um deine eigene Verteilung zu bewerten. Unter [Aktualisierungen](https://morehardy.github.io/anki-forge/de/docs/updates/) und [Build-Garantien](https://morehardy.github.io/anki-forge/de/docs/build-guarantees/) findest du die Clientbedingungen und das Veröffentlichungsverhalten pro Datei.
