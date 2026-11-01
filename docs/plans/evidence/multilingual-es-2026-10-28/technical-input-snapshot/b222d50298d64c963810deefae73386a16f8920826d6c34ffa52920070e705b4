# Guía de desarrollo con Rust

Un `Project` describe una publicación. Créalo con un espacio de nombres estable, añade notas con claves estables y genera un APKG. Los tipos habituales se importan directamente de `ankiforge`; las opciones avanzadas se encuentran en los módulos públicos de cada dominio.

<!-- section: projects-and-diagnostics -->
## Proyectos y diagnósticos

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

Una generación correcta devuelve un `BuildOutput` con un archivo de salida garantizado. `BuildReport` contiene observaciones: recuentos, diagnósticos, tiempo transcurrido y cualquier comparación completada. No contiene el resultado de éxito o fallo ni mantiene la propiedad del archivo. Las advertencias no convierten una salida correcta en un fallo.

Los errores de esquema se comprueban al completar un modelo. `Project::add` comprueba de forma atómica la clave de la nota, las referencias a campos, los campos obligatorios, el mazo, las etiquetas y los conflictos entre dependencias. Si falla, el proyecto queda sin cambios. La generación realiza las comprobaciones restantes de tarjetas, paquetes y actualizaciones. No hay un registro mutable independiente ni un paso de validación obligatorio por separado.

Los errores exponen los métodos inherentes `kind()` y `code()` e implementan `std::error::Error`. Utiliza los códigos procesables por máquina en lugar de analizar el mensaje mostrado. `BuildError::snapshot()` incluye el fallo real, las observaciones, el texto de la cadena de errores de origen y los datos sobre la publicación del archivo. Los contenedores estándar de errores pueden conservar el error original como origen.

<!-- section: models-and-assets -->
## Modelos y recursos

Un `NoteType` completado es inmutable y se puede compartir. Llama a `model.note()`, asigna los campos por su clave estable y añade el resultado a un proyecto. La nota conserva su modelo; al añadirla se recopilan automáticamente el modelo y los recursos multimedia. Consulta [tipos de nota personalizados](/docs/custom-notetypes/) para conocer las declaraciones y las reglas de las plantillas.

`Media::file` y `Media::bytes` obtienen instantáneas cuyos datos pasan a ser gestionados por el valor multimedia. Los archivos de origen no necesitan seguir disponibles una vez importados correctamente. `image.image()` y `audio.sound()` producen contenido estructurado; `Content::sequence` lo combina con texto o HTML. Los recursos utilizados directamente en HTML, CSS o scripts deben incluirse explícitamente mediante `builder.asset` o `project.add_asset`. Consulta [recursos multimedia](/docs/media/) y [paquetes de plantillas](/docs/templates/).

<!-- section: output-ownership -->
## Propiedad del archivo de salida

Elige `BuildOptions::to(path)` para una salida persistente o `BuildOptions::temporary()` para un archivo temporal gestionado por su propietario. Conserva la salida o un clon de `output.artifact()` mientras accedes a una ruta temporal. El último propietario elimina el archivo al liberarse. `persist_to(path)` devuelve un propietario persistente; si falla, el propietario original sigue siendo utilizable. Las instantáneas JSON contienen rutas, pero no mantienen vivos los archivos.

<!-- section: stable-note-identity -->
## Identidad estable de las notas

Utiliza una clave duradera del registro de origen en `project.add(key, note)`. El espacio de nombres del proyecto y las claves de modelos, campos y plantillas son independientes de los nombres que se muestran. Cambiar el contenido no crea una nueva nota lógica. Las claves duplicadas producen un error en lugar de sobrescribir datos. Guarda y reutiliza las claves elegidas cuando reordenes o edites los registros de origen.

<!-- section: updating-distributed-decks -->
## Actualizar mazos distribuidos

Utiliza el APKG original de distribución anterior con `update_from`. Sus pruebas completas integradas conservan las correspondencias de identidad y revisión. Por separado, `project.compare(CompareOptions::against(path))` analiza la versión candidata e informa de si la política permitiría publicarla. Consulta el [flujo con dos versiones](/docs/updates/) para conocer los cambios de alto riesgo, las limitaciones de importación del cliente y las autorizaciones explícitas de riesgos.

Desde una copia del repositorio, estos ejemplos ejecutan la misma API pública:

```sh
cargo run --locked -p ankiforge --example target_api_basic
cargo run --locked -p ankiforge --example target_api_custom_notetype
cargo run --locked -p ankiforge --example target_api_media
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

<!-- section: addition-errors-and-media-categories -->
### Errores al añadir notas y categorías multimedia

`Project` utiliza un espacio de nombres estable y un mazo predeterminado opcional; no tiene un título de presentación. `NoteType`, `Field` y `Template` siguen admitiendo nombres de presentación. `Project::add` es atómico. Consulta las claves de nota y modelo en `AddError::context()`, y la ubicación original en `note::AddTarget`; `detail()` devuelve `note::AddDetail` para los conflictos o los usos multimedia incompatibles. Al usar `match` con estas enumeraciones no exhaustivas, utiliza `..` en las variantes con datos y una rama alternativa. El `content_path` de un campo es `None` para el propio campo, `Some([])` para su raíz, o una secuencia de índices para un nodo anidado. Los rangos de bytes de separadores señalan la hoja Text/Html UTF-8 original.

`Media::image()` y `sound()` siguen siendo infalibles. Al añadir notas, las imágenes requieren `image/*` y las referencias de sonido, `audio/*` o `video/*`. Otras categorías producen `NOTE.MEDIA_USAGE_INVALID`. Esta comprobación utiliza el MIME conservado, no decodifica ni comprueba la reproducción. El HTML sin procesar y los recursos explícitos mantienen su comportamiento anterior. Cambiar el nombre de exportación no cambia el MIME: un PNG renombrado como `wrong.mp3` supera la comprobación al añadir la imagen, pero todavía puede fallar la comprobación independiente de contenido y extensión durante la generación con `MEDIA.DECLARED_MIME_MISMATCH`.

Las recetas de las herramientas ahora requieren `ankiforge-project-v2`, sin `name` en el nivel superior; v1 se rechaza. El paquete de contrato es 2.1.0; la identidad del paquete sigue siendo `ankiforge-identity-v1`.
