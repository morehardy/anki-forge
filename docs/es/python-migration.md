# Transición a la nueva API de Python

Utiliza `python -m pip install ankiforge` y `from ankiforge import ...`. La antigua distribución `anki-forge` y el nombre de importación `anki_forge` no son alias de compatibilidad. En una copia del código fuente, vuelve a compilar la extensión nativa con Maturin después de cambiar los nombres. Consulta la [guía de instalación](/docs/python-quickstart/).

No se conserva una capa de compatibilidad con la API anterior. Sustituye las antiguas API de Deck, registros y procesos de ejecución por el modelo público que mantiene la propiedad de sus valores:

| Flujo anterior | Flujo actual |
| --- | --- |
| Deck o Project con un ID estable opcional | `Project(namespace, default_deck=...)` |
| `add_note(note)` e identidad derivada del contenido | `add(stable_key, note)` |
| NoteType mutable y add_notetype | `NoteType.builder(key)...build()`, seguido de `model.note()` |
| Referencias a modelos mediante cadenas | Las notas conservan su modelo validado |
| Registro multimedia del proyecto | Instantáneas independientes de `Media.file` / `Media.bytes` |
| Recursos multimedia ya renderizados y métodos de asignación específicos para cada campo | `Content.sequence`, `media.image()`, `media.sound()`, `Note.field` |
| HTML implícito | Las cadenas son Text; utiliza `Content.html` |
| Máscaras sin identidad estable | `Mask.rect(key, x, y, width, height)` |
| Archivo de salida opcional más ensure_success | `BuildOutput` con un archivo garantizado; los fallos lanzan errores estructurados |
| Resultado del informe deducido de los diagnósticos | Las instantáneas de Output/Error proporcionan el resultado real; Report solo contiene observaciones |
| Opciones de diff, archivo de bloqueo y seguridad de actualización | `compare(CompareOptions.against(...))`, `BuildOptions.update_from(...)` |
| Paquete de ejecución en un subproceso | Enlace PyO3 dentro del proceso a la API predeterminada de Rust |

Solo los APKG originales de distribución completos que contienen las pruebas actuales de identidad sirven como base de actualización. Los paquetes de plantillas antiguos y las pruebas de los APKG anteriores no se recuperan automáticamente. Consulta el README y los ejemplos ejecutables para seguir el nuevo flujo.
