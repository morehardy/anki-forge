# Rust-API

Häufig verwendete Werte sind direkt im Crate verfügbar: `Project`, `Note`, `NoteType`, `Field`, `Template`, `Content`, `Media`, `BuildOptions` und `BuildOutput`. Die öffentlichen Fachmodule heißen `note`, `schema`, `media`, `build`, `update` und `diagnostics`. Repository-Werkzeuge bilden eine separate, durch Features freigeschaltete Schnittstelle; Anwendungen benötigen sie nicht.

<!-- section: authoring -->
## Inhalte erstellen

| Einstieg | Aufgabe |
| --- | --- |
| `Project::new(namespace)` | Einen stabilen Veröffentlichungsnamensraum validieren |
| `project.default_deck(name)` | Das Standardziel festlegen |
| `project.add(key, note)` | Notiz, Modell und Medien atomar sammeln |
| `project.add_asset(media)` | Eine ausdrückliche Ressource für direktes HTML/CSS/Skripte einbinden |
| `Note::basic(front, back)` / `Note::cloze(text)` | Eine Notiz mit ihrem eingebauten Modell erzeugen |
| `model.note().field(key, content)` | Eine eigene Notiz mit ihrem unveränderlichen Modell erzeugen |
| `note.deck(name).tag(tag)` / `.tags(iterable)` | Eine Notiz vor dem Hinzufügen konfigurieren |
| `NoteType::builder(key)...build()` | Ein unveränderliches Modell validieren und abschließen |
| `NoteType::from_bundle(path)` | Ein Modell und seine Medien mit eigenem Datenbesitz aus bundle-v2 laden |

`Field::new(key).name(display)` und `Template::new(key).name(display)` trennen Identität und Darstellung. Vorlagen binden Feldschlüssel ein, einschließlich Filtern, bedingten Abschnitten und Vorlagen des Kartenbrowsers. `Field::required()` prüft auf nicht leere Inhalte; `Field::sort()` wählt das Sortierfeld. `schema::GenerationRule::all` und `any` verwenden Feldschlüssel. `builder.cloze_field(key)` deklariert ein Cloze-Modell.

`schema::FieldKey` und `TemplateKey` sind unterschiedliche Typen. Zeichenfolgenliterale, eigene und geliehene Strings, `Cow<str>` sowie geliehene oder eigene Schlüssel lassen sich kombinieren. Konvertierungen erhalten den ursprünglichen Wert; der Schemaabschluss validiert die Schlüssel. Ein Vorlagenschlüssel wird nicht implizit als Feldschlüssel akzeptiert.

<!-- section: content-and-media -->
## Inhalte und Medien

Alle gewöhnlichen Zeichenfolgen werden zu `Content::text`. HTML erfordert `Content::html`. `Content::sequence` kombiniert semantische Werte bis zur Darstellung beim Build. `Media::file(path)` und `Media::bytes(bytes, mime)` besitzen Snapshots; ihre `with_limits`-Varianten wenden vor dem Lesen ein Budget `media::MediaLimits { max_bytes }` an. Der Standard beträgt 256 MiB pro Mediendatei. `with_export_name(name)` liefert einen umbenannten Wert mit demselben Snapshot; bestehende Klone behalten ihre Namen.

`media.image()` und `media.sound()` behalten Ressourcenabhängigkeiten. Feste Dateinamen müssen portabel und konfliktfrei sein. Ausdrücklich deklarierte Medien werden auch ohne statisch sichtbare Referenz aufgenommen. Siehe [Medien](/docs/media/).

`Note::image_occlusion(media)` liefert `note::ImageOcclusionBuilder`. `.mask(Mask::rect(key, x, y, width, height)).mode(mode).build()` validiert dekodierte Bildabmessungen, Rechtecke und eindeutige stabile Schlüssel. `HideAllGuessOne` und `HideOneGuessOne` werden unterstützt. Siehe [Bildverdeckung](/docs/image-occlusion/).

<!-- section: build-options -->
## Build-Optionen

`project.build(BuildOptions::to(path))` veröffentlicht ein persistentes APKG. `BuildOptions::temporary()` liefert eine temporäre Ausgabe mit kontrollierter Lebensdauer. Beide akzeptieren `.update_from(previous_apkg)`, `.inspect_limits(limits)` und `.update_policy(policy)`. Eine Richtlinie ohne Basisversion ist ein Konfigurationsfehler. Die Reihenfolge der Setter ändert diese Regel nicht.

`build::InspectLimits::default()` liefert endliche Budgets; die öffentlichen Felder können verändert werden. Alle Werte sind `u64`; `0` bedeutet ein Budget von null, nicht unbegrenzt.

| Feld | Standard |
| --- | --- |
| `max_archive_bytes` | 2 GiB |
| `max_entries` | 100.000 |
| `max_central_directory_bytes` | 16 MiB |
| `max_zip_entry_bytes` / `max_zip_total_bytes` | 1 GiB / 4 GiB |
| `max_meta_bytes` / `max_media_map_bytes` | 64 KiB / 16 MiB |
| `max_identity_bytes` | 64 MiB |
| `max_collection_bytes` / `max_media_bytes` | 512 MiB / 256 MiB |
| `max_decoded_total_bytes` / `max_zstd_window_bytes` | 4 GiB / 64 MiB |

Limits gelten unabhängig für die Inspektion von Basisversion und Kandidat. Eine fehlgeschlagene Prüfung meldet Ressource, Limit, beobachtete Menge und, sofern verfügbar, den betroffenen Eintrag. Ein Limit zu erhöhen ist eine ausdrückliche Handlung des Aufrufers; der Medienimport hat sein eigenes vorgelagertes Budget.

<!-- section: build-compare-and-output -->
## Erzeugen, vergleichen und ausgeben

`BuildOutput::artifact()` liefert eine Referenz auf ein `build::ApkgArtifact` mit eigenem Dateibesitz. `BuildOutput::report()` liefert Beobachtungen. Artefaktklone teilen sich die Lebensdauer temporärer Dateien; `persist_to` liefert einen persistenten Besitzer oder einen `build::PersistError` mit Veröffentlichungsfakten und zugrunde liegender E/A-Ursache.

`project.compare(update::CompareOptions::against(path))` liefert einen abgeschlossenen `ComparisonReport`, auch wenn Befunde mit hohem Risiko die Richtlinie blockieren. Er bietet `findings()`, `highest_risk()`, `policy()`, `diagnostics()` und `snapshot()`. `UpdatePolicy::default()` blockiert High/Critical; `.fail_on(level)` ändert den Schwellenwert, und `.allow(RiskCode)` akzeptiert eine vollständige registrierte Risikokategorie. Fehlende oder beschädigte Identitätsnachweise und andere harte Fehler lassen sich damit nicht akzeptieren.

<!-- section: reports-and-errors -->
## Berichte und Fehler

`BuildReport::snapshot()` liefert `build::json::ReportSnapshot`: Beobachtungen ohne Erfolg-/Fehlerstatus oder Dateibesitz. `BuildOutput::snapshot()` und `BuildError::snapshot()` liefern `build::json::BuildSnapshot` mit tatsächlichem Ergebnis und Bericht. `update::json::ComparisonSnapshot` trennt ursprüngliche Befunde von der Richtlinienbewertung. Alle Snapshot-DTOs implementieren `serde::Serialize` und besitzen keine Dateien. Ein Erweiterungstrait ist nicht erforderlich.

Pfadfelder in Snapshots behalten in Rust ihre nativen `PathBuf`-Werte. Ihre JSON-Form ist bei gültigem Unicode eine Zeichenfolge, sonst eine verlustfreie native Kodierung: `{"encoding":"unix_bytes","bytes":[...]}` für Unix-Namen ohne gültiges UTF-8 und `{"encoding":"windows_wide","units":[...]}` für Windows-Namen mit ungepaarten UTF-16-Surrogaten. Das gilt sowohl für Artefaktpfade bei Erfolg als auch für Veröffentlichungspfade bei Fehlern. Um einen JSON-Pfad auf seiner ursprünglichen Plattform wiederherzustellen, deserialisiere ihn als `build::json::PathSnapshot` und rufe `into_path_buf()` auf; kodierte Pfade anderer Plattformen werden zurückgewiesen. `PathSnapshot::new(path)` liefert dieselbe Darstellung, wenn du den nativen Pfad eines Fehlers selbst serialisierst.

Fehler liegen bei ihren Vorgängen: Schema- und Paketfehler in `schema`, Fehler beim Hinzufügen und bei Bildverdeckung in `note`, Medienfehler in `media`, Build-/Speicherfehler in `build` und Vergleichs-/Richtlinienfehler in `update`. Jeder bietet `kind()` und `code()`, implementiert `std::error::Error + Send + Sync + 'static` und erhält die tatsächlichen zugrunde liegenden Fehler.

<!-- section: addition-context -->
## Kontext beim Hinzufügen

Fehlgeschlagenes Hinzufügen lässt das Projekt unverändert. Strukturierter Kontext erfasst die ursprünglichen Notiz-/Modellschlüssel und das Ziel sowie optionale Details zu Konflikten oder Medienverwendung. Rust bietet `note::{AddContext, AddTarget, AddDetail}` über `AddError::context()` / `detail()`; die SDKs bieten `AddError.details` mit typisiertem Kontext und Details. Node-Felder verwenden camelCase und sind tief eingefroren; Python verwendet snake_case-TypedDicts. Tags behalten ihren Einfügeindex; Feldpositionen behalten ursprüngliche Sequenzindizes (null/None für das Feld, [] für seine Wurzel) und optionale UTF-8-Bytebereiche. Fehler des Standardstapels beim Build haben keine Notiz-/Modellschlüssel und bleiben in der Ursachenverkettung verfügbar.

Typisierte Bilder verlangen beim Hinzufügen den MIME-Typ image/*; Audioreferenzen akzeptieren audio/* oder video/*. Andere Kategorien lösen `NOTE.MEDIA_USAGE_INVALID` aus. Konstruktoren bleiben unfehlbar; ausdrücklich deklarierte Medien und direktes HTML sind nicht betroffen. Die Prüfung verwendet den beim Import gespeicherten MIME-Typ und bestätigt keine Wiedergabefähigkeit. Die unabhängige MIME-/Endungsprüfung des Builds gilt weiterhin. Die vollständige Ziel-/Detailtabelle findest du im [Entwurf](https://github.com/morehardy/anki-forge/blob/main/docs/plans/2026-09-28-rust-api-validation-and-errors-design.md).

<!-- section: review-once-publish-once -->
## Einmal prüfen, einmal veröffentlichen

Die API für vorbereitete Veröffentlichungen ist im veröffentlichten Paket `0.3.1` enthalten.

```rust,no_run
use ankiforge::{BuildOptions, Project};
# fn publish(project: &Project) -> Result<(), Box<dyn std::error::Error>> {
let prepared = project.prepare_publication(
    BuildOptions::to("next.apkg").update_from("previous.apkg"),
)?;
let report = prepared.report().clone(); // observations; owns no file
if report.comparison().is_some_and(|c| !c.policy().allows_publication()) {
    drop(prepared); // deletes the unpublished candidate; reprepare with a new policy
    return Ok(());
}
match prepared.publish() { // consumes the owner on every outcome
    Ok(output) => println!("Published {}", output.artifact().path().display()),
    Err(error) => {
        // Replacement may have succeeded even if durability confirmation failed.
        eprintln!("{:?}", error.publications());
        return Err(error.into());
    }
}
# Ok(())
# }
```

Die Vorbereitung führt eine vollständige Inspektion innerhalb der Budgets und einen Vergleich durch. Vor `publish` wird kein Ziel veröffentlicht. `publish` verwendet denselben privaten Kandidaten und prüft erneut Aliase der Basisversion. Relative Pfade werden beim Aufruf der Vorbereitung gebunden. Projektänderungen können den Kandidaten nicht verändern. Die Berichtsdauer schließt die Prüfzeit aus. Das Freigeben eines unveröffentlichten Besitzers räumt den Kandidaten auf; für temporäre veröffentlichte Ausgaben gilt die gewöhnliche Aufräumregel beim letzten Artefaktbesitzer. Es gibt weder eine Wiederherstellung aus JSON noch einen erneuten Versuch mit einem bereits verbrauchten Besitzer. Native Builds lassen das ungenutzte Staging-Manifest weg, behalten aber eingebettete Identitätsnachweise und Medienvalidierung bei. Interne Staging-Werkzeuge erhalten weiterhin ihr vollständiges Manifest und ihren Fingerabdruck.
