# Genera y actualiza mazos de Anki a partir de JSON

Lee tres preguntas de biología desde JSON, añade una imagen y un fragmento de audio compartidos y exporta un APKG original junto con una actualización revisada que solo cambia una respuesta. Cada versión debe contener **3 notas, 3 tarjetas y 2 archivos multimedia**. Elige uno de los SDK siguientes; cada ejemplo es un programa completo. La exportación habitual no requiere instalar Anki.

Mantenimiento: responsables de Anki Forge · revisión del 2026-10-25.

<!-- section: shared-inputs -->
## Archivos de entrada compartidos

Descarga [v1.json](/generated/recipes/json-to-anki/v1.json), [v2.json](/generated/recipes/json-to-anki/v2.json), [cell.png](/generated/recipes/json-to-anki/cell.png) y [tone.wav](/generated/recipes/json-to-anki/tone.wav). La imagen es un diagrama de una célula; el audio es un tono breve generado, no una palabra pronunciada. En cada directorio nuevo de aplicación, coloca estos cuatro archivos en `recipe/`:

```text
json-deck/
  recipe/
    v1.json
    v2.json
    cell.png
    tone.wav
```

Ambos archivos JSON contienen `namespace`, `deck` y tres `notes` con valores persistentes de `id`, `front` y `back`. v2 solo cambia la primera respuesta. Tu aplicación analiza el JSON; Anki Forge se encarga de crear el contenido y generar el APKG. Si los datos no son de confianza, valida el esquema de tu aplicación antes de crear las notas.

<!-- section: rust -->
## Rust

Rust 1.92 o posterior. El análisis de JSON usa `serde` y `serde_json`. Utiliza el paquete publicado de Anki Forge 0.3.1.

```sh
cargo new json-deck
cd json-deck
cargo add ankiforge@0.3.1
cargo add serde --features derive
cargo add serde_json
```

Guárdalo como `src/main.rs` y crea `recipe/` junto a `Cargo.toml`:

<!-- source: anki_forge/examples/json_to_anki.rs -->
```rust
use ankiforge::update::CompareOptions;
use ankiforge::{BuildOptions, Content, Media, Note, Project};
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize)]
struct Dataset {
    namespace: String,
    deck: String,
    notes: Vec<Row>,
}

#[derive(Deserialize)]
struct Row {
    id: String,
    front: String,
    back: String,
}

fn project(input: &Path, version: &str) -> Result<Project, Box<dyn std::error::Error>> {
    let data: Dataset = serde_json::from_slice(&fs::read(input.join(version))?)?;
    let image = Media::file(input.join("cell.png"))?;
    let audio = Media::file(input.join("tone.wav"))?;
    let mut project = Project::new(data.namespace)?.default_deck(data.deck);
    for row in data.notes {
        let front = Content::sequence([Content::text(row.front), image.image(), audio.sound()]);
        project.add(row.id, Note::basic(front, row.back))?;
    }
    Ok(project)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let input = Path::new(args.get(1).map(String::as_str).unwrap_or("recipe"));
    let output = Path::new(args.get(2).map(String::as_str).unwrap_or("output/rust"));
    fs::create_dir_all(output)?;
    let baseline = output.join("v1.apkg");
    let candidate = output.join("v2.apkg");
    if baseline.exists() || candidate.exists() {
        return Err("Choose a fresh output directory; keep original releases".into());
    }
    project(input, "v1.json")?.build(BuildOptions::to(&baseline))?;
    let next = project(input, "v2.json")?;
    let comparison = next.compare(CompareOptions::against(&baseline))?;
    fs::write(
        output.join("comparison.json"),
        serde_json::to_vec_pretty(&comparison.snapshot())?,
    )?;
    if !comparison.policy().allows_publication() {
        return Err("Comparison policy blocked publication; inspect comparison.json".into());
    }
    let prepared = next.prepare_publication(BuildOptions::to(&candidate).update_from(&baseline))?;
    let review = prepared
        .report()
        .comparison()
        .ok_or("Missing update comparison")?;
    fs::write(
        output.join("review.json"),
        serde_json::to_vec_pretty(&review.snapshot())?,
    )?;
    if !review.policy().allows_publication() {
        return Err("Prepared candidate blocked; inspect review.json".into());
    }
    let result = prepared.publish()?;
    println!("{}", result.artifact().path().display());
    Ok(())
}
```
<!-- /source -->

```sh
cargo run -- recipe output/rust
```

<!-- section: python -->
## Python

CPython estándar 3.11 o posterior con un wheel para tu plataforma; las comprobaciones de publicación cubren las versiones 3.11–3.14. En Windows, activa el entorno con `.venv\Scripts\Activate.ps1`. Utiliza el paquete publicado de Anki Forge 0.3.1.

```sh
mkdir json-deck
cd json-deck
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

Guárdalo como `main.py` y crea `recipe/` junto a `main.py`:

<!-- source: bindings/python/examples/json_to_anki.py -->
```python
"""Generate and review an answer-only update from caller-parsed JSON."""
import json
from pathlib import Path
import sys
from ankiforge import BuildOptions, CompareOptions, Content, Media, Note, Project


def project(source: Path, version: str) -> Project:
    data = json.loads((source / version).read_text(encoding='utf-8'))
    image = Media.file(source / 'cell.png')
    audio = Media.file(source / 'tone.wav')
    result = Project(data['namespace'], default_deck=data['deck'])
    for row in data['notes']:
        front = Content.sequence([row['front'], image.image(), audio.sound()])
        result.add(row['id'], Note.basic(front, row['back']))
    return result


source = Path(sys.argv[1] if len(sys.argv) > 1 else 'recipe')
output = Path(sys.argv[2] if len(sys.argv) > 2 else 'output/python')
output.mkdir(parents=True, exist_ok=True)
baseline, candidate = output / 'v1.apkg', output / 'v2.apkg'
if baseline.exists() or candidate.exists():
    raise RuntimeError('Choose a fresh output directory; keep original releases')
project(source, 'v1.json').build(BuildOptions.to(baseline))
next_project = project(source, 'v2.json')
comparison = next_project.compare(CompareOptions.against(baseline))
(output / 'comparison.json').write_text(json.dumps(comparison.snapshot(), indent=2), encoding='utf-8')
if not comparison.allows_publication:
    raise RuntimeError('Comparison policy blocked publication; inspect comparison.json')
prepared = next_project.prepare_publication(BuildOptions.to(candidate).update_from(baseline))
try:
    review = prepared.report.comparison
    if review is None:
        raise RuntimeError('Missing update comparison')
    (output / 'review.json').write_text(json.dumps(review.snapshot(), indent=2), encoding='utf-8')
    if not review.allows_publication:
        raise RuntimeError('Prepared candidate blocked; inspect review.json')
    result = prepared.publish()
    print(result.artifact.path)
finally:
    prepared.close()
```
<!-- /source -->

```sh
python main.py recipe output/python
```

<!-- section: node -->
## Node

Node.js 22.13 o posterior y la dependencia nativa opcional correspondiente. Este programa compila TypeScript a JavaScript y se ejecuta en Node.js. Utiliza el paquete publicado de Anki Forge 0.3.1.

```sh
mkdir json-deck
cd json-deck
npm init -y
npm install --include=optional ankiforge@0.3.1
npm install --save-dev typescript@6.0.3 @types/node@24.13.6
```

Guárdalo como `main.mts` y crea `recipe/` junto a `main.mts`:

<!-- source: bindings/node/examples/json_to_anki.mts -->
```typescript
import { readFile, mkdir, access, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { Project, Note, Content, Media, BuildOptions, CompareOptions } from 'ankiforge';

interface Dataset {
  namespace: string;
  deck: string;
  notes: { id: string; front: string; back: string }[];
}

async function project(source: string, version: string): Promise<Project> {
  const data: Dataset = JSON.parse(await readFile(path.join(source, version), 'utf8'));
  const image = await Media.file(path.join(source, 'cell.png'));
  const audio = await Media.file(path.join(source, 'tone.wav'));
  const result = new Project(data.namespace).defaultDeck(data.deck);
  for (const row of data.notes) {
    const front = Content.sequence([row.front, image.image(), audio.sound()]);
    result.add(row.id, Note.basic(front, row.back));
  }
  return result;
}

const source = process.argv[2] ?? 'recipe';
const output = process.argv[3] ?? 'output/node';
await mkdir(output, { recursive: true });
const baseline = path.join(output, 'v1.apkg');
const candidate = path.join(output, 'v2.apkg');
for (const file of [baseline, candidate]) {
  const exists = await access(file).then(() => true, error => {
    if (error.code === 'ENOENT') return false;
    throw error;
  });
  if (exists) throw new Error('Choose a fresh output directory; keep original releases');
}
const first = await (await project(source, 'v1.json')).build(BuildOptions.to(baseline));
await first.artifact.close();
const next = await project(source, 'v2.json');
const comparison = await next.compare(CompareOptions.against(baseline));
await writeFile(path.join(output, 'comparison.json'), JSON.stringify(comparison.snapshot(), null, 2));
if (!comparison.policy.allows_publication) {
  throw new Error('Comparison policy blocked publication; inspect comparison.json');
}
const prepared = await next.preparePublication(BuildOptions.to(candidate).updateFrom(baseline));
try {
  const review = prepared.report.comparison;
  if (!review) throw new Error('Missing update comparison');
  await writeFile(path.join(output, 'review.json'), JSON.stringify(review.snapshot(), null, 2));
  if (!review.policy.allows_publication) throw new Error('Prepared candidate blocked; inspect review.json');
  const result = await prepared.publish();
  console.log(result.artifact.path);
  await result.artifact.close();
} finally {
  await prepared.close();
}
```
<!-- /source -->

```sh
npx tsc main.mts --target ES2022 --module NodeNext --moduleResolution NodeNext --strict --outDir dist --types node
node dist/main.mjs recipe output/node
```

<!-- section: check-and-import-the-result -->
## Comprueba e importa el resultado

El programa elegido muestra la ruta `output/<language>/v2.apkg`. Conserva `v1.apkg` junto a ese archivo; `comparison.json` y `review.json` registran `RISK.NOTE_CHANGED`, ninguna nota añadida ni eliminada y `policy.allows_publication: true` para este cambio de respuesta. El programa vuelve a comprobar la política sobre el candidato preparado antes de publicarlo. Rechaza reutilizar un directorio de salida que ya contenga cualquiera de las dos versiones.

Importa `v1.apkg` en Anki: debes encontrar el mazo Biology con tres tarjetas Basic, la imagen de la célula y el tono. A continuación, importa `v2.apkg` y comprueba que la primera respuesta pasa a ser “The basic structural and functional unit of life”, manteniendo las tres notas. El espacio de nombres `biology-json` identifica este proyecto de publicación. Cada ID de negocio es una clave estable de nota; Anki Forge deriva de esa identidad el GUID que utiliza Anki. Conserva el espacio de nombres y esas tres claves sin cambios.

Usa el APKG v1 **original de distribución** como base de la actualización, nunca un paquete vuelto a exportar desde Anki. La coincidencia de notas al importar, las modificaciones locales y la programación de repasos dependen de la versión de Anki, los tipos de nota y las opciones de importación; valida el cliente de destino antes de publicar una versión. Importar un paquete no elimina las notas que se hayan omitido en un paquete posterior. Comparar o generar correctamente un paquete no garantiza el estado de repaso de cada estudiante.

Consulta las [condiciones de actualización](/docs/updates/), los [recursos multimedia](/docs/media/), la [guía de las API](/docs/api/) y la [comparación de bibliotecas](/docs/anki-forge-vs-genanki/).
