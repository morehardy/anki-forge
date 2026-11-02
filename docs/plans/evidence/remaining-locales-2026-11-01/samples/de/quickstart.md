# Anki-Karten mit Rust erzeugen

Verwende Rust ab Version 1.92. Für eine andere Programmiersprache wähle [Node / TypeScript oder Python](/docs/languages/).

<!-- section: install -->
## Installieren

```sh
cargo new anki-deck
cd anki-deck
cargo add ankiforge@0.3.1
```

<!-- section: export-your-first-deck -->
## Deinen ersten Stapel exportieren

Speichere dieses vollständige Programm als `src/main.rs`:

<!-- source: anki_forge/examples/target_api_basic.rs -->
```rust
use ankiforge::{BuildOptions, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", "hello"))?;
    let output = project.build(BuildOptions::to("spanish.apkg"))?;
    println!("{}", output.artifact().path().display());
    Ok(())
}
```
<!-- /source -->

```sh
cargo run
```

Das Programm schreibt die dauerhaft gespeicherte Datei `spanish.apkg`. Öffne sie in Anki, importiere sie und lerne **hola → hello** im Stapel **Spanish**. Für den Export muss Anki nicht installiert sein.

Der Namensraum `spanish` und der stabile Schlüssel `es:hola` kennzeichnen diese Notiz auch nach späteren Inhaltsänderungen. Du kannst auch [das Basic-Beispiel der Website herunterladen](/generated/spanish.apkg).

<!-- section: continue -->
## Nächste Schritte

Die [Anleitung für JSON zu Anki](/docs/json-to-anki/#rust) zeigt einen vollständigen Ablauf mit Medien und Aktualisierungen.

Ergänze [Bilder und Audio](/docs/media/), [eigene Notiztypen](/docs/custom-notetypes/) oder [Bildverdeckung](/docs/image-occlusion/). Bewahre das ursprünglich verteilte APKG auf, bevor du [eine Aktualisierung vergleichst und veröffentlichst](/docs/updates/). Einzelheiten findest du in der [Rust-API](/docs/rust-api/) und unter [geprüfte Umgebungen](/docs/compatibility/).
