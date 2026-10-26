# 用 JSON 数据生成并更新 Anki 牌组

从 JSON 读取三道生物题，为它们加入同一张图片和一段音频，先生成原始 APKG，
再检查一条答案的修改并发布更新。每份牌组应有 **3 条笔记、3 张卡片、2 个媒体文件**。
下方三种语言任选一种，每份程序都能独立运行。生成 APKG 不需要安装 Anki。

Anki Forge 维护者撰写，2026-10-25 复核。

## 准备共同输入

下载 [v1.json](../recipes/json-to-anki/v1.json)、[v2.json](../recipes/json-to-anki/v2.json)、
[cell.png](../recipes/json-to-anki/cell.png) 和 [tone.wav](../recipes/json-to-anki/tone.wav)。
图片是一张细胞结构图，音频是短提示音，不是单词发音。
在每个新建的项目目录中，把这四个文件放进 `recipe/`：

```text
json-deck/
  recipe/
    v1.json
    v2.json
    cell.png
    tone.wav
```

两份 JSON 都包含 `namespace`、`deck` 和三条 `notes`；每条笔记有固定的业务 `id`、
问题 `front` 和答案 `back`。v2 只修改第一条答案。JSON 由调用方解析，
Anki Forge 负责笔记创作和 APKG 生成。接收不可信数据时，应先验证业务数据的格式。

<h2 id="rust">Rust</h2>

使用 Rust 1.92 或更新版本；JSON 由 `serde` 和 `serde_json` 解析。 安装公开版本 Anki Forge 0.3.1：

```sh
cargo new json-deck
cd json-deck
cargo add ankiforge@0.3.1
cargo add serde --features derive
cargo add serde_json
```

保存为 `src/main.rs`，并将 `recipe/` 放在项目根目录：

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

<h2 id="python">Python</h2>

使用普通 CPython 3.11 或更新版本及对应平台的 wheel；发布检查覆盖 3.11–3.14。Windows 下用 `.venv\Scripts\Activate.ps1` 激活环境。 安装公开版本 Anki Forge 0.3.1：

```sh
mkdir json-deck
cd json-deck
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

保存为 `main.py`，并将 `recipe/` 放在项目根目录：

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

<h2 id="node">Node.js / TypeScript</h2>

使用 Node.js 22.13 或更新版本，安装时保留对应平台的原生可选依赖。程序先编译为 JavaScript，再由 Node.js 执行。 安装公开版本 Anki Forge 0.3.1：

```sh
mkdir json-deck
cd json-deck
npm init -y
npm install --include=optional ankiforge@0.3.1
npm install --save-dev typescript@6.0.3 @types/node@24.13.6
```

保存为 `main.mts`，并将 `recipe/` 放在项目根目录：

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

## 检查结果并导入 Anki

程序会打印 `output/<language>/v2.apkg` 的路径。请同时保留 `v1.apkg`。
`comparison.json` 和 `review.json` 应包含 `RISK.NOTE_CHANGED`，没有新增或删除笔记，
本例的 `policy.allows_publication` 为 `true`。程序会再次检查待发布包的策略结果，
确认允许后才写出 v2。输出目录已有任一版本时，程序会停止，避免覆盖历史版本。

先把 `v1.apkg` 导入 Anki：Biology 牌组应有三张 Basic 卡片，可以显示细胞图并播放提示音。
再导入 `v2.apkg`，检查第一条答案变为 “The basic structural and functional unit of life”，
笔记总数仍为三条。namespace `biology-json` 标识这个发布项目，每个业务 ID 是一条笔记固定的 note key；
Anki Forge 据此生成 Anki 使用的 GUID。后续修改时，保持 namespace 和这三个键不变。

更新基线必须是上次**原始分发的 APKG**，不能用 Anki 重新导出的包代替。
能否匹配已有笔记、如何处理本地修改与复习进度，取决于 Anki 版本、笔记类型和导入设置；
发布前应在目标客户端验证。从后续包中省略一条笔记，并不会删除学习者已有的笔记。
比较和生成成功，也不代表所有客户端的复习状态都得到保证。

继续阅读[更新条件（英文）](../updates.md)、[媒体用法（英文）](../media.md)、
[API 参考（英文）](../api-overview.md)或[选型比较](anki-forge-vs-genanki.md)。
