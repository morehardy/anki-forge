# Genera un APKG de Anki con Python

Canonical page: https://morehardy.github.io/anki-forge/es/docs/python-quickstart/
Language: es
Description: Instala el wheel publicado de Python y crea tu primer archivo APKG de Anki con un programa completo y claves de nota estables.

Crea y activa un entorno virtual e instala el [paquete publicado en PyPI](https://pypi.org/project/ankiforge/):

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

En Windows, actívalo con `.venv\Scripts\Activate.ps1`. Usa CPython estándar 3.11 o posterior y un wheel que corresponda a tu plataforma; las comprobaciones de publicación registradas cubren CPython 3.11–3.14. Consulta los [entornos verificados](https://morehardy.github.io/anki-forge/es/docs/compatibility/).

## Exporta un mazo

Guarda lo siguiente como `main.py`:

```python
from ankiforge import Project, Note, BuildOptions

project = Project('spanish', default_deck='Spanish')
project.add('es:hola', Note.basic('hola', 'hello'))
output = project.build(BuildOptions.to('spanish.apkg'))
print(output.artifact.path)
```

```sh
python main.py
```

Abre el archivo persistente `spanish.apkg` en Anki. Contiene **hola → hello** en el mazo **Spanish**. Conserva el espacio de nombres `spanish` y la clave de nota `es:hola` cuando edites el contenido. No necesitas instalar Anki para exportarlo.

## Continúa

Si partes de una hoja de vocabulario, sigue [CSV a Anki](https://morehardy.github.io/anki-forge/es/docs/csv-to-anki/). Para preparar borradores de tarjetas a partir de tus apuntes, empieza con la [creación de tarjetas con ayuda de IA](https://morehardy.github.io/anki-forge/es/docs/ai-anki-cards/).

Para completar una tarea con recursos multimedia y actualizaciones, sigue [JSON a Anki](https://morehardy.github.io/anki-forge/es/docs/json-to-anki/#python).

-   [Imágenes y audio](https://morehardy.github.io/anki-forge/es/docs/media/) y [API multimedia de Python](https://morehardy.github.io/anki-forge/es/docs/python-api/#media-and-occlusion).
-   [Tipos de nota personalizados](https://morehardy.github.io/anki-forge/es/docs/custom-notetypes/) y [paquetes de plantillas](https://morehardy.github.io/anki-forge/es/docs/templates/).
-   [Oclusión de imágenes](https://morehardy.github.io/anki-forge/es/docs/image-occlusion/).
-   [Comparación y actualización](https://morehardy.github.io/anki-forge/es/docs/updates/).
-   [API de Python](https://morehardy.github.io/anki-forge/es/docs/python-api/) y [diagnósticos](https://morehardy.github.io/anki-forge/es/docs/python-diagnostics/).
-   [Migración desde genanki](https://morehardy.github.io/anki-forge/es/docs/genanki-migration/).

Para compilaciones nativas desde el repositorio, consulta la [guía de compilación desde el código fuente](https://morehardy.github.io/anki-forge/es/docs/development/).
