# Eigene Notiztypen

Schließe die Definition eines Modells einmal ab und erzeuge anschließend Notizen daraus. Der zurückgegebene `NoteType` ist unveränderlich und lässt sich in Funktionen oder mehreren Projekten wiederverwenden. Eine Notiz hält ihr Modell und kann daher nicht versehentlich auf einen unregistrierten Modellnamen verweisen.

<!-- section: executable-example -->
## Ausführbares Beispiel

Das Beispiel definiert stabile Schlüssel und davon getrennte Anzeigenamen, weist Werte über Schlüssel zu und verwendet dieselben Schlüssel in den Vorlagen.

<!-- source: anki_forge/examples/target_api_custom_notetype.rs -->
```rust
use ankiforge::schema::GenerationRule;
use ankiforge::{BuildOptions, Field, NoteType, Project, Template};

fn main() -> anyhow::Result<()> {
    let vocab = NoteType::builder("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("expr").name("Expression").sort().required())
        .field(Field::new("meaning").name("Meaning").required())
        .template(
            Template::new("recognition")
                .name("Recognition")
                .front("{{expr}}")
                .back("{{FrontSide}}<hr id=\"answer\">{{meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .build()?;
    let mut project = Project::new("jp-core")?.default_deck("Japanese::Core");
    project.add(
        "taberu",
        vocab
            .note()
            .field("expr", "食べる")
            .field("meaning", "to eat"),
    )?;
    project.build(BuildOptions::to("jp-core.apkg"))?;
    Ok(())
}
```
<!-- /source -->

Führe im Repository `cargo run --locked -p ankiforge --example target_api_custom_notetype` aus. Öffne `jp-core.apkg` in Anki.

<!-- section: keys-and-names -->
## Schlüssel und Namen

`Field::new` und `Template::new` erwarten stabile Schlüssel. `.name` legt einen Anzeigenamen fest; andernfalls entspricht der Name dem Schlüssel. Aus einem Namen wird kein Slug abgeleitet. Leere Schlüssel, Schlüssel, die nur aus Leerraum bestehen, Schlüssel mit führendem oder nachgestelltem Leerraum sowie widersprüchliche oder reservierte Schlüssel führen beim Abschluss des Modells zu einem Fehler. Namen können chinesischen oder anderen Unicode-Text enthalten, ohne die Identitätsschlüssel zu ändern.

Vorlagen binden Schlüssel ein und werden in Anki-Feldnamen übersetzt. Das gilt für Vorder- und Rückseiten, Vorlagen des Kartenbrowsers, bedingte Abschnitte, Filter und Cloze. Sonstiges HTML, Leerraum und Skripttext bleiben erhalten. Unbekannte Referenzen führen zu einem Fehler mit dem Bytebereich in der Quelle. Feldzuweisungen akzeptieren nur Schlüssel; Leerraum wird nicht stillschweigend entfernt und Groß- und Kleinschreibung nicht automatisch vereinheitlicht.

<!-- section: validation-and-generation -->
## Validierung und Kartenerzeugung

`required()` weist fehlende oder leere Notizinhalte zurück. Felder ohne diese Einstellung dürfen fehlen und werden leer dargestellt. `sort()` wählt das Sortierfeld; höchstens ein Feld darf diese Einstellung tragen. Die Kartenerzeugung verwendet Ankis Standardregel, sofern eine Vorlage nicht ausdrücklich `GenerationRule::all` oder `any` festlegt. Diese Regeln verweisen auf Feldschlüssel. `builder.cloze_field(key)` aktiviert die Cloze-Semantik für ein eigenes Modell.

`builder.asset(media)` deklariert eine Abhängigkeit mit eigenem Datenbesitz für CSS, Schriftarten, Skripte oder direkt eingebundenes HTML. Typisierte Bild- und Audioinhalte auf Notizen sammeln Medien automatisch. Ausdrücklich deklarierte Medien bleiben auch dann in der Ausgabe, wenn keine statische Referenz gefunden wird.

Derselbe Modellschlüssel darf innerhalb eines Projekts nur mit derselben Definition wiederverwendet werden. Eine widersprüchliche Definition führt beim Hinzufügen einer Notiz zu einem atomaren Fehler. Änderungen an Anzeigenamen, Feldern oder Vorlagen in späteren Veröffentlichungen benötigen eine [Aktualisierungsanalyse](/docs/updates/), auch wenn ihre stabilen Schlüssel gleich bleiben.

Für wiederverwendbare Dateien und Medien verwende ein [Vorlagenpaket](/docs/templates/).
