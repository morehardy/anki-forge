# Diagnósticos de Python y política de actualización

Una generación correcta devuelve BuildOutput con un archivo de salida garantizado. Una generación fallida lanza BuildError con observaciones y datos sobre la publicación. Los diagnósticos por sí solos nunca determinan el éxito.

```python
import json
from ankiforge import Project, Note, BuildOptions, BuildError

project = Project('course').add('term', Note.basic('Question', 'Answer'))
try:
    output = project.build(BuildOptions.to('course.apkg'))
except BuildError as error:
    print(error.kind, error.code)
    print(json.dumps(error.snapshot(), ensure_ascii=False))
else:
    print(output.artifact.path)
    print(json.dumps(output.report.snapshot(), ensure_ascii=False))
```

Utiliza los valores procesables por máquina `kind` y `code` para tomar decisiones, no analices los mensajes. Todas las excepciones públicas de dominio conservan el texto de la cadena de errores de origen en `causes` y encadenan su excepción nativa. `details` contiene pruebas específicas de la operación: ubicaciones en el código de las plantillas, rutas multimedia y medidas de límites, observaciones de comparaciones incompletas o datos sobre la publicación. Cuando está disponible, un error de E/S de origen se convierte en un OSError encadenado. `source_details` conserva los errores anidados tipados: las causas relacionadas con el límite de texto de los paquetes de plantillas incluyen `limit` y `observed`, mientras que las causas multimedia incluyen su `code`, `path` y `limit_exceeded`.

BuildReport solo contiene observaciones. Su instantánea tiene recuentos, diagnósticos, tiempo transcurrido y cualquier comparación completada. Las instantáneas de Output y BuildError contienen el resultado real Success/Failure. Las instantáneas de fallo distinguen entre un destino que nunca se publicó y uno que se publicó antes de un fallo de durabilidad. Las rutas JSON no mantienen vivos archivos ni prolongan la vida útil de los archivos temporales de salida.

<!-- section: compare-before-publication -->
## Comparar antes de publicar

```python
from ankiforge import CompareOptions, UpdatePolicy

comparison = project.compare(CompareOptions.against('course-v1.apkg'))
print(comparison.findings)
print(comparison.policy)
```

Una comparación completada devuelve el resultado normalmente aunque detecte riesgo High. Comprueba `comparison.allows_publication` para conocer la decisión de la política. Un paquete de referencia ilegible, la falta de pruebas completas de identidad, una versión candidata no válida o un límite excedido provocan CompareError; su `.report` contiene las observaciones completadas antes del fallo.

La política predeterminada bloquea los hallazgos High y Critical. Después de revisar una categoría de riesgo concreta, su aceptación es explícita:

```python
policy = UpdatePolicy().allow('RISK.NOTE_REMOVED')
comparison = project.compare(CompareOptions.against('course-v1.apkg').update_policy(policy))
output = project.build(BuildOptions.to('course-v2.apkg')
    .update_from('course-v1.apkg').update_policy(policy))
```

La aceptación abarca todos los hallazgos de la categoría indicada. Los hallazgos conservan su riesgo y sus pruebas originales. Las autorizaciones de riesgo que no correspondan a ningún hallazgo producen advertencias; las advertencias no convierten una publicación correcta en un fallo. No se pueden aceptar códigos desconocidos, pruebas ausentes ni otros errores que impiden continuar. No aceptes automáticamente todas las categorías devueltas por una comparación.

`update_from` requiere el paquete original de distribución anterior, incluidas sus pruebas completas integradas. Las reexportaciones desde Anki no conservan esas pruebas. Omitir notas o tarjetas en un APKG no las elimina automáticamente de la colección del estudiante. Revisa los cambios estructurales según la configuración de importación de Anki prevista.

<!-- section: budgets-and-artifact-ownership -->
## Límites y propiedad del archivo de salida

```python
from ankiforge import InspectLimits

limits = InspectLimits(max_archive_bytes=3 << 30)
options = BuildOptions.to('course-v2.apkg').update_from('course-v1.apkg').inspect_limits(limits)
```

Los límites se aplican por separado a cada paquete de referencia y versión candidata inspeccionados. La importación multimedia tiene su propio límite anterior, MediaLimits. Si se excede un límite, los datos del recurso, el límite y el valor observado se conservan en `details['limit_exceeded']`; aumenta deliberadamente el límite correspondiente y vuelve a intentarlo. Para paquetes de plantillas, pasa `NoteType.from_bundle(path, limits=MediaLimits(...))` para ajustar el límite de lectura de cada recurso; el manifiesto y el texto de las plantillas conservan sus propios límites fijos. Las inspecciones obligatorias no se pueden desactivar.

Utiliza `BuildOptions.temporary()` cuando no quieras una ruta persistente. Conserva la salida o un identificador de su archivo mientras lo utilizas; `artifact.persist_to(path)` crea una copia persistente. Si la persistencia falla, los detalles de PersistError registran la etapa de publicación del destino y el identificador original sigue siendo utilizable.

Las rutas de las instantáneas y de los detalles de errores son cadenas cuando contienen Unicode válido. Los nombres Unix en bytes que Python representa mediante `surrogateescape` aparecen en JSON como `{"encoding": "unix_bytes", "bytes": [...]}`; restáuralos con `Path(os.fsdecode(bytes(value["bytes"])))`. Los nombres Windows con sustitutos UTF-16 no emparejados usan `{"encoding": "windows_wide", "units": [...]}`, donde cada entrada del array contiene una unidad de código original de 16 bits. Las rutas codificadas describen su plataforma original. Durante la ejecución, `artifact.path` sigue siendo un `pathlib.Path`, y `BuildOptions.to`, `update_from` y `CompareOptions.against` conservan los valores de ruta nativos a través del enlace.

Una extensión nativa ausente o incompatible provoca un error de importación que identifica `BINDING.EXTENSION_UNAVAILABLE` o `BINDING.VERSION_MISMATCH`. Instala un wheel compatible o vuelve a compilar una copia del código fuente con maturin. Las operaciones concurrentes sobre un mismo Project se rechazan; utiliza proyectos independientes para trabajar en paralelo.
