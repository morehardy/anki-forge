# Node- und TypeScript-API

Canonical page: https://morehardy.github.io/anki-forge/de/docs/node-api/
Language: de
Description: Öffentliche Methoden, asynchrones Verhalten, Build-Optionen, Ausgabedateien und Berichte des Node-SDK.

Der Einstiegspunkt `ankiforge` stellt das einzige Modell zur Inhaltserstellung über Project bereit. Die Typen sind im [öffentlichen Quellcode](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/index.ts), in den [Berichtsklassen](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/report.ts) und in den [JSON-Snapshot-Deklarationen](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/snapshots.ts) definiert. Beginne mit dem [Schnellstart](https://morehardy.github.io/anki-forge/de/docs/node-quickstart/).

## Project und Notizen

| API | Verhalten |
| --- | --- |
| `new Project(namespace)` | Einen stabilen Veröffentlichungsnamensraum validieren |
| `.defaultDeck(name)` | Das Standardziel festlegen; liefert das Projekt |
| `.add(key, note)` | Notiz, Modell und Medien atomar sammeln; doppelte Schlüssel führen zu Fehlern |
| `.addAsset(media)` | Eine ausdrückliche Ressource für direktes HTML/CSS/Skripte einbinden |
| `.clone()` | Unabhängig bearbeitbares Projekt, das unveränderliche Werte teilt |
| `.length` | Anzahl registrierter Notizen |
| `Note.basic(front, back)` / `Note.cloze(text)` | Unveränderliche Notiz mit ihrem eingebauten Modell |
| `model.note().field(key, content)` | Unveränderliche eigene Notiz mit ihrem Modell |
| `note.deck(name).tag(tag).tags(iterable)` | Eine neu konfigurierte Notiz zurückgeben |

Zeichenfolgen werden immer zu Text, auch Cloze-Zeichenfolgen. Ausdrückliches Markup verwendet `Content.html`. `Content.sequence(iterable)` kombiniert Text, HTML und typisierte Medien. Verwende ausdrückliche fachliche Schlüssel; weder Inhalt noch Anzeigenamen bestimmen die Identität.

## Unveränderliche Modelle

```js
import { NoteType, Field, Template, GenerationRule } from 'ankiforge';
const model = NoteType.builder('vocabulary')
  .name('词汇')
  .field(new Field('front', { name: '正面', required: true, sort: true }))
  .field(new Field('back', { name: '背面' }))
  .template(new Template('recognition', {
    front: '{{front}}',
    back: '{{FrontSide}}<hr>{{back}}',
    generation: GenerationRule.all(['front']),
  }))
  .build();
const note = model.note().field('front', 'cell').field('back', '细胞');
```

Builder liefern neue Werte. Ein abgeschlossenes Modell hat keine Methoden zum Verändern von Feldern oder Vorlagen. `builder.clozeField(key)` wählt eigene Cloze-Semantik; `builder.css` und `builder.asset` liefern Gestaltung und ausdrückliche Abhängigkeiten. Vorlagen akzeptieren außerdem `name`, `browserFront`, `browserBack` und `targetDeck`. Vorlagenreferenzen und Erzeugungsbedingungen verwenden Feldschlüssel; Anzeigenamen werden für Anki kompiliert. `GenerationRule.ankiDefault()` wählt Ankis Standardverhalten; `all` und `any` wählen ausdrückliche Bedingungen.

`await NoteType.fromBundle(path, mediaLimits?)` liefert dasselbe unveränderliche Modell aus `template-bundle-v2`. Importierte Texte und Medien hängen nach erfolgreichem Laden nicht mehr vom Quellverzeichnis ab. Modelle lassen sich in mehreren Projekten wiederverwenden; unvereinbare Definitionen unter demselben Modellschlüssel führen atomar zu einem Fehler.

## Medien und Bildverdeckung

`await Media.file(path, limits?)` und `await Media.bytes(Uint8Array, mime, limits?)` liefern Snapshots mit eigenem Datenbesitz. Quelldateien dürfen sich ändern oder verschwinden, nachdem das `await` erfolgreich abgeschlossen wurde. `withExportName(name)` liefert einen umbenannten Wert, ohne frühere Klone oder Inhalte zu ändern. Lies `filename`, `mediaType` und `byteLength` als Eigenschaften.

`media.image()` und `.sound()` liefern typisierten Content, der seine Mediendatei automatisch in ein Projekt einbringt. Ausdrückliches `project.addAsset` und `builder.asset` am Modell decken direkte HTML-/CSS-/Skriptreferenzen ab. Medienlimits akzeptieren `{ maxBytes }`; der Standard beträgt 256 MiB, und für native Bytes gibt es keine Inline-Beschränkung auf 64 KiB.

```js
import { Media, Note, Mask } from 'ankiforge';
const image = await Media.file('diagram.png');
const note = Note.imageOcclusion(image)
  .mask(Mask.rect('nucleus', 10, 10, 20, 20))
  .mask(Mask.rect('wall', 50, 50, 20, 20))
  .mode('hide_one_guess_one')
  .build().field('header', 'Cell');
```

Der Standardmodus ist `hide_all_guess_one`; beide Modi erzeugen separate Karten für Masken mit Schlüsseln. Die Methode `.build()` dieses Builders validiert dekodierte Abmessungen, endliche Rechtecke und eindeutige Schlüssel. `image` und `occlusion` sind reservierte erzeugte Felder. Grenzen für Dekodierung und Aktualisierungen beschreibt [Bildverdeckung](https://morehardy.github.io/anki-forge/de/docs/image-occlusion/).

## Erzeugen, vergleichen und ausgeben

| API | Ergebnis |
| --- | --- |
| `await project.build(BuildOptions.to(path))` | BuildOutput, das ein persistentes Artefakt besitzt |
| `await project.build(BuildOptions.temporary())` | BuildOutput, das ein temporäres Artefakt besitzt |
| `await project.compare(CompareOptions.against(path))` | Abgeschlossener ComparisonReport, auch bei blockierender Richtlinie |
| `options.updateFrom(path)` | Aktualisierung anhand vollständiger Nachweise der ursprünglichen Verteilung konfigurieren |
| `options.inspectLimits(limits)` | Unabhängige Inspektionszähler überschreiben |
| `options.updatePolicy(policy)` | Risikorichtlinie für Aktualisierung oder Vergleich konfigurieren |

Eine erfolgreiche Ausgabe bietet `.artifact`, `.report` und `.snapshot()`. `BuildReport` bietet `.counts`, `.baselineCounts`, `.durationMs`, `.diagnostics`, `.comparison` und `.snapshot()`; er enthält weder ein Erfolgs- oder Fehlerergebnis noch Dateibesitz. `.comparison` ist ein `ComparisonReport` oder `null` und verwendet dieselbe Klasse wie `project.compare()`. Vergleiche bieten `.findings`, `.highestRisk`, `.policy`, `.diagnostics` und `.snapshot()`. Snapshot-Feldnamen entsprechen dem Rust-JSON, einschließlich `allows_publication` und `schema_version`; sie sind keine camelCase-Kopien des nativen Berichts. Verwende nach einer Prüfung auf `null` am Vergleichsobjekt `report.comparison.highestRisk` oder im JSON-Snapshot `report.snapshot().comparison.highest_risk`.

`new UpdatePolicy()` blockiert High/Critical. `.failOn('medium')` ändert den Schwellenwert. `.allow('RISK.NOTE_REMOVED')` akzeptiert ausdrücklich diese ganze Kategorie und erhält ihre Nachweise und ihren Schweregrad. Unbekannte Codes und harte Fehler lassen sich nicht freigeben. Eine Richtlinie bei einem reinen Erstellungs-Build ist ein Fehler. Verwende beim Vergleichen und Erzeugen dieselbe Richtlinie und dieselben Limits, wenn du übereinstimmende Veröffentlichungsentscheidungen benötigst.

Die Inspektionsoptionen umfassen `maxArchiveBytes`, `maxEntries`, `maxCentralDirectoryBytes`, `maxZipEntryBytes`, `maxZipTotalBytes`, `maxMetaBytes`, `maxMediaMapBytes`, `maxIdentityBytes`, `maxCollectionBytes`, `maxMediaBytes`, `maxDecodedTotalBytes` und `maxZstdWindowBytes`. Werte sind nicht negative sichere Ganzzahlen oder `bigint` bis zum Maximum von u64; `0` ist ein tatsächliches Nullbudget. Die Zähler gelten unabhängig für Basisversion und Kandidat. Unbekannte Optionen führen zu einem Fehler.

## Artefaktlebensdauer und Parallelität

JSON-Snapshot-Pfade verwenden den exportierten Typ `PathSnapshot`. Unicode-Pfade sind Zeichenfolgen; native Pfade ohne gültiges Unicode verwenden `{ encoding: "unix_bytes", bytes: number[] }` oder `{ encoding: "windows_wide", units: number[] }`. Die Arrays erhalten die exakten Unix-Bytes oder Windows-UTF-16-Codeeinheiten. Dazu gehören Artefaktpfade in Build-Snapshots, Veröffentlichungspfade bei Fehlern und strukturierte Pfaddetails von Fehlern. Dekodiere einen kodierten Pfad nur für seine ursprüngliche Plattform; unter Unix erhält `Buffer.from(value.bytes)` einen Bytepfad für Nodes Dateisystemfunktionen. Nodes Methoden zur Inhaltserstellung akzeptieren weiterhin Zeichenfolgenpfade.

Relative Medien- und Paketpfade werden beim Aufruf des Lesens verankert; Build-Ziele und Basisversionen beim Erstellen der Optionen. Unter POSIX behalten symbolische Links mit nachfolgendem `..` die Traversierungssemantik des Dateisystems. `persistTo` verankert relative Ziele im beim ursprünglichen Build erfassten Arbeitsverzeichnis, auch bei geklonten und persistent gespeicherten Handles. Spätere Änderungen des Arbeitsverzeichnisses leiten diese Pfade nicht um.

Halte einen Artefaktbesitzer, solange du `.path` verwendest. `.clone()` erzeugt einen unabhängigen Besitzer; `await .close()` gibt diesen Besitzer frei. Der letzte temporäre Besitzer löscht seine Datei. `await .persistTo(path)` liefert ein persistentes Artefakt und lässt das Original auch bei einem Fehler nutzbar. Gewöhnliche Node-Dateistreams können ein Artefakt lesen, solange du seinen Besitzer hältst.

Snapshots sind eingefrorene JSON-Daten und halten keine nativen Dateien. Eine persistente Datei bleibt nach Freigabe des Besitzers erhalten. `BuildOutput` kann nicht vom Aufrufer konstruiert werden; es entsteht nur durch erfolgreiche Builds.

Build und Vergleich laufen asynchron auf einem beim Aufruf erfassten Projektsnapshot mit eigenem Datenbesitz. Spätere Änderungen können diese Anfrage nicht verändern. Auch Medienlesen und Paketladen laufen auf Hintergrundarbeitern. Node ESM und CJS teilen dasselbe native Modul und dieselben Klassenidentitäten.

## Fehler

`SchemaError`, `AddError`, `MediaError`, `TemplateBundleError`, `ImageOcclusionError`, `PolicyError`, `ConfigurationError`, `CompareError`, `BuildError` und `PersistError` erweitern `ForgeError`. Sie bieten `kind`, `code`, `domain`, den nativen Text der Ursachenverkettung in `causes`, strukturierte `sourceDetails` für erkannte E/A-, Schema-, Medien- und Limitursachen sowie vorgangsspezifische `details`. Die ursprüngliche native Ausnahme bleibt als JS-`cause` erhalten.

`BuildError.snapshot()` erhält Fehler- und Veröffentlichungsfakten. `BuildError.report` und `CompareError.report` liefern wie `BuildOutput.report` einen `BuildReport`. Verwende an diesen Berichten `.baselineCounts` und `.comparison`; verwende `.snapshot()` für JSON-Felder in snake\_case. Ein fehlgeschlagener Vergleich kann Teilbeobachtungen und einen `null`\-Vergleich haben. `PersistError.publication` erfasst, ob das Ziel bereits veröffentlicht wurde und ob die Dauerhaftigkeit bestätigt wurde. Medien- und Inspektionsfehler erhalten Limitdetails. Verwende diese maschinenlesbaren Felder, statt Fehlermeldungen zu parsen. `ArtifactClosedError` meldet die Verwendung eines geschlossenen Besitzers; `NativeLoadError` meldet fehlende, inkompatible oder veraltete native Pakete.

## Kontext beim Hinzufügen

Fehlgeschlagenes Hinzufügen lässt das Projekt unverändert. Strukturierter Kontext erfasst die ursprünglichen Notiz-/Modellschlüssel und das Ziel sowie optionale Details zu Konflikten oder Medienverwendung. Rust bietet `note::{AddContext, AddTarget, AddDetail}` über `AddError::context()` / `detail()`; die SDKs bieten `AddError.details` mit typisiertem Kontext und Details. Node-Felder verwenden camelCase und sind tief eingefroren; Python verwendet snake\_case-TypedDicts. Tags behalten ihren Einfügeindex; Feldpositionen behalten ursprüngliche Sequenzindizes (null/None für das Feld, \[\] für seine Wurzel) und optionale UTF-8-Bytebereiche. Fehler des Standardstapels beim Build haben keine Notiz-/Modellschlüssel und bleiben in der Ursachenverkettung verfügbar.

Typisierte Bilder verlangen beim Hinzufügen den MIME-Typ `image/*`; Audioreferenzen akzeptieren `audio/*` oder `video/*`. Andere Kategorien lösen `NOTE.MEDIA_USAGE_INVALID` aus. Konstruktoren bleiben unfehlbar; ausdrücklich deklarierte Medien und direktes HTML sind nicht betroffen. Die Prüfung verwendet den beim Import gespeicherten MIME-Typ und bestätigt keine Wiedergabefähigkeit. Die unabhängige MIME-/Endungsprüfung des Builds gilt weiterhin. Die vollständige Ziel-/Detailtabelle findest du im [Entwurf](https://github.com/morehardy/anki-forge/blob/main/docs/plans/2026-09-28-rust-api-validation-and-errors-design.md).

## Medienimport im Stapel und vorbereitete Veröffentlichung

Medienimport im Stapel und vorbereitete Veröffentlichung sind im veröffentlichten Paket `0.3.1` enthalten.

### Vorbereiten, prüfen und veröffentlichen

```js
import { Project, Note, BuildOptions, BuildError } from 'ankiforge';
const project = new Project('review-once').add('one', Note.basic('Question', 'Answer'));
const previous = await project.build(BuildOptions.to('previous.apkg'));
await previous.artifact.close();
const prepared = await project.preparePublication(
  BuildOptions.to('next.apkg').updateFrom('previous.apkg'),
);
try {
  const report = prepared.report; // immutable observations, no file ownership
  console.log(report.comparison?.snapshot());
  if (report.comparison?.policy.allows_publication !== false) {
    const output = await prepared.publish();
    await output.artifact.close(); // persistent next.apkg remains
  }
} catch (error) {
  if (error instanceof BuildError) console.log(error.snapshot().result);
  // Inspect publication/durability facts: a late failure can follow replacement.
  throw error;
} finally {
  await prepared.close(); // idempotent; cleans an unpublished candidate
}
```

`publish()` übernimmt den nativen Besitzer sofort und führt eine Hintergrundaufgabe aus. Jeder Versuch verbraucht ihn, auch eine Ablehnung durch Richtlinie oder E/A-Fehler. Wiederholte Aufrufe werden mit `PreparedPublicationStateError`, Code `BUILD.PREPARED_UNAVAILABLE` und `reason` `consumed` oder `closed` zurückgewiesen. Schließen nach Beginn der Veröffentlichung bricht diese nicht ab. Berichte bleiben nach dem Schließen lesbar. Optionen werden beim Aufruf der Vorbereitung gebunden; eine Änderung von Richtlinie oder Ziel erfordert eine erneute Vorbereitung. Die Dauer schließt Wartezeit für die Prüfung aus. Weder Berichte noch JSON können wieder in einen Besitzer umgewandelt werden.

Build, Vergleich, Vorbereitung und Klone teilen unveränderliche native Project-Versionen. Inhaltsänderungen bleiben synchron und isoliert. Die erste Änderung, während ein anderer Snapshot lebt, kann das Project kopieren; spätere exklusive Änderungen tun dies nicht. Dadurch wird das Kopieren aus dem Absenden des Promise verlagert. Es verspricht weder schnellere erste Änderungen noch eine geringere maximale Timerverzögerung oder weniger RSS. JavaScript-Byteeingaben werden weiterhin vor der Übergabe an einen Hintergrundarbeiter kopiert. Verwende mit diesen Wrappern Binärdateien für Protokoll 6.

## TypeScript

Installiere `typescript` und `@types/node` in deiner Anwendung. Verwende `main.mts` oder `"type": "module"` und diese `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "outDir": "dist"
  }
}
```

Kompiliere mit `npx tsc` und führe für einen `.mts`\-Einstieg anschließend `node dist/main.mjs` aus.
