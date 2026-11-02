# Anki Forge frente a genanki: cuál elegir

Canonical page: https://ankiforge.dev/es/docs/anki-forge-vs-genanki/
Language: es
Description: Compara la creación de APKG, los entornos de ejecución, la revisión de actualizaciones y los costes de migración de Anki Forge y genanki.

Considera Anki Forge si estás diseñando un flujo de publicación reproducible y necesitas conservar los datos multimedia sin depender de los archivos originales, obtener diagnósticos estructurados y revisar las comparaciones y la política de riesgos antes de publicar actualizaciones. Conserva un flujo de genanki ya probado si sus scripts de Python, plantillas y recursos multimedia cubren tus necesidades y migrar supone más coste que beneficio.

| Criterio | Anki Forge | genanki |
| --- | --- | --- |
| Lenguaje de creación | Núcleo en Rust; interfaces de Rust, Python y Node / TypeScript | Python |
| Funciones básicas compartidas | Basic, Cloze, plantillas personalizadas, recursos multimedia y generación de APKG sin conexión | Basic, Cloze, plantillas personalizadas, recursos multimedia y generación de APKG sin conexión |
| Identidad | Espacio de nombres y claves de nota explícitos; conserva el APKG original para actualizar | GUID configurables e ID numéricos de modelo y mazo; admite actualizaciones al volver a importar |
| Antes de publicar | Resultados de comparación, política de riesgos y candidato preparado e inspeccionado | Tu script y tus pruebas definen la revisión previa a la publicación |
| Gestión de recursos multimedia | Las imágenes y los sonidos tipados conservan una copia de sus datos | El script proporciona los archivos multimedia del paquete y sus referencias |
| Oclusión de imágenes por código | Máscaras rectangulares con claves mediante la API pública | Evalúa un modelo o una extensión adecuados para tu flujo |
| Coste de ejecución | Dependencias nativas; requiere una plataforma y un intérprete compatibles | Puede resultar más económico conservar una instalación de Python existente |
| Coste de migración | Valida explícitamente la correspondencia con la identidad anterior, las plantillas y las importaciones | No hay coste de migración si el flujo de genanki ya funciona |

## La misma tarea sencilla

Para convertir tres preguntas JSON en un mazo con una imagen y un sonido, ambas bibliotecas permiten que Python analice el JSON y cree las notas. Con genanki, elige los ID del modelo y del mazo, asigna valores a los campos y reglas de GUID, añade los archivos multimedia al paquete y escribe el APKG. Con Anki Forge, elige un espacio de nombres y claves de negocio persistentes, incorpora a las notas recursos multimedia cuyos datos conserva el proyecto, genera v1 y después compara y prepara v2 usando el APKG original v1 antes de publicarla. El [tutorial completo de JSON](https://ankiforge.dev/es/docs/json-to-anki/) incluye las implementaciones de Rust, Python y TypeScript y sus comandos de ejecución.

## Mazos de genanki existentes

[genanki admite GUID personalizados](https://github.com/kerrickstaley/genanki#note-guids), recursos multimedia y actualizaciones al volver a importar. No es correcto presentar la identidad estable como una función exclusiva de Anki Forge. La regla predeterminada de GUID puede depender de los campos; elige una regla deliberada si estos van a cambiar.

Anki Forge no recupera automáticamente ID numéricos ni GUID arbitrarios de genanki. Sin una correspondencia validada por separado, trata tu primera distribución de Anki Forge como el comienzo de una nueva identidad. Ni esta comparación ni la generación correcta de un APKG prometen una migración sin pérdidas ni la conservación de todo el historial de repaso. Lee la [guía de migración](https://ankiforge.dev/es/docs/genanki-migration/) antes de trasladar un mazo existente.

## Datos históricos de rendimiento

La comparación del 2026-10-08 midió la API Project de Rust frente a genanki 0.13.1 en un Apple M1 Pro. Con 1.000 notas Basic, las cinco cargas de trabajo de Rust medidas emplearon entre un 45,7 y un 53,9 % menos de tiempo de exportación; algunas cargas con recursos multimedia usaron más memoria máxima. Consulta el [método, los resultados originales y las limitaciones](https://github.com/morehardy/anki-forge/blob/bef4aeb653fc875f216614e73d8a617be039b9cf/benchmarks/results/20261008-latest-commit-genanki/report.md). La prueba no midió el rendimiento actual de los procesos de Python o Node y no ofrece una garantía equivalente de velocidad para esos SDK ni para otras cargas de trabajo.

Responsables de Anki Forge · revisión del 2026-10-25 según las API públicas de Anki Forge 0.3.1, la [documentación de genanki](https://github.com/kerrickstaley/genanki) y la medición archivada. Consulta la [compatibilidad del entorno de ejecución](https://ankiforge.dev/es/docs/compatibility/) y [elige un lenguaje](https://ankiforge.dev/es/docs/languages/) para empezar una tarea nueva.
