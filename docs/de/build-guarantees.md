# Build- und Ausgabegarantien

`Project::build` liefert `BuildOutput` nur nach einem erfolgreichen Vorgang mit einem garantierten APKG-Artefakt. Ein fehlgeschlagener Vorgang liefert `BuildError` mit strukturierten Angaben zu Fehlerart und Fehlercode, Beobachtungen, zugrunde liegenden Ursachen und tatsächlichen Veröffentlichungsfakten.

<!-- section: temporary-and-persistent-artifacts -->
## Temporäre und persistente Artefakte

Wähle das Besitzmodell ausdrücklich:

| Anfrage | Besitz und Lebensdauer |
| --- | --- |
| `BuildOptions::to(path)` | Persistentes Ziel; bleibt nach Freigabe der Handles erhalten |
| `BuildOptions::temporary()` | Temporäre Datei; wird mit ihrem letzten Artefaktbesitzer gelöscht |
| `artifact.persist_to(path)` | Neuer persistenter Besitzer; ursprünglicher Besitzer bleibt nutzbar |

Halte `BuildOutput` oder ein geklontes Artefakt, solange du einen temporären Pfad liest. Ein Bericht oder JSON-Snapshot verlängert die Dateilebensdauer nicht. Rust gibt Besitzer wie gewöhnlich frei; Node bietet `artifact.close()` für eine deterministische Freigabe. Das Kopieren des Pfades in den Anwendungszustand erzeugt keinen Besitzer.

Rust-Artefakthandles behalten absolute Pfade, auch wenn der Build oder `persist_to` ein relatives Ziel verwendet. Spätere Änderungen des Arbeitsverzeichnisses leiten Lese- oder Kopiervorgänge eines bestehenden Handles nicht um. Jedes neue Ziel von `persist_to` wird relativ zum Verzeichnis zum Zeitpunkt dieses Aufrufs aufgelöst.

`BuildReport::snapshot()` enthält nur Beobachtungen. `BuildOutput::snapshot()` und `BuildError::snapshot()` geben zusätzlich das tatsächliche Ergebnis an. Eine Warnung kann in einem erfolgreichen Snapshot vorkommen. Leite das Ergebnis nicht aus dem Schweregrad einer Diagnose, der Existenz eines Pfades oder Vergleichsbefunden in einem Bericht ab.

<!-- section: baselines-and-publication -->
## Basisversionen und Veröffentlichung

Bewahre für Aktualisierungen das ursprünglich verteilte APKG der Vorgängerversion als unveränderliche Basis auf und wähle ein separates Ausgabeziel. Nachweise der Basisversion und Inhalte des Kandidaten werden vor der Veröffentlichung geprüft. Eine Ablehnung durch die Richtlinie ersetzt die Ausgabe nicht. Fehlende oder beschädigte Nachweise und ausgeschöpfte Inspektionsbudgets sind harte Fehler.

Die Veröffentlichung ersetzt das APKG atomar. Das ist eine Garantie pro Datei; das Schreiben eines separaten JSON-Berichts deiner Anwendung ist ein eigener Vorgang. Der gewöhnliche Build schreibt keine zusätzliche Identitätsdatei: Vollständige versionierte Nachweise sind im verteilten Paket eingebettet.

Ein später Fehler kann nach der Veröffentlichung auftreten, etwa beim Bestätigen der dauerhaften Speicherung. Fehler-Snapshots zeichnen deshalb `PublicationSnapshot`-Fakten auf: `path`, `stage` (`not_published` oder `published`), `temporary` und Dauerhaftigkeit (`confirmed` oder `unconfirmed`). Prüfe diese Angaben, bevor du den Vorgang wiederholst oder meldest, dass keine Datei geschrieben wurde. `PersistError` erhält ebenfalls die Veröffentlichungsfakten und die tatsächliche E/A-Ursache. Ein atomarer Austausch allein garantiert keine Dauerhaftigkeit nach einem Absturz.

Die Snapshot-Serialisierung erhält native Pfade, ohne auf das Dateisystem zuzugreifen. Unicode-Pfade bleiben JSON-Zeichenfolgen. Unix-Pfade ohne gültiges UTF-8 verwenden `{"encoding":"unix_bytes","bytes":[...]}`; Windows-Pfade mit ungepaarten UTF-16-Surrogaten verwenden `{"encoding":"windows_wide","units":[...]}`. Die Arrays enthalten exakt die nativen Bytes bzw. Codeeinheiten. Artefaktpfade bei Erfolg, Veröffentlichungspfade bei Fehlern und Pfaddetails in Binding-Fehlern verwenden diese Darstellung. Diese Werte beschreiben Pfade und behalten keinen Artefaktbesitz. Ein Dateisystem kann einen bestimmten nativen Pfad zurückweisen; die Fakten zum Veröffentlichungsfehler bleiben dennoch serialisierbar.

<!-- section: inspection-limits -->
## Inspektionslimits

Endliche Budgets für Archive, Eintragsanzahl, Entpackung, dekodierte Sammlung und Medien sowie zstd-Fenster gelten unabhängig für Basisversion und Kandidat. Beginne mit `InspectLimits::default()` und erhöhe bei Bedarf gezielt das passende Limit. Null bedeutet nicht unbegrenzt. Diese Zähler sind keine prozessweite Speicher- oder CPU-Sandbox; Medienimporte haben ein separates, vorgelagertes Budget pro Datei.

<!-- section: concurrent-work -->
## Nebenläufige Vorgänge

Rust-Builds leihen unveränderlichen Projektzustand aus. Modelle und Medien mit eigenem Datenbesitz können zwischen Projekten geteilt werden. Das Node-Binding erfasst beim Aufruf von Build oder Vergleich einen Projektsnapshot, sodass später hinzugefügte Inhalte die Anfrage nicht verändern. Halte den Artefaktbesitzer, bis alle Leser fertig sind. Den letzten temporären Besitzer zu schließen, während ein separater Nutzer noch dessen Pfad benötigt, ist ein Fehler der Lebensdauerverwaltung in der Anwendung.

Eine Paketinspektion kann nicht jedes Verhalten im Client bestätigen. Teste Darstellung, Wiedergabe und Aktualisierungsimporte mit den vorgesehenen Anki-Versionen und Einstellungen. Unter [Aktualisierungen](/docs/updates/) findest du die Grenzen bei Lernplanung und Schemaänderungen.

<!-- section: prepared-publication-from-source -->
## Vorbereitete Veröffentlichung

Die vorbereitete Veröffentlichung ist in den veröffentlichten Paketen `0.3.1` enthalten.

`prepare_publication` erstellt, inspiziert und vergleicht einen privaten Kandidaten; `publish` verwendet dieselbe Datei. Optionen und relative Ziele werden bei der Vorbereitung gebunden. Projektänderungen können den Kandidaten nicht verändern. Die Vorbereitung allein ersetzt das Ziel nicht; das Freigeben oder Schließen eines unveröffentlichten Besitzers entfernt den Kandidaten.

Jeder Veröffentlichungsversuch verbraucht den Besitzer, auch bei Richtlinien- oder E/A-Fehlern. Bereite ihn erneut vor, um den Vorgang zu wiederholen oder Richtlinie bzw. Ziel zu ändern. Berichte bleiben lesbar und besitzen keine Dateien. Node und Python bieten idempotentes `close()` und einen Nichtverfügbarkeitsfehler mit dem Grund `closed` oder `consumed`. Ein Schließen nach Beginn der Veröffentlichung bricht diese nicht ab. Die Dauer schließt Wartezeit für die Prüfung aus. Ein später Fehler bei der dauerhaften Speicherung kann auf einen erfolgreichen Austausch folgen; prüfe deshalb die tatsächlichen Veröffentlichungsfakten.

Folge dem [vollständigen Beispiel](/docs/updates/#build-once-review-then-publish) und den API-Referenzen der Sprachen für typisierte Fehler und das Verhalten der Hintergrundarbeit.
