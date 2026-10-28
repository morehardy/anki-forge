# Imágenes y audio

Un valor `Media` conserva una copia inmutable de los datos. Importa archivos con `Media::file`, o bytes nativos con un tipo MIME explícito mediante `Media::bytes`. Tras una importación correcta, modificar o eliminar el archivo de origen no puede cambiar el recurso. Los clones comparten almacenamiento y se pueden usar en distintos proyectos.

<!-- section: execute-the-media-workflow -->
## Ejecuta el flujo de trabajo multimedia

Desde la raíz del repositorio:

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
cargo run --locked -p ankiforge --example target_api_media
```

El flujo utiliza los archivos PNG y WAV de prueba del repositorio y escribe `media.apkg` con una nota, una tarjeta y tres recursos. El segundo ejemplo genera su propia imagen pequeña y su audio, incluye recursos enlazados desde CSS y recursos explícitos, y escribe `spanish-media.apkg`.

<!-- source: anki_forge/examples/docs_workflow.rs#media -->
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
<!-- /source -->

<!-- section: compose-content-and-explicit-assets -->
## Combina contenido y recursos explícitos

Este programa completo utiliza `fixtures/pixel.png`, `fixtures/silence.wav` y `fixtures/labels.woff`. En tu aplicación, indica las rutas de tus propios archivos. La fuente de prueba es una fuente redistribuible del repositorio.

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

Los valores tipados `image()` y `sound()` conservan las dependencias hasta la exportación. Las referencias escritas a mano en HTML, CSS o scripts necesitan `builder.asset(media)` o `project.add_asset(media)`; la biblioteca no deduce el contenido de los archivos a partir de cadenas arbitrarias. Un recurso explícito se incluye incluso si no se detecta ninguna referencia estática.

Pasa el nombre de archivo literal a `with_export_name`. Al representarse, las referencias tipadas de sonido escapan las entidades HTML, por lo que un nombre como `tone&copy;.mp3` sigue haciendo referencia a ese recurso exacto. No escapes el nombre previamente. Los signos de porcentaje y los caracteres de fragmento de los nombres de sonido se mantienen literales; no son rutas URL.

<!-- section: names-mime-and-budgets -->
## Nombres, tipos MIME y límites

Los nombres de exportación predeterminados se derivan del contenido y no dependen de la ruta de origen. Utiliza `with_export_name` antes de crear referencias cuando necesites un nombre fijo. Cambiar el nombre de un clon no modifica los clones ni los nodos de contenido anteriores.

Los nombres no pueden contener rutas, caracteres de control ni nombres reservados no portables. La detección de conflictos utiliza Unicode NFC y normalización de mayúsculas y minúsculas: las variantes que solo difieren en estos aspectos también entran en conflicto. Los pares de nombre y contenido exactamente iguales son idempotentes; contenidos distintos con un mismo nombre provocan un error sin registro parcial.

El nombre de exportación completo, incluida la extensión, tiene un límite de 255 bytes UTF-8. Los JSON Schemas del proyecto y de los paquetes de plantillas comprueban la sintaxis y el número de caracteres; los cargadores también comprueban este límite de bytes. Por ejemplo, 128 repeticiones de `é` pasan el límite de caracteres del esquema, pero ocupan 256 bytes y devuelven `MEDIA.EXPORT_NAME_INVALID` al cargarse. Usa `tools::load_project` o `NoteType::from_bundle` para validar la entrada completa antes de generar el paquete.

La sintaxis MIME debe coincidir con el contenido identificable. Los bytes nativos no tienen una restricción de 64 KiB para datos integrados. El límite de importación predeterminado es de 256 MiB por recurso; `file_with_limits` y `bytes_with_limits` aceptan `MediaLimits { max_bytes }` antes de leer los datos. Las copias grandes pueden pasar a almacenamiento temporal, que se limpia cuando desaparece el último objeto que las conserva. Conservar una copia no garantiza una vista atómica del sistema de archivos si el origen se modifica durante la importación.

El JSON del proyecto aplica el mismo límite predeterminado por recurso al decodificar un array `bytes` integrado. Se detiene en el primer byte que supera el límite sin reunir el resto del array, y conserva un `MediaError` con el límite y el número de bytes observado. El recuento en el momento del rechazo es un mínimo; no es el tamaño total del array que no se ha leído.

Un paquete multimedia válido no demuestra que sus códecs se reproduzcan en todos los clientes de Anki. Prueba las imágenes, el sonido y las fuentes en los clientes de destino. Consulta [la solución de problemas](/docs/troubleshooting/#media-and-templates).

<!-- section: typed-image-and-sound-usage -->
### Uso tipado de imágenes y sonidos

Los constructores siguen siendo infalibles. `Project::add` comprueba el MIME conservado durante la importación: las imágenes requieren `image/*` y las referencias de sonido aceptan `audio/*` y `video/*`. `application/octet-stream`, texto, fuentes y PDF no admiten ese uso tipado y producen `NOTE.MEDIA_USAGE_INVALID`; regístralos como recursos explícitos para HTML o CSS sin procesar. Las declaraciones explícitas válidas de tipos MIME de imagen o vídeo desconocidos pasan la validación de categoría, sin certificar que un decodificador o reproductor admita los bytes. Cambiar el nombre de un archivo nunca cambia el MIME conservado. La generación sigue identificando los bytes preparados y validando la extensión de exportación por separado: un PNG llamado `wrong.mp3` puede añadirse como imagen y fallar al generar el paquete con `MEDIA.DECLARED_MIME_MISMATCH`.

<a id="batch-import-from-source"></a>

<!-- section: batch-media-import -->
## Importación de recursos multimedia por lotes

Estos métodos están incluidos en el paquete público `0.3.1`.

Importa una colección con concurrencia limitada, conserva el orden de entrada y retén las copias de los recursos multimedia para exportarlas más adelante:

```rust
use ankiforge::Media;
let images = Media::files(["first.png", "second.png"])?;
```

En Node se usa `await Media.files(['first.png', 'second.png'])`; en Python, `Media.files(['first.png', 'second.png'])`.

<!-- section: batch-behavior-and-budgets -->
### Comportamiento y límites de los lotes

Usa `Media::files(paths)` en Rust, `await Media.files(paths)` en Node o `Media.files(paths)` en Python para importar los archivos en el orden de entrada con un máximo de cuatro trabajadores por llamada. Los lotes de menos de 16 archivos se ejecutan de forma secuencial. Rust también ofrece `files_with_limits`; Node y Python aceptan los mismos límites que `file`. El límite se aplica a cada recurso. Si falla una importación, la llamada espera al trabajo que ya ha comenzado, libera las copias del lote y devuelve el primer error según el orden de entrada. Los objetos que ya conservaban recursos multimedia siguen siendo válidos. Python libera el GIL durante la importación; Node ejecuta el trabajo fuera del bucle de eventos. La importación de un solo archivo sigue creando una copia inmediata cuyos datos se conservan; para importar con concurrencia hay que elegir explícitamente el método por lotes.

Las copias activas comparten un límite de memoria de **64 MiB** para todo el proceso. Los recursos individuales de más de 1 MiB siguen enviándose a archivos temporales. Los recursos multimedia codificados comparten un conjunto de búferes de **16 MiB**, que se reserva cuando se necesita en cada preparación y pasa a almacenamiento temporal al llenarse. Estos límites no incluyen el resto de la memoria de trabajo ni constituyen un límite de RSS; las generaciones concurrentes pueden tener cada una su propio conjunto de búferes de codificación.
