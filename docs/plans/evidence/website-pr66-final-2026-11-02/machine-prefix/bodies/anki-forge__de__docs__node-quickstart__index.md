# Anki-Karten mit Node.js erzeugen

Canonical page: https://morehardy.github.io/anki-forge/de/docs/node-quickstart/
Language: de
Description: Installiere ankiforge über npm, erzeuge Karten mit Node.js oder TypeScript und exportiere einen APKG-Stapel zum Import in Anki.

Verwende Node.js ab Version 22.13. Führe in einem neuen Anwendungsverzeichnis Folgendes aus:

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.1
```

Dadurch werden das [veröffentlichte npm-Paket](https://www.npmjs.com/package/ankiforge) und die passende native optionale Abhängigkeit installiert. Die abgedeckten Plattformen findest du unter [geprüfte Umgebungen](https://morehardy.github.io/anki-forge/de/docs/compatibility/).

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

Die [JSON-zu-Anki-Anleitung](https://morehardy.github.io/anki-forge/de/docs/json-to-anki/#node) enthält einen vollständigen TypeScript-Ablauf zum Kompilieren und Ausführen.

Verwende `.mjs` für ESM oder `require('ankiforge')` für CommonJS. TypeScript-Deklarationen sind im Paket enthalten; siehe [TypeScript-Konfiguration](https://morehardy.github.io/anki-forge/de/docs/node-api/#typescript).

-   [Bilder und Audio](https://morehardy.github.io/anki-forge/de/docs/media/) und [Node-Medien-API](https://morehardy.github.io/anki-forge/de/docs/node-api/#media-and-image-occlusion)
-   [Eigene Notiztypen](https://morehardy.github.io/anki-forge/de/docs/custom-notetypes/) und [Vorlagenpakete](https://morehardy.github.io/anki-forge/de/docs/templates/)
-   [Bildverdeckung](https://morehardy.github.io/anki-forge/de/docs/image-occlusion/)
-   [Vergleichen und aktualisieren](https://morehardy.github.io/anki-forge/de/docs/updates/)
-   [Node-API](https://morehardy.github.io/anki-forge/de/docs/node-api/) und [Fehlersuche](https://morehardy.github.io/anki-forge/de/docs/troubleshooting/)

Builds aus dem Quellcode und lokale Paketerstellung beschreibt [Aus dem Quellcode bauen](https://morehardy.github.io/anki-forge/de/docs/development/).
