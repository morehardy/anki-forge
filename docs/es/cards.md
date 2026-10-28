# Tarjetas Basic y Cloze

Una nota Basic tiene anverso y reverso. Una nota Cloze genera una tarjeta por cada número de omisión admitido. Este ejemplo crea dos notas y tres tarjetas.

Desde la raíz del repositorio:

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

Abre `target/docs-examples/cards.apkg` en Anki. La pregunta de la tarjeta Basic es **hola**; las tarjetas Cloze ocultan **Madrid** y **Spain** por separado.

<!-- section: author-the-notes -->
## Escribe las notas

Esta función se extrae del [programa ejecutable completo](https://github.com/morehardy/anki-forge/blob/main/anki_forge/examples/docs_workflow.rs). Su función auxiliar `verify` comprueba el número de notas, tarjetas y recursos multimedia del resultado correcto.

<!-- source: anki_forge/examples/docs_workflow.rs#cards -->
```rust
fn cards(output: &Path) -> anyhow::Result<()> {
    let mut project = Project::new("docs-spanish")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", "hello"))?;
    project.add(
        "capital",
        Note::cloze("{{c1::Madrid}} is in {{c2::Spain}}.")
            .field("back_extra", "A city and its country."),
    )?;
    let built = project.build(BuildOptions::to(output.join("cards.apkg")))?;
    verify(&built, 2, 3, 0)?;
    Ok(())
}
```
<!-- /source -->

El primer argumento de `Project::new` es un espacio de nombres estable. Cada llamada a `add` necesita una clave estable para el registro de origen. Reutilizar una clave para otra nota provoca un error; no se asigna automáticamente una clave basada en el contenido. Si cambias una respuesta manteniendo la misma clave, seguirá siendo la misma nota lógica al generar una actualización a partir de su distribución anterior.

<!-- section: text-fields-and-card-identity -->
## Texto, campos e identidad de las tarjetas

Las cadenas de Basic y Cloze son Text. Usa `Content::html` cuando quieras incluir HTML; la sintaxis de omisión Cloze se sigue reconociendo dentro de Text. `back_extra` es la clave del campo adicional del modelo Cloze integrado. Utiliza el mismo método `Note::field` para texto, HTML, imágenes y sonidos.

Usa `{{c1::answer}}` o `{{c1::answer::hint}}`. Una frase sin omisiones no genera ninguna pregunta Cloze. Se admiten números de omisión del 1 al 500; cambiarlos puede alterar la identidad de las tarjetas. Revisa estos cambios mediante [la comparación y la política de actualización](/docs/updates/).

Después, añade [recursos multimedia](/docs/media/), [un modelo personalizado](/docs/custom-notetypes/) u [oclusiones de imágenes](/docs/image-occlusion/).
