# Pasar de genanki a Anki Forge

¿Todavía estás eligiendo una biblioteca? Lee [Anki Forge frente a genanki](/docs/anki-forge-vs-genanki/).

Anki Forge utiliza claves de datos estables, modelos inmutables validados, instantáneas multimedia con propiedad de sus datos y operaciones explícitas de generación y actualización. Es un modelo de creación nuevo: no emula los objetos de genanki ni las interfaces Deck y de archivos de bloqueo retiradas de Anki Forge.

| Concepto de genanki | Equivalente en Anki Forge |
| --- | --- |
| Modelo con ID numérico | `NoteType.builder(stable_key)...build()` |
| Nombres de campos como identificadores | Claves estables de Field y nombres de presentación independientes |
| Nota que contiene cadenas para los campos | `model.note().field(key, content)` con un modelo que la nota conserva |
| Mazo como contenedor de notas | Proyecto con un mazo predeterminado; `.deck(name)` opcional para cada nota |
| Rutas a los archivos multimedia del paquete | Valores `Media.file(path)` que mantienen sus datos y se recopilan automáticamente mediante contenido tipado |
| Escritura del paquete | `project.build(BuildOptions.to(path))` |
| GUID predeterminado derivado de los campos o regla de GUID personalizada | Espacio de nombres y clave de nota estables y explícitos |

```python
from ankiforge import Project, NoteType, Field, Template, BuildOptions

model = (NoteType.builder('vocabulary').name('Vocabulary')
    .field(Field('word', name='Word', required=True))
    .field(Field('meaning', name='Meaning'))
    .template(Template('recognition', '{{word}}', '{{FrontSide}}<hr>{{meaning}}'))
    .build())
project = Project('vocabulary-course', default_deck='Languages::Vocabulary')
project.add('dictionary:cell', model.note().field('word', 'cell').field('meaning', '细胞'))
project.build(BuildOptions.to('vocabulary.apkg'))
```

Elige las claves a partir de identificadores persistentes de tus datos de negocio. Cambiar un nombre de presentación o el texto de una definición no debe cambiar esas claves. No calcules un hash del contenido actual de los campos salvo que la identidad basada en contenido sea un requisito explícito de tu importador. Si los datos de origen no tienen claves, asígnalas y guárdalas fuera del SDK.

Las expresiones de las plantillas utilizan claves de Field. Los nombres de presentación pueden traducirse de forma independiente; la biblioteca transforma las referencias por clave en nombres de campos de Anki. Al completar un modelo se rechazan las declaraciones ausentes o incompatibles antes de que se pueda añadir una nota. Añadir notas recopila automáticamente las dependencias del modelo y rechaza definiciones incompatibles bajo una misma clave de modelo.

Las cadenas son texto sin formato; envuelve el HTML intencional en `Content.html(...)`. Utiliza `media.image()` / `media.sound()` y `Content.sequence(...)` para combinar contenido. Las instantáneas de archivos siguen disponibles si se elimina o modifica la ruta original. Declara explícitamente las dependencias a las que hagan referencia HTML, CSS o scripts sin procesar mediante `.asset(media)` en el constructor del modelo o `project.add_asset(media)`.

Para las siguientes versiones, crea un Project nuevo con el mismo espacio de nombres y las mismas claves de datos, y llama a `BuildOptions.to(next_path).update_from(previous_original_apkg)`. El APKG original aporta las pruebas completas de identidad. Este flujo no recupera la identidad de paquetes antiguos arbitrarios ni promete convertir automáticamente los ID numéricos o GUID existentes de genanki. Considera la primera distribución con el nuevo modelo como un cambio deliberado de identidad, salvo que una migración de datos validada por separado establezca las correspondencias.

Consulta la [guía de inicio](/docs/python-quickstart/), la [referencia de la API](/docs/python-api/) y la [política de actualización](/docs/python-diagnostics/).
