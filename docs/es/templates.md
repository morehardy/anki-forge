# Paquetes de plantillas

Reutiliza el mismo diseño de tarjetas Anki en distintos mazos: guarda las plantillas, el CSS y los recursos multimedia de un tipo de nota en un solo directorio y cárgalo para tarjetas normales o Cloze. Los ejemplos usan Rust; Python y Node.js cargan paquetes con `NoteType.from_bundle` y `NoteType.fromBundle`, respectivamente. Consulta la [API de Python](python-api.md#models-and-templates) y la [API de Node](node-api.md#immutable-models).

Un paquete de plantillas reúne una declaración de modelo reutilizable, el texto de las plantillas, CSS y todas las dependencias multimedia declaradas explícitamente. `NoteType::from_bundle(directory)` valida el paquete completo y conserva copias de los datos antes de devolver el mismo `NoteType` inmutable que se usa al definir un modelo en código. No modifica parcialmente ningún proyecto.

<!-- section: run-the-bundled-example -->
## Ejecuta el ejemplo con el paquete de plantillas

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

El flujo carga [el archivo de prueba Cloze personalizado](https://github.com/morehardy/anki-forge/blob/main/contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml) y escribe `target/docs-examples/template.apkg`. Su plantilla tiene como destino **Languages::Cloze**. Una nota genera dos tarjetas.

<!-- section: manifest-and-template-references -->
## Manifiesto y referencias de las plantillas

El archivo del manifiesto se llama `anki-template.yaml`. Solo se acepta el formato nuevo:

<!-- source: contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml -->
```yaml
format_version: template-bundle-v2
note_type:
  key: language-cloze
  name: Language Cloze
  cloze_field: text
  fields:
    - key: text
      name: Sentence
      sort: true
      required: true
    - key: extra
      name: Extra
  templates:
    - key: cloze
      name: Cloze
      front_file: front.html
      back_file: back.html
      browser_front_file: browser-front.html
      target_deck: Languages::Cloze
css_file: style.css
```
<!-- /source -->

`note_type.key`, las claves de los campos y las claves de las plantillas son identificadores estables obligatorios. `name` es opcional y, si se omite, toma el valor de la clave correspondiente. `cloze_field` selecciona un modelo Cloze; no hace falta una declaración separada del tipo. Un campo sin `required: true` se puede omitir. Se rechazan las propiedades desconocidas del manifiesto.

Todas las plantillas usan **claves de campo**, incluidas las variantes del anverso, reverso y explorador: `{{cloze:text}}`, `{{extra}}` y secciones condicionales como `{{#extra}}`. Después de enlazar las referencias, se generan para Anki los nombres visibles, como Sentence y Extra. El cargador conserva en los errores las rutas de los archivos originales y los intervalos de bytes de las plantillas.

<!-- section: load-and-reuse-a-model -->
## Carga y reutiliza un modelo

<!-- source: anki_forge/examples/docs_workflow.rs#bundle -->
```rust
fn bundle(output: &Path, template_directory: &Path) -> anyhow::Result<()> {
    let model = NoteType::from_bundle(template_directory)?;
    let mut project = Project::new("docs-languages")?;
    project.add(
        "capital",
        model
            .note()
            .field("text", "{{c1::Madrid}} is in {{c2::Spain}}.")
            .field("extra", "A city and its country."),
    )?;
    let built = project.build(BuildOptions::to(output.join("template.apkg")))?;
    verify(&built, 1, 2, 0)?;
    Ok(())
}
```
<!-- /source -->

Para una aplicación independiente, este ejemplo completo necesita que el mismo paquete de plantillas se haya copiado en `fixtures/template-bundle`:

```rust
use ankiforge::{BuildOptions, NoteType, Project};

fn main() -> anyhow::Result<()> {
    let model = NoteType::from_bundle("fixtures/template-bundle")?;
    let mut project = Project::new("bundle-example")?;
    project.add("capital", model.note()
        .field("text", "{{c1::Madrid}} is in {{c2::Spain}}.")
        .field("extra", "A city and its country."))?;
    let output = project.build(BuildOptions::to("bundle-example.apkg"))?;
    assert_eq!(output.report().counts().cards, 2);
    Ok(())
}
```

El modelo devuelto no depende del directorio de origen. Puedes compartirlo con otro proyecto o usarlo después de eliminar ese directorio. Si falla la carga, no se devuelve ningún modelo parcial y se limpian las copias que no se hayan compartido.

<!-- section: assets-and-budgets -->
## Recursos y límites

Declara explícitamente todos los recursos usados en plantillas, CSS o scripts sin procesar. Cada entrada de `assets` utiliza una ruta relativa al paquete y un nombre de exportación fijo:

```yaml
assets:
  - path: assets/badge.png
    export_as: badge.png
  - path: assets/labels.woff
    export_as: labels.woff
```

Las plantillas pueden usar después `<img src="badge.png">` y el CSS, `url('labels.woff')`. La propiedad `css_file` indica el archivo de origen del CSS; los recursos no se deducen de HTML, CSS o JavaScript arbitrarios. Los recursos declarados permanecen en el APKG aunque no se detecte ninguna referencia estática. Los nombres de exportación cumplen las mismas reglas de portabilidad y de conflicto por Unicode o mayúsculas que [Media](/docs/media/).

El nombre de exportación completo tiene un límite de 255 bytes UTF-8, incluida la extensión. El JSON Schema comprueba la sintaxis y el número de caracteres; `NoteType::from_bundle` también comprueba el límite de bytes, el contenido de los archivos y las referencias antes de devolver un modelo validado. Superar solo la validación del esquema no garantiza que la carga vaya a completarse.

Se rechazan las rutas absolutas, las rutas que salen del paquete y los enlaces simbólicos que apuntan fuera de él. El manifiesto tiene un límite de 256 KiB; cada archivo de texto de plantilla o CSS tiene un límite de 2 MiB y debe ser UTF-8. Las copias de los recursos usan el límite multimedia predeterminado, o un `MediaLimits` indicado por quien llama a `NoteType::from_bundle_with_limits`. Se comprueban tanto las longitudes previas a la lectura como los bytes leídos realmente.

Los errores del paquete conservan el tipo y el código, la ruta de origen, el desplazamiento en bytes cuando corresponde y el error subyacente de esquema, E/S, YAML, UTF-8 o recurso multimedia. Al finalizar el modelo se valida su esquema; al añadir notas se comprueban por separado los conflictos del proyecto. Consulta [los tipos de nota personalizados](/docs/custom-notetypes/) y [la solución de problemas](/docs/troubleshooting/).
