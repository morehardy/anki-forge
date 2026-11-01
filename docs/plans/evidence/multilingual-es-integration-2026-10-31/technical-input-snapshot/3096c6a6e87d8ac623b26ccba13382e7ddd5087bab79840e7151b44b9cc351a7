# Solución de problemas

Usa los valores estructurados `kind` y `code` de la operación fallida, junto con su cadena de causas y sus observaciones. El texto visible está pensado para las personas y puede cambiar. Una operación fallida y una comparación completada cuya política bloquea la publicación son resultados distintos.

<!-- section: installation -->
## Instalación

| Síntoma | Comprobación |
| --- | --- |
| Rust rechaza la versión de las herramientas | Ejecuta `rustc --version`; usa Rust 1.92+ y comprueba la versión instalada con `cargo tree -i ankiforge` |
| Node no puede cargar su paquete nativo | Ejecuta `node -p "process.platform + '/' + process.arch"` y `npm ls ankiforge`; reinstala con `npm install --include=optional ankiforge@0.3.1` |
| Node indica que las versiones del componente nativo y del módulo JavaScript no coinciden | Usa versiones coincidentes; reinstala ambos desde la misma versión del registro o recompílalos desde el mismo repositorio descargado |
| Python no puede cargar su extensión | Ejecuta `python -m pip show ankiforge` y `python -m pip debug --verbose`; comprueba el intérprete y las etiquetas de SO/CPU del wheel |
| Python elige una distribución de código fuente | Elige un wheel y un intérprete compatibles, o sigue [la compilación desde el código fuente](/docs/development/) con un compilador |

Consulta [los entornos verificados](/docs/compatibility/), [la configuración de Node](/docs/node-quickstart/) y [la configuración de Python](/docs/python-quickstart/).

<!-- section: models-and-notes -->
## Modelos y notas

| Síntoma | Corrección |
| --- | --- |
| Clave no válida o vacía | Indica una clave estable explícita; los nombres visibles son independientes |
| `NOTE.KEY_DUPLICATE` | Asigna una clave única a cada registro lógico de origen |
| `NOTE.FIELD_UNKNOWN` | Usa la clave del campo, no su nombre visible |
| `NOTE.FIELD_REQUIRED` | Rellena los campos declarados con `required()` antes de añadir la nota |
| `NOTE.MODEL_CONFLICT` | Reutiliza el mismo modelo finalizado para una misma clave de modelo |
| Error de referencia de plantilla | Utiliza las claves declaradas en las plantillas; inspecciona el intervalo de bytes del origen |
| El HTML aparece como texto literal | Usa `Content::html` solo cuando quieras incluir marcado |
| Faltan tarjetas Cloze | Indica la sintaxis `{{c1::answer}}` con números admitidos del 1 al 500 |

El fallo de una adición es atómico. Corrige la nota o el modelo y vuelve a intentarlo; no hay que deshacer ningún registro parcial de modelos o recursos multimedia. Los constructores de modelos deben terminar con `.build()` antes de poder crear notas personalizadas.

<!-- section: media-and-templates -->
## Recursos multimedia y plantillas

| Síntoma | Corrección |
| --- | --- |
| Falla la importación de un archivo | Comprueba el origen al llamar a `Media::file`; después de importarlo se puede eliminar el archivo de origen |
| El MIME no coincide con el contenido | Pasa el tipo MIME real al constructor de bytes |
| Conflicto de nombres de exportación | Elige nombres portables únicos; las variantes de mayúsculas o normalización Unicode también entran en conflicto |
| Falta un recurso de HTML, CSS o script sin procesar | Decláralo con `asset` en el modelo o con `add_asset` en el proyecto |
| La importación multimedia supera un límite | Aumenta explícitamente `MediaLimits.max_bytes` antes de importar |
| La inspección de generación supera un límite | Ajusta el contador de `InspectLimits` pertinente para la operación |
| Falla la carga de un paquete de plantillas | Usa bundle-v2, claves explícitas, plantillas que referencien claves y rutas dentro del paquete |
| Falla un rectángulo de oclusión de imágenes | Comprueba las dimensiones decodificadas que se muestran, las coordenadas finitas y las claves únicas de las máscaras |

Los valores de texto predeterminados son uniformes en todos los tipos de nota. Los nodos multimedia incorporan sus dependencias; escribir un nombre de archivo a mano no importa sus bytes. Un paquete válido sigue necesitando pruebas de reproducción en el cliente con el códec o la fuente que quieras utilizar.

<!-- section: updates-and-outputs -->
## Actualizaciones y archivos de salida

Usa como base un APKG original de distribución que contenga pruebas completas de identidad. Los paquetes reexportados desde Anki no conservan esas pruebas. Una política de actualización no puede aceptar un espacio de nombres distinto ni pruebas dañadas.

Una comparación completada puede tener `allows_publication == false`. Inspecciona los hallazgos originales y sus pruebas antes de elegir una excepción explícita para una categoría de riesgo. No autorices todos los resultados sin revisarlos. Debes pasar la misma política a la generación real de la actualización si quieres obtener la misma decisión. Establecer una política en una solicitud que solo crea un paquete es un error.

Si desaparece una ruta temporal, conserva el objeto de salida o del archivo hasta que todos los lectores hayan terminado. Los informes y las instantáneas JSON no mantienen la propiedad de los archivos. En los fallos de persistencia, inspecciona la fase de publicación y la durabilidad del error: puede devolverse un error después de que el destino ya se haya publicado. Consulta [las garantías de salida](/docs/build-guarantees/).
