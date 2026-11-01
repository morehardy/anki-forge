Usa Rust, Node / TypeScript o Python para generar mazos de Anki a partir de tus datos. Los tres pueden exportar un archivo persistente `spanish.apkg` sin instalar Anki.

| Lenguaje | Paquete publicado | Instalación | Primer mazo completo |
| --- | --- | --- | --- |
| Rust | [ankiforge 0.3.1](https://crates.io/crates/ankiforge) | `cargo add ankiforge@0.3.1` | [Inicio rápido con Rust](/docs/quickstart/) |
| Node / TypeScript | [ankiforge 0.3.1](https://www.npmjs.com/package/ankiforge) | `npm install --include=optional ankiforge@0.3.1` | [Inicio rápido con Node](/docs/node-quickstart/) |
| Python | [ankiforge 0.3.1](https://pypi.org/project/ankiforge/) | `python -m pip install ankiforge==0.3.1` | [Inicio rápido con Python](/docs/python-quickstart/) |

Rust requiere la versión 1.92 o posterior. Node requiere Node 22.13 o posterior y la dependencia nativa opcional correspondiente. Python utiliza un wheel nativo para un intérprete CPython estándar; las comprobaciones de publicación registradas cubren CPython 3.11–3.14. Consulta los [entornos verificados](/docs/compatibility/).

<!-- section: next-tasks -->
## Qué hacer después

Genera y actualiza un mazo a partir de [archivos compartidos de JSON, imagen y audio](/docs/json-to-anki/), o compara [Anki Forge y genanki](/docs/anki-forge-vs-genanki/) según tu forma de trabajar.

Añade [imágenes y audio](/docs/media/), crea [tipos de nota personalizados](/docs/custom-notetypes/) o [compara y actualiza un mazo que ya hayas distribuido](/docs/updates/).

Para TypeScript, consulta la [configuración del compilador](/docs/node-api/#typescript). Si vienes de genanki, lee la [guía de migración](/docs/genanki-migration/). El desarrollo del repositorio y las funciones disponibles solo en el código fuente tienen sus propias [instrucciones de compilación](/docs/development/#source-builds).
