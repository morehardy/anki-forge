# Compatibilidad de Anki Forge y entornos verificados

Canonical page: https://ankiforge.dev/es/docs/compatibility/
Language: es
Description: Consulta los paquetes públicos, los entornos probados y las condiciones de importación y reproducción en Anki.

## Paquetes y entornos verificados

Los paquetes públicos de Rust, npm y PyPI son de la versión `0.3.1`, según la comprobación del 2026-10-13. Empieza con [Rust](https://ankiforge.dev/es/docs/installation/), [Node / TypeScript](https://ankiforge.dev/es/docs/node-quickstart/) o [Python](https://ankiforge.dev/es/docs/python-quickstart/).

| Interfaz | Paquete | Entorno de ejecución y verificación |
| --- | --- | --- |
| Rust | [ankiforge 0.3.1](https://crates.io/crates/ankiforge) | Rust 1.92.0 y la versión estable actual; aplicaciones consumidoras del paquete en las cuatro plataformas admitidas, además de una aplicación nueva que usa el paquete público de crates.io |
| Node / TypeScript | [ankiforge 0.3.1](https://www.npmjs.com/package/ankiforge) | Node 22.13.0, las versiones actuales de las ramas 22, 24 y 26; 16 comprobaciones de instalación pública desde npm, ESM/CJS y TypeScript |
| Python | [ankiforge 0.3.1](https://pypi.org/project/ankiforge/) | CPython convencional 3.11–3.14; 16 comprobaciones aisladas de instalación de wheels públicos de PyPI |

Los paquetes nativos publicados cubren macOS arm64/x64, Linux x64 GNU/glibc y Windows x64. Que un paquete esté disponible no significa que se haya probado correctamente su ejecución en todos los equipos. La [cobertura de Node](https://github.com/morehardy/anki-forge/blob/main/bindings/node/COVERAGE.md), la [cobertura de Python](https://github.com/morehardy/anki-forge/blob/main/bindings/python/COVERAGE.md) y las pruebas de publicación detallan las comprobaciones exactas. Las ejecuciones correctas de la publicación `0.3.1` son [Rust](https://github.com/morehardy/anki-forge/actions/runs/38371221696), [npm](https://github.com/morehardy/anki-forge/actions/runs/38371222114) y [PyPI](https://github.com/morehardy/anki-forge/actions/runs/38371222047). Otras implementaciones de Python, los intérpretes sin GIL y los subintérpretes necesitan validación por separado. Los navegadores, Electron, Bun, Alpine/musl y Linux/Windows ARM64 quedan fuera de la verificación registrada de Node.

Las guías de [importación multimedia por lotes](https://ankiforge.dev/es/docs/media/#batch-import-from-source) y [publicación preparada](https://ankiforge.dev/es/docs/updates/#build-once-review-then-publish-from-source) están disponibles en los paquetes públicos `0.3.1`.

## Importación y reproducción en Anki

La exportación normal no necesita que Anki esté instalado. La importación, la representación y la reproducción multimedia siguen dependiendo de la versión, los códecs y la configuración de importación del cliente de destino. El benchmark del 2026-10-08 superó 40 comprobaciones de importación, contenido y representación de ejemplos en Anki (20 por implementación), pero no comprueba todos los clientes ni todos los comportamientos de la interfaz gráfica o del audio.

Para las actualizaciones, conserva el APKG original de distribución anterior. Las reexportaciones de Anki no mantienen las pruebas de identidad necesarias. Omitir una nota de un paquete no la elimina de la colección de quien estudia. Los cambios de campos o plantillas, los campos de ordenación, las ediciones locales y el historial de repaso requieren pruebas específicas de cada cliente.

Utiliza [los escenarios manuales de cliente](https://github.com/morehardy/anki-forge/blob/main/docs/manual-validation/anki-desktop-v1/TEMPLATE.md) y la versión y configuración exactas de Anki de cada referencia de validación para evaluar tu propia distribución. Lee [las actualizaciones](https://ankiforge.dev/es/docs/updates/) y [las garantías de generación](https://ankiforge.dev/es/docs/build-guarantees/) para conocer las condiciones del cliente y el comportamiento de publicación de cada archivo.
