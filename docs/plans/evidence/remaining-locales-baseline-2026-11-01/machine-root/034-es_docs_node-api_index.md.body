# API de Node y TypeScript

Canonical page: https://ankiforge.dev/es/docs/node-api/
Language: es
Description: Métodos públicos de Node, operaciones asíncronas, opciones de generación, gestión de archivos de salida e informes.

El punto de entrada `ankiforge` expone un único modelo de creación de contenido basado en Project. Los tipos se definen en [el código fuente público](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/index.ts), las [clases de informes](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/report.ts) y las [declaraciones de instantáneas JSON](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/snapshots.ts). Empieza por la [guía de inicio](https://ankiforge.dev/es/docs/node-quickstart/).

## Proyecto y notas

| API | Comportamiento |
| --- | --- |
| `new Project(namespace)` | Validar un espacio de nombres estable para la publicación |
| `.defaultDeck(name)` | Definir el destino predeterminado; devuelve el proyecto |
| `.add(key, note)` | Recopilar nota, modelo y recursos multimedia de forma atómica; las claves duplicadas producen un error |
| `.addAsset(media)` | Incluir un recurso explícito para HTML, CSS o scripts sin procesar |
| `.clone()` | Proyecto independiente y editable que comparte valores inmutables |
| `.length` | Número de notas registradas |
| `Note.basic(front, back)` / `Note.cloze(text)` | Nota inmutable que conserva su modelo integrado |
| `model.note().field(key, content)` | Nota personalizada inmutable que conserva su modelo |
| `note.deck(name).tag(tag).tags(iterable)` | Devolver una nota nueva configurada |

Las cadenas siempre se convierten en Text, incluidas las cadenas Cloze. El marcado explícito utiliza `Content.html`. `Content.sequence(iterable)` combina texto, HTML y recursos multimedia tipados. Utiliza claves explícitas de tus datos de negocio; ni el contenido ni los nombres que se muestran determinan la identidad.

## Modelos inmutables

```js
import { NoteType, Field, Template, GenerationRule } from 'ankiforge';
const model = NoteType.builder('vocabulary')
  .name('词汇')
  .field(new Field('front', { name: '正面', required: true, sort: true }))
  .field(new Field('back', { name: '背面' }))
  .template(new Template('recognition', {
    front: '{{front}}',
    back: '{{FrontSide}}<hr>{{back}}',
    generation: GenerationRule.all(['front']),
  }))
  .build();
const note = model.note().field('front', 'cell').field('back', '细胞');
```

Los constructores devuelven valores nuevos. Un modelo completado no tiene métodos para modificar campos o plantillas. `builder.clozeField(key)` selecciona la semántica Cloze personalizada; `builder.css` y `builder.asset` aportan estilos y dependencias explícitas. Las plantillas también aceptan `name`, `browserFront`, `browserBack` y `targetDeck`. Las referencias de las plantillas y las condiciones de generación utilizan claves de campos; los nombres de presentación se compilan para Anki. `GenerationRule.ankiDefault()` selecciona el comportamiento predeterminado de Anki; `all` y `any` seleccionan condiciones explícitas.

`await NoteType.fromBundle(path, mediaLimits?)` devuelve el mismo modelo inmutable desde `template-bundle-v2`. Tras una carga correcta, el texto y los recursos importados dejan de depender del directorio de origen. Los modelos pueden reutilizarse entre proyectos; las definiciones incompatibles bajo una misma clave de modelo provocan un fallo atómico.

## Recursos multimedia y oclusión de imágenes

`await Media.file(path, limits?)` y `await Media.bytes(Uint8Array, mime, limits?)` devuelven instantáneas que gestionan sus propios datos. Los archivos de origen pueden cambiar o desaparecer una vez que `await` termina correctamente. `withExportName(name)` devuelve un valor renombrado sin cambiar los clones ni el contenido anteriores. Lee `filename`, `mediaType` y `byteLength` como propiedades.

`media.image()` y `.sound()` devuelven Content tipado que incorpora automáticamente su recurso al proyecto. `project.addAsset` y `builder.asset` del modelo cubren las referencias explícitas de HTML, CSS o scripts sin procesar. Los límites multimedia aceptan `{ maxBytes }`; el valor predeterminado es 256 MiB y los bytes nativos no tienen una restricción de 64 KiB para datos en línea.

```js
import { Media, Note, Mask } from 'ankiforge';
const image = await Media.file('diagram.png');
const note = Note.imageOcclusion(image)
  .mask(Mask.rect('nucleus', 10, 10, 20, 20))
  .mask(Mask.rect('wall', 50, 50, 20, 20))
  .mode('hide_one_guess_one')
  .build().field('header', 'Cell');
```

El modo predeterminado es `hide_all_guess_one`; ambos modos generan tarjetas independientes para las máscaras identificadas por claves. `.build()` valida las dimensiones decodificadas, los rectángulos finitos y las claves únicas. `image` y `occlusion` son campos generados reservados. Consulta [oclusión de imágenes](https://ankiforge.dev/es/docs/image-occlusion/) para conocer los límites de decodificación y actualización.

## Generación, comparación y salida

| API | Resultado |
| --- | --- |
| `await project.build(BuildOptions.to(path))` | BuildOutput que mantiene la propiedad de un archivo persistente |
| `await project.build(BuildOptions.temporary())` | BuildOutput que mantiene la propiedad de un archivo temporal |
| `await project.compare(CompareOptions.against(path))` | ComparisonReport completado, incluso si la política bloquea la publicación |
| `options.updateFrom(path)` | Configurar la actualización a partir de las pruebas completas del APKG original de distribución |
| `options.inspectLimits(limits)` | Modificar los contadores de inspección independientes |
| `options.updatePolicy(policy)` | Configurar la política de riesgo de actualización y comparación |

Una salida correcta expone `.artifact`, `.report` y `.snapshot()`. `BuildReport` expone `.counts`, `.baselineCounts`, `.durationMs`, `.diagnostics`, `.comparison` y `.snapshot()`; no contiene el resultado de éxito o fallo ni mantiene vivos archivos. `.comparison` es un `ComparisonReport` o `null`, y utiliza la misma clase que `project.compare()`. Las comparaciones exponen `.findings`, `.highestRisk`, `.policy`, `.diagnostics` y `.snapshot()`. Los nombres de los campos de las instantáneas coinciden con el JSON de Rust, incluidos `allows_publication` y `schema_version`; no son copias en camelCase del informe nativo. Utiliza `report.comparison.highestRisk` en el objeto de comparación, o `report.snapshot().comparison.highest_risk` en su instantánea JSON, después de comprobar que no sea `null`.

`new UpdatePolicy()` bloquea High/Critical. `.failOn('medium')` modifica el umbral. `.allow('RISK.NOTE_REMOVED')` acepta explícitamente toda esa categoría y conserva sus pruebas y gravedad. No se pueden autorizar códigos desconocidos ni errores que impiden continuar. Una política en una generación exclusivamente de creación es un error. Utiliza la misma política y los mismos límites al comparar y generar si necesitas decisiones de publicación coincidentes.

Las opciones de inspección incluyen `maxArchiveBytes`, `maxEntries`, `maxCentralDirectoryBytes`, `maxZipEntryBytes`, `maxZipTotalBytes`, `maxMetaBytes`, `maxMediaMapBytes`, `maxIdentityBytes`, `maxCollectionBytes`, `maxMediaBytes`, `maxDecodedTotalBytes` y `maxZstdWindowBytes`. Los valores son enteros seguros no negativos o `bigint`, hasta el máximo de u64; cero es un límite real de cero. Los contadores se aplican al paquete de referencia y a la versión candidata por separado. Las opciones desconocidas se rechazan.

## Vida útil del archivo de salida y concurrencia

Las rutas de las instantáneas JSON utilizan el tipo exportado `PathSnapshot`. Las rutas Unicode son cadenas; las rutas nativas que no son Unicode utilizan `{ encoding: "unix_bytes", bytes: number[] }` o `{ encoding: "windows_wide", units: number[] }`. Los arrays conservan exactamente los bytes Unix o las unidades de código UTF-16 de Windows. Esto incluye las rutas de archivos de salida en las instantáneas de generación, las rutas de publicación en los fallos y los detalles estructurados de rutas de errores. Decodifica una ruta codificada solo para su plataforma original; en Unix, `Buffer.from(value.bytes)` conserva una ruta en bytes para las funciones del sistema de archivos de Node. Los métodos de creación de contenido de Node siguen aceptando rutas de cadena.

Las rutas relativas de recursos multimedia y paquetes de plantillas se fijan cuando se invoca la lectura; los destinos y los paquetes de referencia de la generación se fijan cuando se construyen las opciones. En POSIX, los enlaces simbólicos seguidos de `..` conservan la semántica de recorrido del sistema de archivos. `persistTo` fija los destinos relativos al directorio de trabajo capturado por la generación original, incluso en identificadores clonados o persistidos. Los cambios posteriores del directorio de trabajo no redirigen esas rutas.

Mantén vivo un propietario del archivo mientras utilizas `.path`. `.clone()` crea un propietario independiente; `await .close()` libera ese propietario. El último propietario temporal elimina su archivo. `await .persistTo(path)` devuelve un archivo persistente y deja utilizable el original, incluso si falla. Los flujos estándar de archivos de Node pueden leer el archivo mientras conservas su propietario.

Las instantáneas son datos JSON inmutables y no mantienen vivos archivos nativos. Un archivo persistente sobrevive a la limpieza de su propietario. Quien llama a la API no puede construir `BuildOutput`; solo lo crean las generaciones correctas.

La generación y la comparación se ejecutan de forma asíncrona sobre una instantánea del proyecto, con propiedad de sus datos, capturada al invocar la operación. Las modificaciones posteriores no pueden alterar esa solicitud. Las lecturas multimedia y las cargas de paquetes de plantillas también se ejecutan en workers. Node ESM y CJS comparten el mismo módulo nativo y las mismas identidades de clase.

## Errores

`SchemaError`, `AddError`, `MediaError`, `TemplateBundleError`, `ImageOcclusionError`, `PolicyError`, `ConfigurationError`, `CompareError`, `BuildError` y `PersistError` extienden `ForgeError`. Exponen `kind`, `code`, `domain`, el texto de la cadena de errores nativos en `causes`, `sourceDetails` estructurado para las causas de E/S, esquema, recursos multimedia y límites reconocidas, y los `details` de la operación. La excepción nativa original se conserva como `cause` de JavaScript.

`BuildError.snapshot()` conserva los datos del fallo y de la publicación; `BuildError.report` y `CompareError.report` devuelven `BuildReport`, como `BuildOutput.report`. Utiliza `.baselineCounts` y `.comparison` en estos informes; usa `.snapshot()` para los campos JSON en snake\_case. Una comparación fallida puede tener observaciones parciales y una comparación `null`. `PersistError.publication` registra si el destino ya se había publicado y si se confirmó la durabilidad. Los errores multimedia y de inspección conservan detalles de los límites. Utiliza estos campos procesables por máquina en lugar de analizar mensajes de error. `ArtifactClosedError` informa del uso de un propietario cerrado; `NativeLoadError` informa de paquetes nativos ausentes, incompatibles o desactualizados.

## Contexto al añadir notas

Si una operación de adición falla, el proyecto queda sin cambios. El contexto estructurado registra las claves originales de nota y modelo y el destino, con detalles opcionales sobre el conflicto o el uso multimedia. Rust expone `note::{AddContext, AddTarget, AddDetail}` mediante `AddError::context()` / `detail()`; los SDK exponen `AddError.details` con contexto y detalles tipados. Los campos de Node usan camelCase y son inmutables en todos sus niveles; Python usa TypedDicts con snake\_case. Las etiquetas conservan su índice de inserción; las ubicaciones de los campos conservan los índices originales de la secuencia (`null`/`None` para el campo, `[]` para su raíz) y rangos opcionales de bytes UTF-8. Los errores de mazo predeterminado durante la generación no tienen claves de nota o modelo y siguen disponibles en la cadena de errores de origen.

Las imágenes tipadas requieren MIME image/\* al añadirlas; las referencias de sonido aceptan audio/\* o video/\*. Otras categorías provocan `NOTE.MEDIA_USAGE_INVALID`. Los constructores siguen siendo infalibles; los recursos explícitos y el HTML sin procesar no cambian. La comprobación utiliza el MIME conservado al importar y no certifica la reproducción. La validación independiente de MIME y extensión durante la generación sigue aplicándose. Consulta [el diseño](https://github.com/morehardy/anki-forge/blob/main/docs/plans/2026-09-28-rust-api-validation-and-errors-design.md) para ver la tabla completa de destinos y detalles.

## Recursos multimedia por lotes y publicación preparada

Las operaciones multimedia por lotes y la publicación preparada están incluidas en el paquete público `0.3.1`.

### Preparar, revisar y publicar

```js
import { Project, Note, BuildOptions, BuildError } from 'ankiforge';
const project = new Project('review-once').add('one', Note.basic('Question', 'Answer'));
const previous = await project.build(BuildOptions.to('previous.apkg'));
await previous.artifact.close();
const prepared = await project.preparePublication(
  BuildOptions.to('next.apkg').updateFrom('previous.apkg'),
);
try {
  const report = prepared.report; // immutable observations, no file ownership
  console.log(report.comparison?.snapshot());
  if (report.comparison?.policy.allows_publication !== false) {
    const output = await prepared.publish();
    await output.artifact.close(); // persistent next.apkg remains
  }
} catch (error) {
  if (error instanceof BuildError) console.log(error.snapshot().result);
  // Inspect publication/durability facts: a late failure can follow replacement.
  throw error;
} finally {
  await prepared.close(); // idempotent; cleans an unpublished candidate
}
```

`publish()` toma inmediatamente el propietario nativo y realiza una tarea en un worker. Cada intento lo consume, incluso si se rechaza por la política o por E/S. Las llamadas repetidas se rechazan con `PreparedPublicationStateError`, el código `BUILD.PREPARED_UNAVAILABLE` y el `reason` `consumed` o `closed`. Cerrar el propietario después de que comience la publicación no la cancela. Los informes siguen siendo legibles después de cerrarlo. Las opciones se fijan al invocar la preparación; cambiar la política o el destino requiere otra preparación. La duración excluye la espera de revisión. Ni los informes ni el JSON pueden convertirse de nuevo en un propietario.

La generación, la comparación, la preparación y los clones comparten versiones nativas inmutables de Project. Los cambios de contenido siguen siendo síncronos y aislados. El primer cambio mientras existe otra instantánea puede copiar el Project; los cambios exclusivos posteriores no lo hacen. Esto desplaza la copia fuera del envío de la Promise y no promete primeras modificaciones más rápidas, un menor retraso máximo de los temporizadores ni menos RSS. Los bytes de entrada de JavaScript se siguen copiando antes de enviar el trabajo al worker. Utiliza binarios del protocolo 6 con esta capa de JavaScript.

## TypeScript

Instala `typescript` y `@types/node` en tu aplicación. Utiliza `main.mts` o `"type": "module"` y este `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "outDir": "dist"
  }
}
```

Compila con `npx tsc` y después ejecuta `node dist/main.mjs` si el archivo de entrada es `.mts`.
