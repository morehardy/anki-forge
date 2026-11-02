# Anki-Karten um Bilder und Audio ergänzen

Canonical page: https://morehardy.github.io/anki-forge/de/docs/media/
Language: de
Description: Importiere mit Rust, Python oder Node.js Dateien, kombiniere Karteninhalte und verwalte Exportnamen, MIME-Typen und Medienbudgets.

Bündle Bilder und Audio in erzeugten Anki-Karten, etwa Vokabelkarten mit Aussprache oder Diagrammfragen. Anki Forge kopiert die bereitgestellten Dateien in das APKG. Zum Erzeugen von Audio oder Bildern, etwa durch Sprachsynthese, benötigst du ein separates Werkzeug. Die Beispiele verwenden Rust; der unten beschriebene Import mehrerer Mediendateien zeigt auch die Python- und Node.js-Aufrufe.

Ein `Media`\-Wert besitzt einen unveränderlichen Snapshot. Importiere Dateien mit `Media::file` oder native Bytes mit einem ausdrücklichen MIME-Typ über `Media::bytes`. Nach erfolgreichem Import können Änderungen oder das Löschen der Quelldatei die Mediendatei nicht mehr verändern. Klone teilen sich den Speicher und können in mehreren Projekten verwendet werden.

## Den Medienablauf ausführen

Führe im Stammverzeichnis des Repositorys Folgendes aus:

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
cargo run --locked -p ankiforge --example target_api_media
```

Der Ablauf verwendet PNG-/WAV-Testdateien des Repositorys und schreibt `media.apkg` mit einer Notiz, einer Karte und drei Mediendateien. Das zweite Beispiel erzeugt selbst ein kleines Bild und Audio, bindet über CSS referenzierte sowie ausdrücklich deklarierte Medien ein und schreibt `spanish-media.apkg`.

```rust
fn media(output: &Path, image: &Path, sound: &Path) -> anyhow::Result<()> {
    let image = Media::file(image)?.with_export_name("cell.png")?;
    let sound = Media::file(sound)?;
    let mut project = Project::new("docs-media")?;
    project.add("cell", Note::basic(image.image(), sound.sound()))?;
    let css = Media::bytes(b".card { color: navy; }".to_vec(), "text/css")?
        .with_export_name("course.css")?;
    project.add_asset(css)?;
    let built = project.build(BuildOptions::to(output.join("media.apkg")))?;
    verify(&built, 1, 1, 3)?;
    Ok(())
}
```

## Inhalte kombinieren und Medien ausdrücklich einbinden

Dieses vollständige Programm verwendet `fixtures/pixel.png`, `fixtures/silence.wav` und `fixtures/labels.woff`. Übergib in deiner Anwendung deine eigenen Quellpfade. Die Beispielschrift ist die weiterverteilbare Testschrift des Repositorys.

```rust
use ankiforge::{BuildOptions, Content, Field, Media, NoteType, Project, Template};

fn main() -> anyhow::Result<()> {
    let image = Media::file("fixtures/pixel.png")?.with_export_name("badge.png")?;
    let sound = Media::file("fixtures/silence.wav")?;
    let font = Media::file("fixtures/labels.woff")?.with_export_name("labels.woff")?;
    let model = NoteType::builder("media-card")
        .field(Field::new("front"))
        .field(Field::new("back"))
        .template(Template::new("card")
            .front(r#"{{front}}<img src="badge.png">"#)
            .back("{{FrontSide}}<hr>{{back}}"))
        .css("@font-face { font-family: labels; src: url('labels.woff'); } .card { font-family: labels; }")
        .asset(font)
        .asset(image.clone())
        .build()?;
    let mut project = Project::new("media-guide")?;
    project.add("cell", model.note()
        .field("front", Content::sequence([Content::text("Cell "), image.image()]))
        .field("back", sound.sound()))?;
    let output = project.build(BuildOptions::to("media-guide.apkg"))?;
    assert_eq!(output.report().counts().media, 3);
    Ok(())
}
```

Typisierte `image()`\- und `sound()`\-Werte behalten Abhängigkeiten bis zum Export. Von Hand erstellte HTML-/CSS-/Skriptreferenzen benötigen `builder.asset(media)` oder `project.add_asset(media)`; die Bibliothek leitet keine Dateiinhalte aus beliebigen Zeichenfolgen ab. Eine ausdrücklich deklarierte Mediendatei wird auch ohne statisch erkennbare Referenz aufgenommen.

Übergib `with_export_name` den wörtlichen Dateinamen. Typisierte Audioreferenzen maskieren HTML-Entitäten bei der Darstellung, sodass ein Name wie `tone&copy;.mp3` weiterhin genau diese Datei bezeichnet. Maskiere den Dateinamen nicht vorab. Prozentzeichen und Fragmentzeichen in Audiodateinamen bleiben wörtlich erhalten; die Namen sind keine URL-Pfade.

## Namen, MIME-Typen und Budgets

Standardmäßige Exportnamen werden aus dem Inhalt abgeleitet und sind vom Quellpfad unabhängig. Verwende `with_export_name` vor dem Erzeugen von Referenzen, wenn du einen festen Dateinamen benötigst. Das Umbenennen eines Klons benennt frühere Klone oder Inhaltsknoten nicht um.

Namen dürfen keine Pfade, Steuerzeichen oder nicht portablen reservierten Namen enthalten. Konfliktprüfungen verwenden Unicode NFC und eine Vereinheitlichung der Groß-/Kleinschreibung: Auch Varianten, die sich nur in Schreibweise oder Normalisierung unterscheiden, verursachen Konflikte. Exakt identische Name-Inhalt-Paare sind idempotent; unterschiedliche Inhalte unter demselben Namen führen zu einem Fehler ohne teilweise Registrierung.

Der vollständige Exportdateiname ist einschließlich Endung auf 255 UTF-8-Bytes begrenzt. Die JSON-Schemas für Projekte und Vorlagenpakete prüfen Dateinamensyntax und Zeichenanzahl; die Lader prüfen zusätzlich dieses Bytebudget. Beispielsweise erfüllen 128 Wiederholungen von `é` die Zeichengrenze des Schemas, belegen aber 256 Bytes und führen beim Laden zu `MEDIA.EXPORT_NAME_INVALID`. Verwende `tools::load_project` oder `NoteType::from_bundle`, um vor dem Build die vollständige Eingabe zu validieren. `tools::load_project` erfordert das Feature `internal-tools` und gehört zu den internen Repository-Werkzeugen, nicht zur unterstützten API für nutzende Anwendungen.

MIME-Syntax und erkennbarer Inhalt müssen zusammenpassen. Für native Bytes gibt es keine Inline-Beschränkung auf 64 KiB. Das Standard-Importbudget beträgt 256 MiB pro Mediendatei; `file_with_limits` und `bytes_with_limits` akzeptieren vor dem Lesen `MediaLimits { max_bytes }`. Große Snapshots können in temporären Speicher ausgelagert werden, der beim Wegfall des letzten Besitzers aufgeräumt wird. Der Snapshot-Besitz verspricht keine atomare Dateisystemsicht auf eine Quelle, die während des Imports gleichzeitig verändert wird.

Projekt-JSON wendet dasselbe Standardbudget pro Mediendatei beim Dekodieren eines Inline-Arrays `bytes` an. Es stoppt beim ersten überschüssigen Byte, ohne den Rest des Arrays zu sammeln, und erhält einen `MediaError` mit Limit und beobachteter Byteanzahl. Die Anzahl bei der Zurückweisung ist eine Untergrenze, nicht die Gesamtgröße des ungelesenen Arrays.

Ein gültiges Medienpaket belegt nicht die Codec-Wiedergabe in jedem Anki-Client. Teste Bilder, Audio und Schriftarten in deinen Zielclients. Siehe [Fehlersuche](https://morehardy.github.io/anki-forge/de/docs/troubleshooting/#media-and-templates).

### Typisierte Bilder und Audioreferenzen verwenden

Konstruktoren bleiben unfehlbar. `Project::add` prüft den beim Import gespeicherten MIME-Typ: Bilder benötigen `image/*`, Audioreferenzen akzeptieren `audio/*` und `video/*`. `application/octet-stream`, Text, Schriftarten und PDF scheitern bei typisierter Verwendung mit `NOTE.MEDIA_USAGE_INVALID`; registriere sie als ausdrückliche Medien für direktes HTML/CSS. Gültige ausdrückliche Deklarationen unbekannter Bild-/Video-MIME-Typen bestehen die Kategorieprüfung, ohne zu bestätigen, dass ein Decoder oder Player die Bytes unterstützt. Das Umbenennen einer Datei ändert den gespeicherten MIME-Typ niemals. Der Build prüft weiterhin unabhängig die bereitgestellten Bytes und die Exportendung: Ein PNG namens `wrong.mp3` kann als Bild hinzugefügt werden und beim Build mit `MEDIA.DECLARED_MIME_MISMATCH` scheitern.

## Mehrere Mediendateien importieren

Diese Methoden sind im veröffentlichten Paket `0.3.1` enthalten.

Importiere mehrere Dateien mit begrenzter Parallelität, behalte die Eingabereihenfolge bei und halte Mediensnapshots mit eigenem Datenbesitz für den späteren Export:

```rust
use ankiforge::Media;
let images = Media::files(["first.png", "second.png"])?;
```

Node verwendet `await Media.files(['first.png', 'second.png'])`; Python verwendet `Media.files(['first.png', 'second.png'])`.

### Verhalten und Limits beim Import mehrerer Mediendateien

Verwende `Media::files(paths)` in Rust, `await Media.files(paths)` in Node oder `Media.files(paths)` in Python, um Dateien in Eingabereihenfolge mit höchstens vier Arbeitern pro Aufruf zu importieren. Gruppen mit weniger als 16 Dateien werden seriell verarbeitet. Rust bietet außerdem `files_with_limits`; Node und Python akzeptieren dieselben Limits wie `file`. Das Limit gilt pro Mediendatei. Schlägt ein Import fehl, wartet der Aufruf auf bereits begonnene Arbeit, gibt die Snapshots der Gruppe frei und liefert den ersten Fehler in Eingabereihenfolge. Bestehende Medienbesitzer bleiben gültig. Python gibt während des Imports den GIL frei; Node führt die Arbeit außerhalb der Ereignisschleife aus. Einzeldateiimporte bleiben sofortige Snapshots mit eigenem Datenbesitz; für parallele Importe müssen Aufrufer ausdrücklich die Stapelmethode wählen.

Lebende Snapshots teilen sich ein prozessweites Speicherbudget von **64 MiB**. Einzelne Dateien über 1 MiB werden weiterhin in temporäre Dateien gestreamt. Kodierte Medien teilen sich pro Vorbereitung einen bei Bedarf angelegten Pufferpool von **16 MiB** und werden ausgelagert, wenn dieser voll ist. Diese Budgets umfassen keinen sonstigen Arbeitsspeicher und sind keine RSS-Obergrenze; parallele Builds können jeweils einen eigenen Kodierungspool besitzen.
