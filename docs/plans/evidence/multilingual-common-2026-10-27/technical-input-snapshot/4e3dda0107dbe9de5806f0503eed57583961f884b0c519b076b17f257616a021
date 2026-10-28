# Generate and update Anki decks from JSON

Read three biology questions from JSON, add a shared image and audio clip, and
export an original APKG plus a reviewed answer-only update. Each release should
contain **3 notes, 3 cards and 2 media files**. Choose one SDK below; each is a
complete program. Normal export needs no Anki installation.

Maintained by Anki Forge maintainers · reviewed 2026-10-25.

## Shared inputs

Download [v1.json](recipes/json-to-anki/v1.json), [v2.json](recipes/json-to-anki/v2.json),
[cell.png](recipes/json-to-anki/cell.png) and [tone.wav](recipes/json-to-anki/tone.wav).
The image is a cell diagram; the audio is a short generated tone, not spoken vocabulary.
In each new application directory, put these four files in `recipe/`:

```text
json-deck/
  recipe/
    v1.json
    v2.json
    cell.png
    tone.wav
```

Both JSON files contain `namespace`, `deck` and three `notes` with durable `id`,
`front` and `back` values. v2 changes only the first answer. The caller parses JSON;
Anki Forge handles authoring and APKG generation. For untrusted inputs, validate
your application's schema before constructing notes.

## Rust

Rust 1.92+. JSON parsing uses `serde` and `serde_json`. Use public Anki Forge 0.3.1.

```sh
cargo new json-deck
cd json-deck
cargo add ankiforge@0.3.1
cargo add serde --features derive
cargo add serde_json
```

Save as `src/main.rs` (create `recipe/` alongside `Cargo.toml`):

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

## Python

Ordinary CPython 3.11+ with a platform wheel; 3.11–3.14 is covered by release checks. On Windows, activate with `.venv\Scripts\Activate.ps1`. Use public Anki Forge 0.3.1.

```sh
mkdir json-deck
cd json-deck
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

Save as `main.py` (create `recipe/` alongside `main.py`):

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

## Node

Node.js 22.13+ and a matching native optional dependency. This program compiles TypeScript to JavaScript and runs on Node.js. Use public Anki Forge 0.3.1.

```sh
mkdir json-deck
cd json-deck
npm init -y
npm install --include=optional ankiforge@0.3.1
npm install --save-dev typescript@6.0.3 @types/node@24.13.6
```

Save as `main.mts` (create `recipe/` alongside `main.mts`):

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

## Check and import the result

The selected program prints `output/<language>/v2.apkg`. Keep `v1.apkg` alongside
it; `comparison.json` and `review.json` record `RISK.NOTE_CHANGED`, no added or
removed notes, and `policy.allows_publication: true` for this answer-only edit.
The program checks the policy again on the prepared candidate before publishing.
It refuses to reuse an output directory containing either release.

Import `v1.apkg` into Anki: expect the Biology deck with three Basic cards, the
cell image and the tone. Then import `v2.apkg`: check that the first answer becomes
“The basic structural and functional unit of life”, with three notes remaining.
The namespace `biology-json` identifies this publishing project. Each business ID
is a stable note key; Anki Forge derives the GUID used by Anki from that identity.
Keep the namespace and those three keys unchanged.

Use the **original distribution** v1 APKG as the update baseline, never an Anki
re-export. Client import matching, local edits and scheduling depend on Anki
version, note types and import settings; validate your target client before a
release. Importing a package does not remove notes omitted from a later package.
A successful compare/build is not a guarantee about every learner's review state.

See [update conditions](updates.md), [media](media.md), [API navigation](api-overview.md)
and [library selection](anki-forge-vs-genanki.md).
