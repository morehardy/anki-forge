# Anki-Stapel aus JSON erzeugen und aktualisieren

Canonical page: https://ankiforge.dev/de/docs/json-to-anki/
Language: de
Description: Lies JSON mit Rust, Python oder TypeScript ein, bündle Bilder und Audio und prüfe eine APKG-Aktualisierung, die nur eine Antwort ändert.

Lies drei Biologiefragen aus JSON ein, ergänze ein gemeinsames Bild und einen Audioclip und exportiere ein ursprüngliches APKG sowie eine geprüfte Aktualisierung, die nur eine Antwort ändert. Jede Version soll **3 Notizen, 3 Karten und 2 Mediendateien** enthalten. Wähle unten ein SDK; jedes Beispiel ist ein vollständiges Programm. Für den normalen Export muss Anki nicht installiert sein.

Gepflegt von den Anki-Forge-Maintainern · englische Vorlage geprüft am 2026-10-25.

## Gemeinsame Eingabedateien

Lade [v1.json](https://ankiforge.dev/generated/recipes/json-to-anki/v1.json), [v2.json](https://ankiforge.dev/generated/recipes/json-to-anki/v2.json), [cell.png](https://ankiforge.dev/generated/recipes/json-to-anki/cell.png) und [tone.wav](https://ankiforge.dev/generated/recipes/json-to-anki/tone.wav) herunter. Das Bild zeigt eine Zelle; die Audiodatei enthält einen kurzen erzeugten Ton, keine gesprochene Vokabel. Lege diese vier Dateien in jedem neuen Anwendungsverzeichnis unter `recipe/` ab:

```text
json-deck/
  recipe/
    v1.json
    v2.json
    cell.png
    tone.wav
```

Beide JSON-Dateien enthalten `namespace`, `deck` und drei `notes` mit dauerhaft verwendeten Werten für `id`, `front` und `back`. v2 ändert ausschließlich die erste Antwort. Die aufrufende Anwendung liest das JSON ein; Anki Forge übernimmt die Erstellung der Notizen und des APKG. Prüfe nicht vertrauenswürdige Eingaben gegen das Schema deiner Anwendung, bevor du daraus Notizen erstellst.

## Rust

Verwende Rust ab Version 1.92. Das JSON wird mit `serde` und `serde_json` eingelesen. Das Beispiel nutzt das veröffentlichte Paket Anki Forge 0.3.1.

```sh
cargo new json-deck
cd json-deck
cargo add ankiforge@0.3.1
cargo add serde --features derive
cargo add serde_json
```

Speichere den folgenden Code als `src/main.rs`. Lege `recipe/` neben `Cargo.toml` an:

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

```sh
cargo run -- recipe output/rust
```

## Python

Verwende reguläres CPython ab Version 3.11 mit einem Wheel für deine Plattform. Die Release-Prüfungen decken 3.11–3.14 ab. Unter Windows aktivierst du die virtuelle Umgebung mit `.venv\Scripts\Activate.ps1`. Das Beispiel nutzt das veröffentlichte Paket Anki Forge 0.3.1.

```sh
mkdir json-deck
cd json-deck
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

Speichere den folgenden Code als `main.py` und lege daneben das Verzeichnis `recipe/` an:

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

```sh
python main.py recipe output/python
```

## Node

Benötigt werden Node.js ab Version 22.13 und die passende native optionale Abhängigkeit. Dieses Programm kompiliert TypeScript zu JavaScript und führt es mit Node.js aus. Es nutzt das veröffentlichte Paket Anki Forge 0.3.1.

```sh
mkdir json-deck
cd json-deck
npm init -y
npm install --include=optional ankiforge@0.3.1
npm install --save-dev typescript@6.0.3 @types/node@24.13.6
```

Speichere den folgenden Code als `main.mts` und lege daneben das Verzeichnis `recipe/` an:

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

```sh
npx tsc main.mts --target ES2022 --module NodeNext --moduleResolution NodeNext --strict --outDir dist --types node
node dist/main.mjs recipe output/node
```

## Ergebnis prüfen und importieren

Das gewählte Programm gibt `output/<language>/v2.apkg` aus. Bewahre `v1.apkg` daneben auf. Für diese reine Antwortänderung dokumentieren `comparison.json` und `review.json` den Befund `RISK.NOTE_CHANGED`, keine hinzugefügten oder entfernten Notizen und `policy.allows_publication: true`. Vor der Veröffentlichung prüft das Programm die Richtlinie erneut am vorbereiteten Kandidaten. Ein Ausgabeverzeichnis, das bereits eine der beiden Versionen enthält, wird nicht wiederverwendet.

Importiere zunächst `v1.apkg` in Anki: Im Stapel Biology sollten drei Basic-Karten mit dem Zellbild und dem Ton erscheinen. Importiere danach `v2.apkg`. Prüfe, dass die erste Antwort nun „The basic structural and functional unit of life“ lautet und es weiterhin drei Notizen gibt. Der Namensraum `biology-json` kennzeichnet dieses Veröffentlichungsprojekt. Jede fachliche ID dient als stabiler Notizschlüssel; daraus leitet Anki Forge die von Anki verwendete GUID ab. Behalte den Namensraum und diese drei Schlüssel unverändert bei.

Verwende als Aktualisierungsbasis das **ursprünglich verteilte** v1-APKG, niemals einen erneuten Export aus Anki. Wie der Client Einträge beim Import zuordnet und mit lokalen Änderungen sowie der Lernplanung umgeht, hängt von der Anki-Version, den Notiztypen und den Importeinstellungen ab. Prüfe den vorgesehenen Client vor einer Veröffentlichung. Beim Import eines Pakets werden Notizen, die in einem späteren Paket fehlen, nicht gelöscht. Ein erfolgreicher Vergleich oder Build garantiert nicht den Erhalt jedes individuellen Lernstands.

Weitere Informationen findest du unter [Aktualisierungsbedingungen](https://ankiforge.dev/de/docs/updates/), [Medien](https://ankiforge.dev/de/docs/media/), [API-Übersicht](https://ankiforge.dev/de/docs/api/) und [Bibliotheksvergleich](https://ankiforge.dev/de/docs/anki-forge-vs-genanki/).
