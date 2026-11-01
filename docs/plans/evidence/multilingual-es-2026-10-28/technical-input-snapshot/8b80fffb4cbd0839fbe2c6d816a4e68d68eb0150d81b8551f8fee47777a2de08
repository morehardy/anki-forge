# Crea tarjetas de Anki con ayuda de un asistente de IA

Usa un asistente de IA para preparar pares de preguntas y respuestas a partir de tus apuntes, compruébalos con el material original y utiliza Anki Forge para exportar un mazo `.apkg`. En este flujo, Anki Forge es una biblioteca de código abierto para generar paquetes de Anki: el asistente prepara contenido o escribe un script, y tu programa local de Python, Node.js o Rust genera las tarjetas.

Puedes [copiar las instrucciones para un asistente de programación](/#ai-quickstart) y empezar con un lenguaje, o seguir el flujo CSV que aparece a continuación. El generador de instrucciones del sitio compone el texto localmente; no llama a ningún modelo de IA ni sube tus apuntes.

<!-- section: choose-input -->
## Empieza con material que puedas comprobar

Usa un fragmento breve sobre un tema claro, como tus propios apuntes de clase o una lista de vocabulario. Si partes de PDF, documentos escaneados o diapositivas, extrae y revisa el texto primero con otra herramienta. Anki Forge recibe el contenido que proporcionas mediante sus API; la extracción de PDF y el OCR quedan fuera de esta biblioteca.

Haz que cada pregunta propuesta se centre en un solo dato. Añade contexto suficiente para que la respuesta tenga sentido cuando la tarjeta aparezca fuera del documento original.

<!-- section: draft-cards -->
## Pide tarjetas en formato CSV

Pega este ejemplo en tu asistente. Al crear tu propio mazo, sustituye el fragmento de origen y ajusta el número de notas solicitado y los ID de las notas nuevas:

```text
Turn the source below into exactly three Basic Anki notes.
Use only facts stated in the source. Do not add facts from memory.
Each question must test one fact and be understandable on its own.
Return only UTF-8 CSV with the header id,front,back, with no Markdown fence.
Use the IDs es:hola, es:gracias and es:adios for these three vocabulary items.
Quote any CSV field containing a comma, quote or newline correctly.

Source:
In Spanish, hola means hello, gracias means thank you, and adiós means goodbye.
```

También puedes dar estas instrucciones en español; conserva el encabezado `id,front,back`, los ID y las reglas de revisión.

El contenido esperado equivale a lo siguiente:

```csv
id,front,back
es:hola,hola,hello
es:gracias,gracias,thank you
es:adios,adiós,goodbye
```

El asistente puede formular el anverso de otra manera. Comprueba cada fila con el material original, elimina las respuestas ambiguas o sin respaldo y guarda el resultado revisado como `cards.csv`. Asigna un ID único a cada nota nueva y consérvalo cuando la edites; no pidas al asistente que vuelva a generar los ID de un mazo existente.

<!-- section: export-apkg -->
## Convierte las tarjetas revisadas en un APKG

Sigue [CSV a Anki con Python](/docs/csv-to-anki/) para obtener el programa completo, los comandos de instalación y los datos de ejemplo. Ese script comprueba la estructura del CSV, exige ID únicos y escribe un paquete persistente. El ejemplo crea tres tarjetas Basic sin recursos multimedia.

Si tus datos ya están en JSON o necesitas imágenes y audio, utiliza el [tutorial de JSON a Anki](/docs/json-to-anki/), con programas completos de Python, Node / TypeScript y Rust. Tu aplicación analiza el CSV o el JSON antes de pasar las notas a Anki Forge.

Para que un asistente te ayude a escribir un programa a medida, usa las [instrucciones para el asistente de programación](/#ai-quickstart) y describe los campos de entrada, el tipo de tarjeta y el nombre del archivo de salida. La [Skill opcional de Anki Forge](https://github.com/morehardy/anki-forge/blob/main/docs/ai-assistants.md) ofrece referencias de la API pública y flujos de trabajo para asistentes que cargan Skills.

<!-- section: check-result -->
## Revisa las tarjetas antes de estudiar

Abre el APKG generado en Anki y revisa el número de tarjetas, las preguntas, las respuestas y el formato. En el ejemplo CSV, debes obtener tres notas y tres tarjetas en el mazo Spanish. Una exportación correcta confirma que se ha generado un paquete; no demuestra que las respuestas de la IA sean correctas ni útiles para estudiar.

Para preguntas de completar huecos, utiliza [tarjetas Cloze](/docs/cards/). Para las etiquetas de un diagrama, usa [oclusión de imágenes](/docs/image-occlusion/) con la imagen y las coordenadas de las máscaras que proporciones; esta función no detecta automáticamente las etiquetas de una imagen.

Conserva el APKG original de distribución para la siguiente versión y sigue el [flujo de actualización](/docs/updates/). Mantén el espacio de nombres y los ID de nota, inspecciona la comparación y ten en cuenta las opciones de importación de Anki y las modificaciones locales más recientes.
