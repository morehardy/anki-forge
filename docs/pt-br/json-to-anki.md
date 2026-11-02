# Gere e atualize baralhos do Anki a partir de JSON

Leia três perguntas de biologia em JSON, adicione uma imagem e um áudio compartilhados e exporte um APKG original seguido de uma atualização revisada que altera apenas uma resposta. Cada versão deve conter **3 notas, 3 cartões e 2 arquivos de mídia**. Escolha um dos SDKs abaixo; cada exemplo é um programa completo. A exportação normal não exige que o Anki esteja instalado.

Mantido pelos mantenedores do Anki Forge · conteúdo revisado em 2026-10-25.

<!-- section: shared-inputs -->
## Arquivos de entrada compartilhados

Baixe [v1.json](/generated/recipes/json-to-anki/v1.json), [v2.json](/generated/recipes/json-to-anki/v2.json), [cell.png](/generated/recipes/json-to-anki/cell.png) e [tone.wav](/generated/recipes/json-to-anki/tone.wav). A imagem é um diagrama de célula; o áudio é um tom curto gerado, não uma gravação de vocabulário falado. Em cada novo diretório de aplicação, coloque os quatro arquivos em `recipe/`:

```text
json-deck/
  recipe/
    v1.json
    v2.json
    cell.png
    tone.wav
```

Os dois arquivos JSON contêm `namespace`, `deck` e três `notes`, cada uma com um `id` durável e os campos `front` e `back`. A versão 2 altera somente a primeira resposta. A aplicação lê o JSON; o Anki Forge cuida da criação das notas e da geração do APKG. Para entradas não confiáveis, valide o esquema da sua aplicação antes de construir as notas.

<!-- section: rust -->
## Rust

Use Rust 1.92 ou posterior. A leitura de JSON usa `serde` e `serde_json`. Instale o pacote público Anki Forge 0.3.1.

```sh
cargo new json-deck
cd json-deck
cargo add ankiforge@0.3.1
cargo add serde --features derive
cargo add serde_json
```

Salve como `src/main.rs` e crie `recipe/` no mesmo diretório de `Cargo.toml`:

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

Use CPython convencional 3.11 ou posterior com um wheel para a plataforma; as verificações de lançamento abrangem as versões 3.11–3.14. No Windows, ative o ambiente com `.venv\Scripts\Activate.ps1`. Instale o pacote público Anki Forge 0.3.1.

```sh
mkdir json-deck
cd json-deck
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

Salve como `main.py` e crie `recipe/` no mesmo diretório de `main.py`:

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

Use Node.js 22.13 ou posterior e a dependência nativa opcional correspondente. Este programa compila TypeScript para JavaScript e executa o resultado no Node.js. Instale o pacote público Anki Forge 0.3.1.

```sh
mkdir json-deck
cd json-deck
npm init -y
npm install --include=optional ankiforge@0.3.1
npm install --save-dev typescript@6.0.3 @types/node@24.13.6
```

Salve como `main.mts` e crie `recipe/` no mesmo diretório de `main.mts`:

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
## Confira e importe o resultado

O programa escolhido imprime `output/<language>/v2.apkg`. Guarde `v1.apkg` no mesmo diretório; `comparison.json` e `review.json` registram `RISK.NOTE_CHANGED`, nenhuma nota adicionada ou removida e `policy.allows_publication: true` para essa alteração restrita à resposta. Antes de publicar, o programa verifica novamente a política no candidato preparado. Ele se recusa a reutilizar um diretório de saída que já contenha qualquer uma das versões.

Importe `v1.apkg` no Anki: espere o baralho Biology com três cartões Basic, a imagem da célula e o tom. Depois importe `v2.apkg`: confira se a primeira resposta mudou para “The basic structural and functional unit of life” e se continuam existindo três notas. O namespace `biology-json` identifica este projeto de publicação. Cada ID de negócio é uma chave estável da nota; o Anki Forge deriva dessa identidade o GUID usado pelo Anki. Mantenha o namespace e essas três chaves inalterados.

Use o APKG da versão 1 **originalmente distribuído** como base de atualização, nunca um pacote reexportado pelo Anki. A correspondência durante a importação, as edições locais e o agendamento dependem da versão do Anki, dos tipos de nota e das opções de importação; valide o cliente de destino antes de publicar. Importar um pacote não remove notas omitidas de um pacote posterior. Uma comparação ou geração bem-sucedida não garante o comportamento de todo o histórico de revisão de cada pessoa.

Consulte as [condições de atualização](/docs/updates/), [mídia](/docs/media/), a [navegação das APIs](/docs/api/) e a [escolha de biblioteca](/docs/anki-forge-vs-genanki/).
