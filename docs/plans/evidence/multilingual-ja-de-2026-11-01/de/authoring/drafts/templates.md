# Vorlagenpakete

Verwende dasselbe Anki-Kartendesign in mehreren Stapeln: Bewahre die Vorlagen, das CSS und die Medien eines Notiztyps in einem Verzeichnis auf und lade es für normale Karten oder Cloze-Karten. Die Beispiele verwenden Rust; Python und Node.js laden Pakete mit `NoteType.from_bundle` und `NoteType.fromBundle` (siehe [Python-API](/docs/python-api/#models-and-templates) und [Node-API](/docs/node-api/#immutable-models)).

Ein Vorlagenpaket umfasst eine wiederverwendbare Modelldeklaration, Vorlagentext, CSS und alle ausdrücklich deklarierten Medienabhängigkeiten. `NoteType::from_bundle(directory)` validiert das vollständige Paket und erstellt Snapshots mit eigenem Datenbesitz, bevor es denselben unveränderlichen `NoteType` zurückgibt, der auch bei der Definition im Code verwendet wird. Es verändert ein Projekt nicht teilweise.

<!-- section: run-the-bundled-example -->
## Das Paketbeispiel ausführen

[[CODE0]]

Der Ablauf lädt das [Testpaket für einen eigenen Cloze-Typ](https://github.com/morehardy/anki-forge/blob/main/contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml) und schreibt `target/docs-examples/template.apkg`. Seine Vorlage weist Karten dem Stapel **Languages::Cloze** zu. Eine Notiz erzeugt zwei Karten.

<!-- section: manifest-and-template-references -->
## Manifest und Vorlagenreferenzen

Die Manifestdatei heißt `anki-template.yaml`. Nur das neue Format wird akzeptiert:

<!-- source: contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml -->
[[CODE1]]
<!-- /source -->

`note_type.key`, Feldschlüssel und Vorlagenschlüssel sind erforderliche stabile Bezeichner. `name` ist optional und entspricht standardmäßig dem jeweiligen Schlüssel. `cloze_field` wählt ein Cloze-Modell; eine separate Typdeklaration ist nicht nötig. Ein Feld ohne `required: true` darf fehlen. Unbekannte Manifesteigenschaften werden zurückgewiesen.

Jede Vorlage verwendet **Feldschlüssel**, auch für Vorderseite, Rückseite und Kartenbrowser: `{{cloze:text}}`, `{{extra}}` und bedingte Abschnitte wie `{{#extra}}`. Anzeigenamen wie Sentence und Extra werden nach der Bindung für Anki erzeugt. Der Lader erhält ursprüngliche Dateipfade und Bytebereiche der Vorlagen in Fehlermeldungen.

<!-- section: load-and-reuse-a-model -->
## Ein Modell laden und wiederverwenden

<!-- source: anki_forge/examples/docs_workflow.rs#bundle -->
[[CODE2]]
<!-- /source -->

Für eine unabhängige Anwendung erwartet dieses vollständige Beispiel dasselbe Paket unter `fixtures/template-bundle`:

[[CODE3]]

Das zurückgegebene Modell ist vom Quellverzeichnis unabhängig. Du kannst es mit einem anderen Projekt teilen oder nach dem Entfernen dieses Verzeichnisses verwenden. Ein Ladefehler liefert kein unvollständiges Modell und räumt die nicht anderweitig gehaltenen Snapshots auf.

<!-- section: assets-and-budgets -->
## Mediendateien und Budgets

Deklariere jede Datei, die direkt von Vorlagen, CSS oder Skripten verwendet wird, ausdrücklich. Jeder `assets`-Eintrag verwendet einen zum Paket relativen Pfad und einen festen Exportnamen:

[[CODE4]]

Anschließend können Vorlagen `<img src="badge.png">` und CSS `url('labels.woff')` verwenden. Die Eigenschaft `css_file` gibt die CSS-Quelldatei an; Medien werden nicht aus beliebigem HTML, CSS oder JavaScript abgeleitet. Deklarierte Medien bleiben auch ohne erkannte statische Referenz im APKG. Exportnamen unterliegen denselben Regeln für portable Namen und Unicode-/Groß-/Kleinschreibungskonflikte wie [Media](/docs/media/). Der vollständige Exportname ist einschließlich Endung auf 255 UTF-8-Bytes begrenzt. Das JSON-Schema prüft Syntax und Zeichenanzahl; `NoteType::from_bundle` prüft zusätzlich Bytebudget, Dateiinhalte und Referenzen, bevor es ein validiertes Modell zurückgibt. Eine erfolgreiche Schemaprüfung allein garantiert kein erfolgreiches Laden.

Absolute Pfade, Pfadwechsel außerhalb des Pakets und symbolische Links auf Ziele außerhalb des Pakets werden zurückgewiesen. Das Manifest ist auf 256 KiB begrenzt; jede Vorlagen-/CSS-Textdatei auf 2 MiB und muss UTF-8 verwenden. Mediensnapshots verwenden das Standard-Medienbudget oder vom Aufrufer über `NoteType::from_bundle_with_limits` bereitgestellte `MediaLimits`. Sowohl vorab ermittelte Längen als auch tatsächlich gelesene Bytes werden geprüft.

Paketfehler erhalten Art und Code, Quellpfad, gegebenenfalls Byteposition und den zugrunde liegenden Schema-, E/A-, YAML-, UTF-8- oder Medienfehler. Der Modellabschluss validiert das Schema; das Hinzufügen von Notizen prüft separat Konflikte auf Projektebene. Siehe [Eigene Notiztypen](/docs/custom-notetypes/) und [Fehlersuche](/docs/troubleshooting/).
