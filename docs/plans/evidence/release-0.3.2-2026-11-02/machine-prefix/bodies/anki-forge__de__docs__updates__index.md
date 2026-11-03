# Anki-Stapel mit stabilen Notiz-IDs aktualisieren

Canonical page: https://morehardy.github.io/anki-forge/de/docs/updates/
Language: de
Description: Veröffentliche eine neue Stapelversion: Behalte stabile Notiz-IDs bei, verwende das ursprüngliche APKG der Vorgängerversion als Basis und prüfe den Vergleich vor der Verteilung.

Veröffentliche Korrekturen oder neue Karten für einen Anki-Stapel, den du bereits verteilt hast. Stabile Notiz-IDs helfen Anki Forge und Anki dabei, vorhandene Notizen wiederzuerkennen, statt sie als neue Notizen zu behandeln. Was tatsächlich geändert wird, hängt jedoch auch von den Importeinstellungen in Anki und neueren lokalen Bearbeitungen ab.

Behalte den Projektnamensraum sowie die Schlüssel für Notizen, Modelle, Felder und Vorlagen unverändert bei. Erzeuge jede neue Version auf Basis des vorherigen **ursprünglich verteilten APKG**, das vollständige, versionierte Identitätsnachweise enthält. Bewahre das Paket zusammen mit der zugehörigen Quellcoderevision auf.

## Zwei Versionen erzeugen und vergleichen

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

Dies schreibt `spanish-v1.apkg` und `spanish-v2.apkg`, vergleicht die geänderte Antwort und prüft die Anzahlen im aktualisierten Paket. Der ausgeführte Quellcode lautet:

```rust
fn vocabulary(answer: &str) -> anyhow::Result<Project> {
    let mut project = Project::new("docs-spanish-updates")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", answer))?;
    Ok(project)
}

fn updates(output: &Path) -> anyhow::Result<()> {
    let previous = output.join("spanish-v1.apkg");
    let next = output.join("spanish-v2.apkg");
    vocabulary("hello")?.build(BuildOptions::to(&previous))?;
    let project = vocabulary("hello; hi")?;
    let comparison = project.compare(CompareOptions::against(&previous))?;
    ensure!(comparison.policy().allows_publication());
    let built = project.build(BuildOptions::to(&next).update_from(&previous))?;
    verify(&built, 1, 1, 0)?;
    ensure!(built.report().comparison().is_some());
    Ok(())
}
```

`compare` wird auch dann erfolgreich abgeschlossen, wenn seine Befunde eine Veröffentlichung verhindern würden. Lies `comparison.policy().allows_publication()` getrennt vom ursprünglichen Risikograd und den zugehörigen Nachweisen aus. `build(...update_from(...))` führt dieselbe Analyse durch. Blockiert die Richtlinie den Kandidaten, gibt die Methode vor der Veröffentlichung einen `BuildError` zurück.

## Richtlinie und Prüfgrenzen

Dieses vollständige Programm wählt einen strengeren Schwellenwert und verwendet dieselbe Konfiguration für Vergleich und Aktualisierung:

```rust
use ankiforge::{BuildOptions, Note, Project};
use ankiforge::build::InspectLimits;
use ankiforge::update::{CompareOptions, RiskLevel, UpdatePolicy};

fn main() -> anyhow::Result<()> {
    let mut first = Project::new("policy-example")?;
    first.add("cell", Note::basic("Cell?", "Unit of life"))?;
    first.build(BuildOptions::to("policy-v1.apkg"))?;
    let mut next = Project::new("policy-example")?;
    next.add("cell", Note::basic("Cell?", "The basic unit of life"))?;
    let mut limits = InspectLimits::default();
    limits.max_collection_bytes = 1 << 30;
    let policy = UpdatePolicy::default().fail_on(RiskLevel::Medium);
    let comparison = next.compare(CompareOptions::against("policy-v1.apkg")
        .inspect_limits(limits.clone()).update_policy(policy.clone()))?;
    assert!(comparison.policy().allows_publication());
    next.build(BuildOptions::to("policy-v2.apkg").update_from("policy-v1.apkg")
        .inspect_limits(limits).update_policy(policy))?;
    Ok(())
}
```

Die Standardrichtlinie blockiert Befunde der Stufen High und Critical. Nach Prüfung einer bestimmten Kategorie kann die aufrufende Anwendung bewusst `UpdatePolicy::default().allow(RiskCode::NoteRemoved)` wählen. Damit werden **alle Befunde dieser Kategorie** zugelassen; ihre ursprüngliche Stufe und ihre Nachweise bleiben erhalten. Die Freigabe gilt nicht einzeln für ausgewählte Notizen. Eine Freigabe ohne passenden Befund erzeugt eine Warnung, unbekannte Risikocodes werden abgewiesen. Gib nicht automatisch jeden Code frei, den ein Vergleich zurückgibt.

Fehlende oder beschädigte Nachweise, ein abweichender Namensraum, ungültige Kandidatendaten und überschrittene Prüfgrenzen sind Fehler, keine freigebbaren Risikokategorien. Kann die Analyse nicht abgeschlossen werden, gibt der Vergleich einen Fehler mit den bereits gewonnenen Prüfergebnissen zurück. Eine explizite Richtlinie bei einer Neuanlage ohne `update_from` ist unabhängig von der Reihenfolge der Setter ein Konfigurationsfehler.

## Nachweise und Identität

Das Paket enthält `ankiforge-identity.json` im Format `ankiforge-identity-v1`. Es verknüpft Modell- und Konfigurations-IDs, Notiz-GUIDs, historische Feld- und Vorlagenpositionen, Karten- und Maskenordnungsnummern, Revisionen und Medien mit der tatsächlichen Sammlung und den enthaltenen Daten. Ein erneuter Export aus Anki erhält diese Nachweise nicht und ist daher keine geeignete Aktualisierungsbasis. Fehlende Nachweise werden niemals stillschweigend rekonstruiert.

Änderungen an Anzeigenamen und Inhalten erhalten die Identitätsschlüssel. Unveränderte Inhalte behalten die Revision der Basisversion; geänderte Inhalte erhalten eine fortgeschriebene Revision. Das Umordnen von Feldern oder Vorlagen erhält die historischen Zuordnungen. Ausgemusterte Identitäten bleiben reserviert, damit sie bei Wiederherstellung desselben Schlüssels erneut verwendet werden können. Ein Notizschlüssel kann nicht zu einem anderen Modell wechseln; ein Modellschlüssel kann nicht zwischen dem normalen Typ und Cloze wechseln.

## Client-Import und Prüfung vor der Veröffentlichung

Ein APKG ist ein Verteilungspaket, keine Synchronisierung von Löschungen. Wenn Notizen, Felder, Vorlagen oder Masken fehlen, werden die vorhandenen Daten der Lernenden nicht gelöscht. Änderungen am Feld- oder Vorlagenschema und am Sortierfeld gelten als hohes Risiko, weil die Zusammenführungs- und Importeinstellungen von Anki das Verhalten beeinflussen. Schemaänderungen können Modelle kopieren oder die Zeitstempel der Zielnotizen aktualisieren, bevor deren Inhalte verglichen werden.

Teste den ersten Import und Aktualisierungen einer vorhandenen Sammlung auf den von dir unterstützten Clients. Prüfe Felder, GUIDs, Kartenbestände, lokale Änderungen und die bestehende Lernplanung mit den vorgesehenen Einstellungen für Zusammenführung und Notizaktualisierung. Stabile Identitäten allein garantieren weder das Überschreiben neuerer lokaler Änderungen noch die Wiederherstellung gelöschter Verlaufsdaten.

Schreibe Kandidaten an einen anderen Pfad als die archivierte Basisversion. Verteile nach der Prüfung das freigegebene Paket und bewahre es als Nachweis für die nächste Version auf. Angaben zu Dateibesitz und Veröffentlichung findest du unter [Build-Garantien](https://morehardy.github.io/anki-forge/de/docs/build-guarantees/).

## Einmal bauen, prüfen und dann veröffentlichen

Diese API ist im veröffentlichten Paket `0.3.1` enthalten.

Erzeuge den Build einmal, prüfe den Bericht und veröffentliche anschließend dieselbe Kandidatendatei. Wer nach der Prüfung einen zweiten gewöhnlichen Build ausführt, erzeugt einen anderen Kandidaten. Bei der vorbereiteten Veröffentlichung bleibt das tatsächliche APKG an den geprüften Bericht gebunden.

```rust
use ankiforge::{BuildOptions, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", "hello"))?;
    project.build(BuildOptions::to("spanish-original.apkg"))?;

    let mut next = Project::new("spanish")?.default_deck("Spanish");
    next.add("es:hola", Note::basic("hola", "hello; hi"))?;
    let prepared = next.prepare_publication(
        BuildOptions::to("spanish-reviewed.apkg").update_from("spanish-original.apkg"),
    )?;
    println!("{:?}", prepared.report().comparison());
    if prepared
        .report()
        .comparison()
        .is_some_and(|report| report.policy().allows_publication())
    {
        let output = prepared.publish()?;
        println!("{}", output.artifact().path().display());
    }
    Ok(())
}
```

Speichere das vollständige Programm oben im festgelegten Quellcode-Checkout als `anki_forge/examples/docs_publication.rs`. Dieses Beispiel wird durch die Anleitung ergänzt und ist nicht in `1199196` enthalten. Führe anschließend Folgendes aus:

```sh
cargo run --locked -p ankiforge --example docs_publication
```

Das Beispiel schreibt `spanish-original.apkg` und, sofern die Richtlinie die Veröffentlichung erlaubt, `spanish-reviewed.apkg` in das Stammverzeichnis des Repositorys. Die Vorbereitung veröffentlicht noch nichts am Zielpfad. Prüfe die Befunde und teste den Import auf deinem Client, bevor du die Verteilung freigibst. Das Beispiel veröffentlicht nur diese einfache Inhaltsänderung automatisch, wenn seine Richtlinie sie zulässt. Der Benchmark misst diesen Ablauf nicht.

Weitere Informationen: [Build-Lebenszyklus](https://morehardy.github.io/anki-forge/de/docs/build-guarantees/#prepared-publication-from-source), [Rust-API](https://morehardy.github.io/anki-forge/de/docs/rust-api/#review-once-publish-once), [Node-API](https://morehardy.github.io/anki-forge/de/docs/node-api/#prepare-review-and-publish) und [Python-API](https://morehardy.github.io/anki-forge/de/docs/python-api/#prepare-review-and-publish).
