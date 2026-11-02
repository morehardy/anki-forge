# Anki-Karten mit Node.js erzeugen

Verwende Node.js ab Version 22.13. Führe in einem neuen Anwendungsverzeichnis Folgendes aus:

[[CODE0]]

Dadurch werden das [veröffentlichte npm-Paket](https://www.npmjs.com/package/ankiforge) und die passende native optionale Abhängigkeit installiert. Die abgedeckten Plattformen findest du unter [geprüfte Umgebungen](/docs/compatibility/).

<!-- section: export-a-deck -->
## Einen Stapel exportieren

Speichere Folgendes als `main.mjs`:

<!-- source: bindings/node/examples/quickstart.mjs -->
[[CODE1]]
<!-- /source -->

[[CODE2]]

Öffne die dauerhaft gespeicherte Datei `spanish.apkg` in Anki. Sie enthält **hola → hello** im Stapel **Spanish**. Behalte den Namensraum `spanish` und den Schlüssel `es:hola` für spätere Änderungen bei. Wenn du das persistente Artefakt schließt, bleibt die Datei auf dem Datenträger. Für den Export muss Anki nicht installiert sein.

<!-- section: typescript-and-next-tasks -->
## TypeScript und nächste Aufgaben

Die [JSON-zu-Anki-Anleitung](/docs/json-to-anki/#node) enthält einen vollständigen TypeScript-Ablauf zum Kompilieren und Ausführen.

Verwende `.mjs` für ESM oder `require('ankiforge')` für CommonJS. TypeScript-Deklarationen sind im Paket enthalten; siehe [TypeScript-Konfiguration](/docs/node-api/#typescript).

- [Bilder und Audio](/docs/media/) und [Node-Medien-API](/docs/node-api/#media-and-image-occlusion)
- [Eigene Notiztypen](/docs/custom-notetypes/) und [Vorlagenpakete](/docs/templates/)
- [Bildverdeckung](/docs/image-occlusion/)
- [Vergleichen und aktualisieren](/docs/updates/)
- [Node-API](/docs/node-api/) und [Fehlersuche](/docs/troubleshooting/)

Builds aus dem Quellcode und lokale Paketerstellung beschreibt [Aus dem Quellcode bauen](/docs/development/).
