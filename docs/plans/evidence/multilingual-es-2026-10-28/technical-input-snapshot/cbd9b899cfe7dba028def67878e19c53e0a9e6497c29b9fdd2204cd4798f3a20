Usa Rust 1.92 o posterior. Si prefieres otro lenguaje, elige [Node / TypeScript o Python](/docs/languages/).

<!-- section: install -->
## Instalación

```sh
cargo new anki-deck
cd anki-deck
cargo add ankiforge@0.3.1
```

<!-- section: export-your-first-deck -->
## Exporta tu primer mazo

Guarda este programa completo como `src/main.rs`:

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

El programa escribe el archivo persistente `spanish.apkg`. Ábrelo en Anki, impórtalo y estudia **hola → hello** en el mazo **Spanish**. No necesitas instalar Anki para exportar el archivo.

El espacio de nombres `spanish` y la clave estable `es:hola` identifican esta nota aunque cambie su contenido. También puedes [descargar el ejemplo Basic del sitio](/generated/spanish.apkg).

<!-- section: continue -->
## Continúa

Sigue el [tutorial de JSON a Anki](/docs/json-to-anki/#rust) para completar un flujo con recursos multimedia y actualizaciones.

Añade [imágenes y audio](/docs/media/), [tipos de nota personalizados](/docs/custom-notetypes/) u [oclusión de imágenes](/docs/image-occlusion/). Conserva el APKG original de distribución antes de [comparar y publicar una actualización](/docs/updates/). Consulta la [API de Rust](/docs/rust-api/) o los [entornos verificados](/docs/compatibility/) para obtener más detalles.
