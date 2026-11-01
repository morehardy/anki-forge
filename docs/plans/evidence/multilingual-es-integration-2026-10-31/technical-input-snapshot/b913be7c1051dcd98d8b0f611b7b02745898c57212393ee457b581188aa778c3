# Convierte CSV en tarjetas de Anki con Python

Convierte una hoja de vocabulario en un archivo `.apkg` de Anki con Python y Anki Forge. Este ejemplo lee un CSV UTF-8 con las columnas `id`, `front` y `back`, crea tarjetas Basic de pregunta y respuesta y exporta un mazo que puedes compartir. El lector de CSV de la biblioteca estándar de Python lee la hoja de cálculo; Anki Forge genera el paquete. No necesitas instalar Anki para exportarlo.

Para importar una hoja de cálculo una sola vez, Anki ya admite [archivos de texto delimitado](https://docs.ankiweb.net/manual/importing/text-files). Usa este tutorial si necesitas un script que puedas volver a ejecutar, tus propios ID de nota y un APKG para distribuir. Primero debes guardar los archivos Excel `.xlsx` como CSV UTF-8.

<!-- section: prepare-csv -->
## Prepara el CSV

Guarda lo siguiente como `cards.csv` o [descarga el ejemplo](/generated/recipes/csv-to-anki/cards.csv):

<!-- source: docs/recipes/csv-to-anki/cards.csv -->
```csv
id,front,back
es:hola,hola,hello
es:gracias,gracias,thank you
es:adios,adiós,goodbye
```
<!-- /source -->

Conserva la cabecera tal como aparece y usa comas como separadores. Pon entre comillas los campos que contengan comas, comillas dobles o saltos de línea según las [reglas de CSV](https://docs.python.org/3/library/csv.html). El script admite una marca de orden de bytes UTF-8, presente en algunas exportaciones de hojas de cálculo. Rechaza los ID duplicados, los valores ausentes, las columnas adicionales y la entrada vacía.

Asigna cada ID una sola vez y consérvalo cuando cambien la pregunta o la respuesta. No lo derives de la posición de una fila ni del texto editable de la tarjeta. Usa un espacio de nombres distinto para conjuntos de datos sin relación; conserva el mismo espacio de nombres para las sucesivas versiones de este mazo.

<!-- section: install -->
## Instala Anki Forge

Usa CPython estándar 3.11 o posterior en una [plataforma compatible](/docs/compatibility/). Crea y activa un entorno virtual:

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

En Windows, actívalo con `.venv\Scripts\Activate.ps1`. Esta conversión no necesita una biblioteca de hojas de cálculo ni un servicio de IA.

<!-- section: convert -->
## Genera el APKG

Guarda lo siguiente como `csv_to_anki.py` junto a `cards.csv` o [descarga el script](/generated/recipes/csv-to-anki/csv_to_anki.py):

<!-- source: bindings/python/examples/csv_to_anki.py -->
```python
"""Convert a UTF-8 id,front,back CSV into a persistent Basic-card APKG."""
import argparse
import csv
from pathlib import Path

from ankiforge import BuildOptions, Note, Project


def build_csv(source: Path, destination: Path, namespace: str, deck: str) -> int:
    if destination.exists():
        raise ValueError('Choose a new output path; keep previously distributed APKGs')
    project = Project(namespace, default_deck=deck)
    seen = set()
    with source.open(encoding='utf-8-sig', newline='') as handle:
        rows = csv.DictReader(handle, strict=True)
        if rows.fieldnames != ['id', 'front', 'back']:
            raise ValueError('Expected the header id,front,back in that order')
        for row in rows:
            if None in row or any(value is None or not value.strip() for value in row.values()):
                raise ValueError(f'Line {rows.line_num}: expected three nonempty fields')
            key = row['id'].strip()
            if key in seen:
                raise ValueError(f'Line {rows.line_num}: duplicate id {key!r}')
            project.add(key, Note.basic(row['front'], row['back']))
            seen.add(key)
    if not seen:
        raise ValueError('The CSV must contain at least one note')
    project.build(BuildOptions.to(destination))
    return len(seen)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('destination', type=Path)
    parser.add_argument('--namespace', default='csv-vocabulary')
    parser.add_argument('--deck', default='Vocabulary')
    args = parser.parse_args()
    count = build_csv(args.source, args.destination, args.namespace, args.deck)
    print(f'{args.destination}: {count} notes, {count} Basic cards')
```
<!-- /source -->

Ejecútalo desde ese directorio:

```sh
python csv_to_anki.py cards.csv vocabulary-v1.apkg --namespace spanish-vocabulary --deck Spanish
```

El ejemplo genera `vocabulary-v1.apkg` con **tres notas y tres tarjetas Basic** en el mazo **Spanish**, sin recursos multimedia. Muestra este resultado:

```text
vocabulary-v1.apkg: 3 notes, 3 Basic cards
```

Elige una ruta de salida que no exista. El script se niega a sustituir un archivo anterior para que puedas conservar el paquete original de distribución.

<!-- section: verify-and-extend -->
## Importa, comprueba y amplía

Abre `vocabulary-v1.apkg` en Anki. Comprueba que el mazo contiene `hola → hello`, `gracias → thank you` y `adiós → goodbye`; previsualiza las tarjetas y verifica las tildes. Revisa el contenido antes de distribuirlo, especialmente si una IA preparó el borrador del CSV.

Este tutorial crea tarjetas Basic solo de texto. Para otros flujos de trabajo:

- [Creación de tarjetas de Anki con ayuda de IA](/docs/ai-anki-cards/): prepara y revisa filas CSV a partir de tus apuntes.
- [JSON a Anki](/docs/json-to-anki/): usa Python, Node / TypeScript o Rust, incluye imágenes y audio y revisa una segunda versión.
- [Tarjetas Basic y Cloze](/docs/cards/): elige tarjetas de pregunta y respuesta o de completar huecos.
- [Actualización de mazos de Anki generados](/docs/updates/): usa el APKG original anterior como base de la actualización e inspecciona la comparación.

Para publicar una versión posterior, conserva el espacio de nombres y los ID de nota y sigue el flujo de actualización antes de publicarla. Los ID estables ayudan a identificar las notas; las opciones de importación de Anki y las modificaciones locales más recientes también afectan a las actualizaciones.
