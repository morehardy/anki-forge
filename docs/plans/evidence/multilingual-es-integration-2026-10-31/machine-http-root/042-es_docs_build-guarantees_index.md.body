# Garantías de generación y de los archivos APKG

Canonical page: https://ankiforge.dev/es/docs/build-guarantees/
Language: es
Description: Comprende la vida útil de los archivos, los límites de inspección y la publicación de una versión candidata revisada.

`Project::build` solo devuelve `BuildOutput` después de una operación correcta, con un archivo APKG garantizado. Una operación fallida devuelve `BuildError` con tipo y código estructurados, observaciones, causas subyacentes y los datos reales sobre la publicación.

## Archivos temporales y persistentes

Elige explícitamente cómo se conserva el archivo:

| Solicitud | Conservación del archivo |
| --- | --- |
| `BuildOptions::to(path)` | Destino persistente; permanece tras liberar los objetos que lo gestionan |
| `BuildOptions::temporary()` | Archivo temporal; se elimina al liberar el último objeto que lo mantiene vivo |
| `artifact.persist_to(path)` | Nuevo objeto que conserva un archivo persistente; el objeto original sigue siendo utilizable |

Mantén vivo el `BuildOutput` o un clon del objeto del archivo mientras leas una ruta temporal. Un informe o una instantánea JSON no alargan la vida del archivo. Rust libera normalmente estos objetos; Node ofrece `artifact.close()` para liberarlos de forma determinista. Copiar la ruta al estado de la aplicación no crea un objeto que conserve el archivo.

Los objetos de archivo de Rust conservan rutas absolutas, incluso cuando la generación o `persist_to` reciben un destino relativo. Si después cambia el directorio de trabajo del proceso, las lecturas o copias realizadas mediante un objeto existente no se redirigen. Cada nuevo destino de `persist_to` se resuelve con respecto al directorio de trabajo en el momento de esa llamada.

`BuildReport::snapshot()` solo contiene observaciones. `BuildOutput::snapshot()` y `BuildError::snapshot()` indican además el resultado real. Una instantánea correcta puede incluir una advertencia. No deduzcas el resultado a partir de la gravedad de los diagnósticos, de la existencia de una ruta ni de que un informe contenga hallazgos de comparación.

## Bases de actualización y publicación

Para las actualizaciones, conserva el APKG original de distribución anterior como base inmutable y elige otro destino de salida. Las pruebas de la base y el contenido candidato se comprueban antes de publicar. Un rechazo de la política no reemplaza la salida. Las pruebas ausentes o dañadas y el agotamiento de los límites de inspección son fallos.

La publicación reemplaza el APKG de forma atómica. Esta garantía se aplica a cada archivo; escribir un informe JSON separado de la aplicación es otra operación. La generación normal no escribe un archivo de identidad adicional: las pruebas completas con versión de formato están incorporadas en el paquete de distribución.

Puede producirse un error tardío después de publicar, por ejemplo al confirmar la persistencia. Por eso, las instantáneas de error registran los datos de `PublicationSnapshot`: `path`, `stage` (`not_published` o `published`), `temporary` y durabilidad (`confirmed` o `unconfirmed`). Comprueba estos datos antes de reintentar o de anunciar que no se ha escrito ningún archivo. `PersistError` también conserva el hecho de la publicación y la causa real de E/S. El reemplazo atómico, por sí solo, no garantiza la durabilidad después de un fallo del sistema.

La serialización de las instantáneas conserva las rutas nativas sin acceder al sistema de archivos. Las rutas Unicode siguen siendo cadenas JSON. Las rutas Unix que no son UTF-8 utilizan `{"encoding":"unix_bytes","bytes":[...]}`; las rutas de Windows con sustitutos UTF-16 sin pareja utilizan `{"encoding":"windows_wide","units":[...]}`. Los arrays contienen los bytes o las unidades de código nativas exactas. Las rutas de archivos correctos, las rutas de publicación fallida y los detalles de rutas de error de los bindings usan esta representación. Estos valores describen rutas y no mantienen la propiedad de los archivos. Un sistema de archivos puede rechazar una ruta nativa concreta, pero los datos de su fallo de publicación siguen siendo serializables.

## Límites de inspección

Los límites finitos del archivo contenedor, el número de entradas, la expansión, la colección y los recursos multimedia decodificados y la ventana de zstd se aplican por separado a la base y a la versión candidata. Empieza con `InspectLimits::default()` y aumenta deliberadamente el límite pertinente cuando lo necesites. Un límite de cero no significa ilimitado. Estos contadores no aíslan la memoria o la CPU del conjunto del proceso; las importaciones multimedia tienen un límite anterior separado por recurso.

## Trabajo concurrente

Las generaciones en Rust toman prestado el estado inmutable del proyecto. Los modelos y los recursos multimedia cuyos datos se conservan se pueden compartir entre proyectos. El binding de Node captura una instantánea del proyecto al invocar una generación o comparación, por lo que las adiciones posteriores no cambian esa solicitud. Conserva el objeto que mantiene el archivo hasta que todos los lectores hayan terminado; cerrar el último objeto de un archivo temporal mientras otro consumidor todavía necesita su ruta es un error de gestión de vida útil de la aplicación.

La inspección de paquetes no puede determinar todos los comportamientos del cliente. Prueba la representación, la reproducción y las importaciones de actualizaciones con las versiones de Anki y la configuración previstas. Consulta [las actualizaciones](https://ankiforge.dev/es/docs/updates/) para conocer las limitaciones de la programación de repaso y de los cambios de esquema.

## Publicación preparada

La publicación preparada está incluida en los paquetes públicos `0.3.1`.

`prepare_publication` genera, inspecciona y compara una única versión candidata privada; `publish` usa ese mismo archivo. Las opciones y los destinos relativos quedan fijados durante la preparación. Las ediciones del proyecto no pueden cambiar la versión candidata. La preparación por sí sola no reemplaza el destino; liberar o cerrar un objeto de una versión no publicada elimina su archivo.

Cada intento de publicación consume el objeto que mantiene la versión candidata, incluso si falla la política o la E/S. Vuelve a preparar para reintentar o para cambiar la política o el destino. Los informes siguen siendo legibles y no conservan archivos. Node y Python ofrecen `close()` idempotente y un error de indisponibilidad con motivo `closed` o `consumed`; cerrar después de que empiece la publicación no la cancela. La duración no incluye la espera de revisión. Un fallo tardío de durabilidad puede producirse después de un reemplazo correcto, por lo que debes comprobar los datos reales de la publicación.

Sigue [el ejemplo completo](https://ankiforge.dev/es/docs/updates/#build-once-review-then-publish-from-source) y las referencias de las API de cada lenguaje para conocer los errores tipados y el funcionamiento de los trabajadores.
