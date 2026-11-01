# Oclusión de imágenes

Genera tarjetas de oclusión de imágenes de Anki desde código: proporciona una imagen y las coordenadas de los rectángulos, y Anki Forge crea una nota con una tarjeta por máscara, lista para importar. Así puedes generar preguntas sobre diagramas por lotes sin dibujar las máscaras en el editor de Anki. La biblioteca no detecta las etiquetas de la imagen; debes proporcionar las coordenadas.

Crea una nota estructurada de oclusión de imágenes a partir de una imagen cuyos datos se conservan y de varios rectángulos. Cada máscara necesita una clave estable que siga identificando la misma región en versiones posteriores. La nota se incorpora a un proyecto mediante la llamada habitual `add(key, note)`. Estos ejemplos usan Rust; Python y Node.js ofrecen el mismo constructor en la [API de Python](python-api.md#media-and-occlusion) y la [API de Node](node-api.md#media-and-image-occlusion).

<!-- section: run-the-example -->
## Ejecuta el ejemplo

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

Abre `target/docs-examples/diagram.apkg` en Anki. El programa ejecutable completo carga la imagen de oclusión del repositorio y comprueba que haya una nota, una tarjeta y un recurso.

<!-- source: anki_forge/examples/docs_workflow.rs#occlusion -->
```rust
fn occlusion(output: &Path, diagram: &Path) -> anyhow::Result<()> {
    let image = Media::file(diagram)?;
    let note = Note::image_occlusion(image)
        .mask(Mask::rect("region-1", 10, 10, 40, 25))
        .build()?;
    let mut project = Project::new("docs-diagram")?.default_deck("Diagram");
    project.add("diagram", note)?;
    let built = project.build(BuildOptions::to(output.join("diagram.apkg")))?;
    verify(&built, 1, 1, 1)?;
    Ok(())
}
```
<!-- /source -->

<!-- section: multiple-masks-and-modes -->
## Varias máscaras y modos

Este ejemplo independiente necesita `fixtures/occlusion.png`, una imagen de al menos 80 × 80 píxeles una vez decodificada:

```rust
use ankiforge::{BuildOptions, Media, Note, Project};
use ankiforge::note::{Mask, OcclusionMode};

fn main() -> anyhow::Result<()> {
    let note = Note::image_occlusion(Media::file("fixtures/occlusion.png")?)
        .mask(Mask::rect("nucleus", 10, 10, 20, 20))
        .mask(Mask::rect("wall", 50, 50, 20, 20))
        .mode(OcclusionMode::HideOneGuessOne)
        .build()?
        .field("header", "Cell structure")
        .field("back_extra", "Review the diagram.");
    let mut project = Project::new("cell-diagram")?;
    project.add("cell", note)?;
    let output = project.build(BuildOptions::to("cell-diagram.apkg"))?;
    assert_eq!(output.report().counts().cards, 2);
    Ok(())
}
```

`HideAllGuessOne` es el modo predeterminado; también se admite `HideOneGuessOne`. Cada máscara con clave produce su propia tarjeta Cloze. Las coordenadas son valores finitos en píxeles sobre la imagen mostrada, después de aplicar la orientación EXIF. La biblioteca decodifica por completo PNG, JPEG, GIF, WebP y BMP; el límite del búfer decodificado es de 256 MiB, independiente del límite de importación de recursos comprimidos. Los rectángulos deben tener dimensiones positivas, quedar dentro de la imagen y usar claves únicas que no estén vacías.

El constructor valida la imagen y las máscaras antes de devolver una `Note`. Usa las asignaciones normales de `field` para `header`, `back_extra` y `comments`; los campos generados `image` y `occlusion` están reservados y no se pueden sobrescribir.

Al actualizar a partir de una distribución anterior, las claves de las máscaras conservan los números ordinales de las tarjetas aunque cambie su orden. Las máscaras eliminadas reservan sus números; restaurar la misma clave reutiliza el número ordinal. Las claves nuevas nunca heredan la identidad de otra máscara. Se admite la asignación del 1 al 500, y el agotamiento provoca un error explícito. Eliminar o añadir máscaras aparece en [el análisis de riesgos de actualización](/docs/updates/); omitir una tarjeta del APKG no la elimina de la colección existente de quien estudia.
