# Anki-Karten mit Node.js erzeugen

Canonical page: https://ankiforge.dev/de/docs/node-quickstart/
Language: de
Description: Installiere ankiforge über npm, erzeuge Karten mit Node.js oder TypeScript und exportiere einen APKG-Stapel zum Import in Anki.

Verwende Node.js ab Version 22.13. Führe in einem neuen Anwendungsverzeichnis Folgendes aus:

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.1
```

Dadurch werden das [veröffentlichte npm-Paket](https://www.npmjs.com/package/ankiforge) und die passende native optionale Abhängigkeit installiert. Die abgedeckten Plattformen findest du unter [geprüfte Umgebungen](https://ankiforge.dev/de/docs/compatibility/).

## Einen Stapel exportieren

Speichere Folgendes als `main.mjs`:

```js
import { Project, Note, BuildOptions } from 'ankiforge';

const project = new Project('spanish').defaultDeck('Spanish');
project.add('es:hola', Note.basic('hola', 'hello'));
const output = await project.build(BuildOptions.to('spanish.apkg'));
console.log(output.artifact.path);
await output.artifact.close();
```

```sh
node main.mjs
```

Öffne die dauerhaft gespeicherte Datei `spanish.apkg` in Anki. Sie enthält **hola → hello** im Stapel **Spanish**. Behalte den Namensraum `spanish` und den Schlüssel `es:hola` für spätere Änderungen bei. Wenn du das persistente Artefakt schließt, bleibt die Datei auf dem Datenträger. Für den Export muss Anki nicht installiert sein.

## TypeScript und nächste Aufgaben

Die [JSON-zu-Anki-Anleitung](https://ankiforge.dev/de/docs/json-to-anki/#node) enthält einen vollständigen TypeScript-Ablauf zum Kompilieren und Ausführen.

Verwende `.mjs` für ESM oder `require('ankiforge')` für CommonJS. TypeScript-Deklarationen sind im Paket enthalten; siehe [TypeScript-Konfiguration](https://ankiforge.dev/de/docs/node-api/#typescript).

-   [Bilder und Audio](https://ankiforge.dev/de/docs/media/) und [Node-Medien-API](https://ankiforge.dev/de/docs/node-api/#media-and-image-occlusion)
-   [Eigene Notiztypen](https://ankiforge.dev/de/docs/custom-notetypes/) und [Vorlagenpakete](https://ankiforge.dev/de/docs/templates/)
-   [Bildverdeckung](https://ankiforge.dev/de/docs/image-occlusion/)
-   [Vergleichen und aktualisieren](https://ankiforge.dev/de/docs/updates/)
-   [Node-API](https://ankiforge.dev/de/docs/node-api/) und [Fehlersuche](https://ankiforge.dev/de/docs/troubleshooting/)

Builds aus dem Quellcode und lokale Paketerstellung beschreibt [Aus dem Quellcode bauen](https://ankiforge.dev/de/docs/development/).
