# Conceptos fundamentales

<!-- section: project-and-note -->
## Proyecto y nota

Un `Project` representa una publicación con un espacio de nombres estable. Puede distribuir notas entre varios mazos de Anki: `default_deck` define el mazo predeterminado y `Note::deck` permite elegir otro para una nota. Un mazo es el destino de las notas, no un segundo contenedor para gestionar el contenido, los recursos multimedia o las compilaciones.

Una `Note` contiene su modelo y su contenido. `project.add(key, note)` le asigna una clave de origen estable y obligatoria, e incorpora sus dependencias de forma atómica. Las claves duplicadas, los campos desconocidos y los modelos o recursos multimedia en conflicto provocan un error sin incorporar un estado parcial. El espacio de nombres del proyecto y las claves de los registros deben mantenerse al editar o reordenar los datos de origen. Los nombres visibles y el texto no son claves de identidad.

<!-- section: models-fields-and-templates -->
## Modelos, campos y plantillas

`NoteType::builder(key)` reúne las declaraciones; `.build()` las valida y devuelve un modelo inmutable que se puede compartir. `model.note()` crea una nota que conserva ese modelo. Las notas integradas Basic, Cloze y de oclusión de imágenes incluyen sus propios modelos.

Los campos y las plantillas tienen claves estables y nombres visibles opcionales. Escribe las plantillas con las claves de los campos, por ejemplo `{{front}}`, aunque el nombre que muestre Anki sea `正面`. Al finalizar el modelo se validan y compilan las referencias, incluidas las secciones, los filtros, Cloze y las plantillas del explorador. Las asignaciones también usan claves: nunca se deduce una clave ni se recurre al nombre visible.

<!-- section: text-and-html -->
## Texto y HTML

Las cadenas normales siempre se interpretan como Text, tanto en notas Basic y Cloze como en notas personalizadas. Text escapa el HTML; `Content::html` incluye marcado de forma intencionada. La sintaxis Cloze, como `{{c1::answer}}`, se conserva en Text sin dar por seguro el HTML que pueda contener. `Content::sequence` combina texto, HTML, imágenes y sonidos en un solo campo.

El contenido tipado de imagen y sonido conserva las dependencias multimedia hasta la exportación. No conviertas esos elementos en cadenas antes de añadirlos, salvo que también declares el recurso explícitamente para tu HTML, CSS o script escrito a mano.

<!-- section: media-ownership -->
## Cómo se conservan los recursos multimedia

`Media` conserva una copia de los datos con un nombre de archivo determinista derivado del contenido. La importación solo termina con éxito después de leer y validar la entrada dentro del límite establecido. A partir de entonces, el archivo de origen puede cambiar o desaparecer. Los clones comparten almacenamiento y pueden usarse en varios proyectos. `with_export_name` permite elegir un nombre de archivo fijo y portable sin cambiar los clones ni los nodos de contenido ya existentes.

Las referencias tipadas incorporan los recursos multimedia automáticamente. Los recursos declarados explícitamente en un modelo o proyecto se incluyen aunque el análisis estático no pueda identificar su uso. La biblioteca no descubre el contenido de los archivos a partir de HTML o scripts arbitrarios.

<!-- section: build-and-update -->
## Generación y actualización

Una generación correcta devuelve un resultado con un APKG garantizado. Un informe contiene observaciones; una instantánea es información JSON y no mantiene ningún archivo en uso. Los archivos temporales solo se conservan mientras exista al menos un objeto que mantenga su propiedad.

Una actualización utiliza las pruebas completas de identidad incorporadas en el APKG original de distribución anterior. La comparación independiente separa los hallazgos de la política de publicación. Los hallazgos de riesgo alto bloquean las actualizaciones de forma predeterminada; los errores no se pueden autorizar mediante excepciones a la política. La configuración de importación de Anki, las ediciones locales y la programación de repaso de las tarjetas siguen requiriendo validación en el cliente. Consulta [las actualizaciones](/docs/updates/) y [las garantías de generación](/docs/build-guarantees/).
