# Resumen de la API pública

Anki Forge es una biblioteca de código abierto para generar APKG de Anki, con un núcleo en Rust e interfaces para Rust, Python y Node.js. Utiliza el paquete público `ankiforge` para crear notas, empaquetar recursos multimedia y revisar las actualizaciones antes de distribuir un nuevo mazo. Se distribuye con licencia MIT.

| Tarea | Rust | Python | Node / TypeScript |
| --- | --- | --- | --- |
| Crear notas y modelos | [API de Rust](/docs/rust-api/) | [API de Python](/docs/python-api/) | [API de Node](/docs/node-api/) |
| Imágenes, audio y oclusión | [Recursos multimedia](/docs/media/) | [Gestión de recursos multimedia](/docs/python-api/#media-and-occlusion) | [Gestión de recursos multimedia](/docs/node-api/#media-and-image-occlusion) |
| Diagnosticar fallos | [Solución de problemas](/docs/troubleshooting/) | [Diagnósticos](/docs/python-diagnostics/) | [Errores](/docs/node-api/#errors) |
| Comparar y publicar | [Flujo de actualización](/docs/updates/) | [Generación y comparación](/docs/python-api/) | [Generación y comparación](/docs/node-api/#build-compare-and-output) |

<!-- section: stable-note-identity -->
## Identidad estable de las notas

Mantén un espacio de nombres de proyecto y una clave de nota estables para cada nota lógica. Actualizar una versión existente también requiere su APKG original de distribución. Una reexportación desde Anki no sustituye las pruebas completas de identidad de ese paquete original.

<!-- section: start-with-a-task -->
## Empieza por una tarea

[Elige tu lenguaje](/docs/languages/) o sigue la [receta completa de generación y actualización desde JSON](/docs/json-to-anki/). Consulta [compatibilidad](/docs/compatibility/) para conocer los entornos de ejecución publicados y [garantías de salida](/docs/build-guarantees/) para entender quién mantiene vivos los archivos.
