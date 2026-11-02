# Inhalte mit Rust erstellen

Canonical page: https://morehardy.github.io/anki-forge/de/docs/rust-guide/
Language: de
Description: Erstelle Projekte, validiere Notizen und verwalte Medien und Aktualisierungen über die unterstützte Rust-API.

Ein `Project` beschreibt eine Veröffentlichung. Erstelle es mit einem stabilen Namensraum, füge Notizen unter stabilen Schlüsseln hinzu und erzeuge ein APKG. Häufig verwendete Typen importierst du direkt aus `ankiforge`; erweiterte Optionen liegen in den öffentlichen Fachmodulen.

## Projekte und Diagnostik

```rust
use ankiforge::{BuildOptions, Content, Note, Project};

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("jp-core")?
        .default_deck("Japanese::Core");
    project.add("taberu", Note::basic("食べる", Content::html("<b>to eat</b>")))?;
    let output = project.build(BuildOptions::to("jp-core.apkg"))?;
    println!("{} notes", output.report().counts().notes);
    println!("{}", serde_json::to_string_pretty(&output.snapshot())?);
    Ok(())
}
```

Ein erfolgreicher Build liefert `BuildOutput` mit einem garantierten Artefakt. `BuildReport` enthält Beobachtungen: Anzahlen, Diagnosen, verstrichene Zeit und einen gegebenenfalls abgeschlossenen Vergleich. Er enthält weder ein Erfolgs- oder Fehlerergebnis noch Artefaktbesitz. Warnungen machen aus einer erfolgreichen Ausgabe keinen Fehler.

Schemafehler werden beim Abschluss eines Modells geprüft. `Project::add` prüft Notizschlüssel, Feldreferenzen, Pflichtfelder, Stapel, Tags und Abhängigkeitskonflikte atomar. Bei einem Fehler bleibt das Projekt unverändert. Der Build führt die übrigen Karten-, Paket- und Aktualisierungsprüfungen aus. Es gibt weder eine separate veränderliche Registrierung noch einen verpflichtenden zusätzlichen Validierungsschritt.

Fehler stellen eigene Methoden `kind()` und `code()` bereit und implementieren `std::error::Error`. Verwende maschinenlesbare Codes, statt die angezeigte Meldung zu parsen. `BuildError::snapshot()` enthält den tatsächlichen Fehler, Beobachtungen, den Text der Ursachenverkettung und Veröffentlichungsfakten. Standardmäßige Fehler-Wrapper können den ursprünglichen Fehler als Ursache erhalten.

## Modelle und Medien

Ein abgeschlossener `NoteType` ist unveränderlich und gemeinsam nutzbar. Rufe `model.note()` auf, weise Feldern über stabile Schlüssel Werte zu und füge das Ergebnis einem Projekt hinzu. Die Notiz hält ihr Modell; beim Hinzufügen werden das Modell und die Medien automatisch gesammelt. Deklarationen und Vorlagenregeln erklärt [Eigene Notiztypen](https://morehardy.github.io/anki-forge/de/docs/custom-notetypes/).

`Media::file` und `Media::bytes` erzeugen Snapshots mit eigenem Datenbesitz. Nach erfolgreichem Import müssen Quelldateien nicht weiter verfügbar bleiben. `image.image()` und `audio.sound()` erzeugen strukturierte Inhalte; `Content::sequence` kombiniert sie mit Text oder HTML. Mediendateien für direktes HTML, CSS oder Skripte musst du mit `builder.asset` oder `project.add_asset` ausdrücklich einbinden. Siehe [Medien](https://morehardy.github.io/anki-forge/de/docs/media/) und [Vorlagenpakete](https://morehardy.github.io/anki-forge/de/docs/templates/).

## Besitz der Ausgabedateien

Wähle `BuildOptions::to(path)` für eine persistente Ausgabe oder `BuildOptions::temporary()` für eine temporäre Datei mit kontrollierter Lebensdauer. Halte die Ausgabe oder einen Klon von `output.artifact()`, solange du auf einen temporären Pfad zugreifst. Der letzte Besitzer entfernt die Datei. `persist_to(path)` liefert ein persistentes Artefakt; bei einem Fehler bleibt der ursprüngliche Besitzer nutzbar. JSON-Snapshots enthalten Pfade, besitzen aber keine Dateien.

## Stabile Notizidentität

Verwende mit `project.add(key, note)` einen dauerhaften Schlüssel des Quelldatensatzes. Projektnamensraum, Modellschlüssel, Feldschlüssel und Vorlagenschlüssel sind unabhängig von Anzeigenamen. Inhaltsänderungen erzeugen keine neue logische Notiz. Doppelte Schlüssel führen zu einem Fehler, statt eine Notiz zu überschreiben. Speichere die gewählten Schlüssel und verwende sie erneut, wenn Quelldatensätze umgeordnet oder bearbeitet werden.

## Bereits verteilte Stapel aktualisieren

Verwende das ursprünglich verteilte APKG der Vorgängerversion mit `update_from`. Seine vollständigen eingebetteten Nachweise erhalten Identitäts- und Revisionszuordnungen. Der unabhängige Aufruf `project.compare(CompareOptions::against(path))` analysiert den Kandidaten und meldet, ob die Richtlinie eine Veröffentlichung zulassen würde. Der [Ablauf mit zwei Versionen](https://morehardy.github.io/anki-forge/de/docs/updates/) erklärt risikoreiche Änderungen, Grenzen des Clientimports und ausdrückliche Freigaben.

Im Repository führen diese Beispiele dieselbe öffentliche API aus:

```sh
cargo run --locked -p ankiforge --example target_api_basic
cargo run --locked -p ankiforge --example target_api_custom_notetype
cargo run --locked -p ankiforge --example target_api_media
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

### Fehler beim Hinzufügen und Medienkategorien

`Project` verwendet einen stabilen Namensraum und einen optionalen Standardstapel; es hat keinen Anzeigetitel. `NoteType`, `Field` und `Template` unterstützen weiterhin Anzeigenamen. `Project::add` ist atomar. Prüfe `AddError::context()` für Notiz- und Modellschlüssel sowie `note::AddTarget` für die ursprüngliche Stelle. `detail()` liefert `note::AddDetail` für Konflikte oder unpassende Medienverwendung. Behandle diese nicht abschließenden Enums mit `..` bei Datenvarianten und einem Auffangzweig. Der `content_path` eines Feldes ist `None` für das Feld, `Some([])` für seine Wurzel oder eine Folge von Sequenzindizes für einen verschachtelten Knoten. Bytebereiche von Trennzeichen zeigen in das ursprüngliche UTF-8-Text-/Html-Blatt.

`Media::image()` und `sound()` bleiben unfehlbar. Beim Hinzufügen wird für Bilder `image/*` verlangt; Audioreferenzen akzeptieren `audio/*` oder `video/*`. Andere Kategorien führen zu `NOTE.MEDIA_USAGE_INVALID`. Geprüft wird der gespeicherte MIME-Typ, nicht die Dekodierung oder Wiedergabe. Direktes HTML und ausdrücklich deklarierte Medien behalten ihr bisheriges Verhalten. Das Umbenennen beim Export ändert den MIME-Typ nicht: Ein PNG mit dem Namen `wrong.mp3` kann als Bild hinzugefügt werden, aber bei der unabhängigen Inhalts-/Dateiendungsprüfung des Builds mit `MEDIA.DECLARED_MIME_MISMATCH` scheitern.

Tool-Rezepte verlangen jetzt `ankiforge-project-v2` ohne `name` auf oberster Ebene; v1 wird zurückgewiesen. Das Vertragspaket hat Version 2.1.0; die Paketidentität bleibt `ankiforge-identity-v1`.
