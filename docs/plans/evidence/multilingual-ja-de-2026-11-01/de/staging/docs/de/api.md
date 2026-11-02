# Überblick über die öffentliche API

Anki Forge ist eine Open-Source-Bibliothek zur Erzeugung von Anki-APKGs mit einem Rust-Kern und Schnittstellen für Rust, Python und Node.js. Verwende das veröffentlichte Paket `ankiforge`, um Notizen zu erstellen, Medien zu bündeln und Aktualisierungen vor der Verteilung eines neuen Stapels zu prüfen. Die Bibliothek steht unter der MIT-Lizenz.

| Aufgabe | Rust | Python | Node / TypeScript |
| --- | --- | --- | --- |
| Notizen und Modelle erstellen | [Rust-API](/docs/rust-api/) | [Python-API](/docs/python-api/) | [Node-API](/docs/node-api/) |
| Bilder, Audio und Bildverdeckung | [Medien](/docs/media/) | [Medien mit eigenem Datenbesitz](/docs/python-api/#media-and-occlusion) | [Medien mit eigenem Datenbesitz](/docs/node-api/#media-and-image-occlusion) |
| Fehler diagnostizieren | [Fehlersuche](/docs/troubleshooting/) | [Diagnostik](/docs/python-diagnostics/) | [Fehler](/docs/node-api/#errors) |
| Vergleichen und veröffentlichen | [Aktualisierungsablauf](/docs/updates/) | [Erzeugen und vergleichen](/docs/python-api/) | [Erzeugen und vergleichen](/docs/node-api/#build-compare-and-output) |

<!-- section: stable-note-identity -->
## Stabile Notizidentität

Behalte für jede logische Notiz einen stabilen Projektnamensraum und Notizschlüssel bei. Für die Aktualisierung einer bestehenden Veröffentlichung benötigst du außerdem deren ursprünglich verteiltes APKG. Ein erneuter Export aus Anki ersetzt nicht die vollständigen Identitätsnachweise im ursprünglichen Paket.

<!-- section: start-with-a-task -->
## Mit einer Aufgabe beginnen

[Wähle deine Programmiersprache](/docs/languages/) oder folge der vollständigen [Anleitung zur Erzeugung und Aktualisierung aus JSON](/docs/json-to-anki/). Unter [Kompatibilität](/docs/compatibility/) findest du die Laufzeitumgebungen der veröffentlichten Pakete, unter [Ausgabegarantien](/docs/build-guarantees/) den Besitz und die Lebensdauer von Dateien.
