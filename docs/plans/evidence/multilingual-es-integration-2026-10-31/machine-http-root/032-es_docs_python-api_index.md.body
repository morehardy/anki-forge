# API de Python

Canonical page: https://ankiforge.dev/es/docs/python-api/
Language: es
Description: Project, recursos multimedia, opciones de generación, informes y gestión de la vida útil de los archivos temporales.

El SDK llama a la API pública predeterminada de Rust mediante una extensión nativa. Incluye `py.typed` y firmas públicas verificadas. Empieza por la [guía de inicio](https://ankiforge.dev/es/docs/python-quickstart/).

## Valores de proyecto y nota

| Entrada | Comportamiento |
| --- | --- |
| `Project(namespace, *, default_deck=None)` | Valida el espacio de nombres estable explícito y configura el mazo predeterminado |
| `project.add(key, note)` | Recopila de forma atómica la nota, el modelo y los recursos multimedia que gestionan sus datos; las claves de nota duplicadas producen un error |
| `project.add_asset(media)` | Incluye una dependencia explícita de HTML, CSS o scripts sin procesar |
| `len(project)` | Número de notas añadidas correctamente |
| `Note.basic(front, back)` | Crea una nota con el modelo Basic integrado; las cadenas representan Text |
| `Note.cloze(text)` | Crea una nota con el modelo Cloze integrado; las cadenas representan Text y conservan la sintaxis cloze |
| `note.field(key, content)` | Devuelve una nota nueva con el campo asignado; la clave se valida al añadir la nota |
| `note.deck(name)` / `.tag(tag)` / `.tags(tags)` | Devuelve una nota configurada |
| `note.note_type` | El modelo inmutable que conserva la nota |

Las cadenas ordinarias se convierten en Text. `Content.text(value)` y `Content.html(value)` son constructores explícitos; `Content.sequence(iterable)` combina cadenas y contenido sin renderizarlos ni perder las dependencias multimedia. Un campo no acepta un nombre de archivo sin más como referencia multimedia.

## Modelos y plantillas

`NoteType.builder(key)` devuelve un constructor inmutable. Sus métodos `.name`, `.field`, `.template`, `.css`, `.cloze_field` y `.asset` devuelven constructores actualizados. `.build()` valida la definición completa y devuelve un NoteType inmutable. `model.note()` crea una nota que conserva ese modelo. `NoteType.from_bundle(path, limits=MediaLimits())` carga el mismo modelo y sus recursos desde el formato actual de paquetes de plantillas; el límite opcional se aplica a cada recurso por separado antes de leerlo.

`Field(key, name=None, required=False, sort=False)` separa la clave estable del nombre que se muestra. `Template(key, front, back, name=None, browser_front=None, browser_back=None, target_deck=None, generation=GenerationRule())` utiliza claves de campos en todas las cadenas de origen. `GenerationRule.all(keys)` y `.any(keys)` configuran la generación de tarjetas. El comportamiento predeterminado infiere la condición de presencia de campos de Anki. El método `.cloze_field(key)` del constructor declara la generación cloze personalizada.

## Recursos multimedia y oclusión

| Entrada | Comportamiento |
| --- | --- |
| `Media.file(path, *, limits=MediaLimits())` | Lee inmediatamente una instantánea cuyos datos gestiona el valor |
| `Media.bytes(data, media_type, *, limits=MediaLimits())` | Conserva los bytes con un tipo MIME explícito |
| `media.with_export_name(name)` | Devuelve un valor con otro nombre de archivo que comparte la instantánea; las referencias anteriores conservan sus nombres |
| `media.filename`, `media.media_type`, `len(media)` | Metadatos de solo lectura de la instantánea |
| `media.image()` / `.sound()` | Contenido tipado que conserva la instantánea |

`MediaLimits(max_bytes=256 << 20)` controla cada importación antes de crear la instantánea. Los recursos grandes pueden almacenarse en archivos temporales gestionados por sus propietarios. Los conflictos de nombres se validan entre todos los recursos explícitos y los recopilados automáticamente.

`Note.image_occlusion(media)` devuelve un ImageOcclusionBuilder. Encadena `.mask(Mask.rect(key, x, y, width, height))`, opcionalmente `.mode(OcclusionMode.HIDE_ONE_GUESS_ONE)`, y después `.build()`. El modo predeterminado es `HIDE_ALL_GUESS_ONE`. Al llamar a `.build()` se decodifica la imagen y se validan las claves estables de las máscaras y las coordenadas en píxeles, que deben ser finitas y estar dentro de los límites. En el Note devuelto, asigna `header`, `back_extra` o `comments` de la forma habitual. Los campos generados `image` y `occlusion` no se pueden asignar manualmente.

## Operaciones y resultados

| Entrada | Resultado |
| --- | --- |
| `project.build(BuildOptions.to(path))` | BuildOutput con un archivo persistente garantizado |
| `project.build(BuildOptions.temporary())` | BuildOutput con un archivo temporal gestionado por su propietario |
| `options.update_from(path)` | Devuelve opciones para actualizar a partir de las pruebas completas del paquete de referencia |
| `project.compare(CompareOptions.against(path))` | ComparisonReport completado, con una decisión de publicación separada según la política |
| `options.update_policy(policy)` | Aplica una política de riesgo explícita a Update/Compare; no es válida para Create |
| `options.inspect_limits(limits)` | Aplica límites finitos al paquete de referencia y a la versión candidata por separado |

BuildOutput tiene `.artifact`, `.report` y `.snapshot()`. BuildReport expone `.counts` (BuildCounts con notas, tarjetas y recursos multimedia), `.diagnostics`, `.comparison` y `.snapshot()`. Un informe no contiene el resultado de la operación ni mantiene vivo un archivo. ComparisonReport expone `.findings`, `.policy`, `.allows_publication` y `.snapshot()`. Las instantáneas son diccionarios ordinarios serializables como JSON y no mantienen la propiedad de los archivos de salida.

ApkgArtifact expone `.path`, `.persist_to(path)` y `.close()`. Copiarlo con `copy.copy` conserva otro propietario; un gestor de contexto libera ese propietario al salir. La salida temporal desaparece cuando se libera su último propietario. Las rutas persistentes sobreviven a la limpieza de los identificadores. Si la persistencia falla, el identificador de origen sigue siendo utilizable.

`UpdatePolicy()` bloquea los hallazgos High y Critical. `.fail_on(RiskLevel.MEDIUM)` modifica el umbral. `.allow('RISK.NOTE_REMOVED')` acepta explícitamente toda la categoría registrada y conserva las pruebas. Los códigos desconocidos se rechazan inmediatamente. `InspectLimits` expone todos los límites finitos de bytes y recuentos, incluido `max_identity_bytes`; no puede desactivar las inspecciones obligatorias.

## Errores y concurrencia

Los fallos lanzan SchemaError, AddError, MediaError, ImageOcclusionError, TemplateBundleError, CompareError, PolicyError, BuildError o PersistError. Cada uno expone `kind`, `code`, `details`, observaciones estructuradas en `source_details` y el texto de la cadena de errores en `causes`, y conserva la excepción nativa como `__cause__`. Cuando está disponible, el error de E/S de origen se conserva como un OSError encadenado. BuildError proporciona `.report` y `.snapshot()`; CompareError proporciona un `.report` parcial; los detalles de PersistError incluyen los datos reales sobre la publicación.

Las operaciones nativas de Project liberan el GIL y rechazan el uso simultáneo del mismo objeto con `BINDING.PROJECT_BUSY`; los proyectos independientes pueden ejecutarse a la vez. Después de bifurcar un proceso con fork, vuelve a crear todos los valores e identificadores nativos. Los identificadores heredados rechazan operaciones con `BINDING.FORKED_OBJECT`, y su limpieza no puede eliminar archivos que pertenecen al proceso padre. Los valores creados por el proceso hijo funcionan y se limpian normalmente. Consulta [diagnósticos](https://ankiforge.dev/es/docs/python-diagnostics/).

## Contexto al añadir notas

Si una operación de adición falla, el proyecto queda sin cambios. El contexto estructurado registra las claves originales de nota y modelo y el destino, con detalles opcionales sobre el conflicto o el uso multimedia. Rust expone `note::{AddContext, AddTarget, AddDetail}` mediante `AddError::context()` / `detail()`; los SDK exponen `AddError.details` con contexto y detalles tipados. Los campos de Node usan camelCase y son inmutables en todos sus niveles; Python usa TypedDicts con snake\_case. Las etiquetas conservan su índice de inserción; las ubicaciones de los campos conservan los índices originales de la secuencia (`null`/`None` para el campo, `[]` para su raíz) y rangos opcionales de bytes UTF-8. Los errores de mazo predeterminado durante la generación no tienen claves de nota o modelo y siguen disponibles en la cadena de errores de origen.

Las imágenes tipadas requieren MIME image/\* al añadirlas; las referencias de sonido aceptan audio/\* o video/\*. Otras categorías provocan `NOTE.MEDIA_USAGE_INVALID`. Los constructores siguen siendo infalibles; los recursos explícitos y el HTML sin procesar no cambian. La comprobación utiliza el MIME conservado al importar y no certifica la reproducción. La validación independiente de MIME y extensión durante la generación sigue aplicándose. Consulta [el diseño](https://github.com/morehardy/anki-forge/blob/main/docs/plans/2026-09-28-rust-api-validation-and-errors-design.md) para ver la tabla completa de destinos y detalles.

## Operaciones por lotes y publicación preparada

Las operaciones multimedia por lotes y la publicación preparada están incluidas en el paquete público `0.3.1`.

### Preparar, revisar y publicar

```python
from ankiforge import Project, Note, BuildOptions, BuildError
project = Project('review-once').add('one', Note.basic('Question', 'Answer'))
previous = project.build(BuildOptions.to('previous.apkg'))
previous.artifact.close()
prepared = project.prepare_publication(
    BuildOptions.to('next.apkg').update_from('previous.apkg')
)
try:
    report = prepared.report  # observations; does not own the candidate
    if report.comparison is None or report.comparison.allows_publication:
        output = prepared.publish()
        output.artifact.close()  # persistent next.apkg remains
except BuildError as error:
    print(error.snapshot()['result'])  # actual publication/durability facts
    raise
finally:
    prepared.close()  # idempotent; deletes an unused candidate
```

Un intento de publicación consume el propietario, incluso si falla por la política o por un error de E/S. Las llamadas posteriores lanzan `PreparedPublicationStateError` con el código `BUILD.PREPARED_UNAVAILABLE` y el motivo `closed` o `consumed`; cerrar el propietario no cancela una publicación que ya esté en curso. Las rutas relativas se fijan al invocar la preparación, y los cambios en Project no pueden modificar el APKG revisado. Los informes siguen disponibles después de cerrar el propietario, sin mantener archivos vivos. Para cambiar opciones o reintentar, vuelve a preparar la publicación. La duración excluye la espera de revisión. Un error tardío puede significar que la sustitución se realizó, pero su durabilidad no se confirmó; consulta los datos de publicación antes de decidir qué hacer. Se requieren el protocolo de enlace nativo 1 y el paquete de contrato 2.1.0; los binarios anteriores con la misma versión se rechazan.
