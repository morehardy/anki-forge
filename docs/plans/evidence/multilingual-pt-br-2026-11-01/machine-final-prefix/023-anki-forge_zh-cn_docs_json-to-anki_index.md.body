# 用 JSON 数据生成并更新 Anki 牌组

Canonical page: https://morehardy.github.io/anki-forge/zh-cn/docs/json-to-anki/
Language: zh-CN
Description: 用 Rust、Node 或 Python 读取 JSON，打包图片和音频，再检查仅修改答案的更新。

从 JSON 读取三个生物学问题，添加共享图片与音频，导出原始 APKG 和经过检查的仅修改答案的更新。每个版本应包含 **3 条笔记、3 张卡片和 2 个媒体文件**。选择下方任一 SDK，每份都是完整程序。普通导出不需要安装 Anki。

由 Anki Forge 维护者维护 · 2026-10-25 复核。

## 共享输入

下载 [v1.json](https://morehardy.github.io/anki-forge/generated/recipes/json-to-anki/v1.json)、[v2.json](https://morehardy.github.io/anki-forge/generated/recipes/json-to-anki/v2.json)、[cell.png](https://morehardy.github.io/anki-forge/generated/recipes/json-to-anki/cell.png) 和 [tone.wav](https://morehardy.github.io/anki-forge/generated/recipes/json-to-anki/tone.wav)。图片是细胞示意图；音频是短合成音，并非词汇朗读。在每个新应用目录中，将这四个文件放入 `recipe/`：

```text
json-deck/
  recipe/
    v1.json
    v2.json
    cell.png
    tone.wav
```

两份 JSON 都包含 `namespace`、`deck` 和三条 `notes`，每条有持久 `id`、`front` 与 `back`。v2 只改变第一个答案。JSON 由调用方解析；Anki Forge 负责内容编写和 APKG 生成。对于不可信输入，创建笔记前应校验应用自己的 schema。

## Rust

Rust 1.92+。JSON 解析使用 `serde` 与 `serde_json`。使用公开 Anki Forge 0.3.1。

```sh
cargo new json-deck
cd json-deck
cargo add ankiforge@0.3.1
cargo add serde --features derive
cargo add serde_json
```

保存为 `src/main.rs`（在 `Cargo.toml` 同级创建 `recipe/`）：

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

普通 CPython 3.11+ 及匹配平台 wheel；发布检查覆盖 3.11–3.14。Windows 使用 `.venv\Scripts\Activate.ps1` 激活。使用公开 Anki Forge 0.3.1。

```sh
mkdir json-deck
cd json-deck
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

保存为 `main.py`（在 `main.py` 同级创建 `recipe/`）：

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

Node.js 22.13+ 及匹配原生可选依赖。程序将 TypeScript 编译为 JavaScript，再在 Node.js 上运行。使用公开 Anki Forge 0.3.1。

```sh
mkdir json-deck
cd json-deck
npm init -y
npm install --include=optional ankiforge@0.3.1
npm install --save-dev typescript@6.0.3 @types/node@24.13.6
```

保存为 `main.mts`（在 `main.mts` 同级创建 `recipe/`）：

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

## 检查并导入结果

所选程序输出 `output/<language>/v2.apkg`。将 `v1.apkg` 保留在旁边；`comparison.json` 和 `review.json` 记录 `RISK.NOTE_CHANGED`，没有新增或移除笔记，且对这次仅修改答案的编辑给出 `policy.allows_publication: true`。发布前，程序在预备候选包上再次检查策略。它拒绝复用已包含任一版本的输出目录。

将 `v1.apkg` 导入 Anki，应看到 Biology 牌组的三张 Basic 卡片、细胞图片与合成音。再导入 `v2.apkg`，检查第一个答案变成“The basic structural and functional unit of life”，且仍有三条笔记。命名空间 `biology-json` 标识该发布项目。每个业务 ID 是稳定笔记键；Anki Forge 根据该身份派生 Anki 使用的 GUID。保持命名空间和这三个键不变。

更新基线必须是**原始分发** v1 APKG，不能使用 Anki 重新导出的文件。客户端导入匹配、本地编辑和调度取决于 Anki 版本、笔记类型与导入设置；发布前验证目标客户端。导入包不会删除后续包中省略的笔记。比较或构建成功不保证每位学习者的复习状态。

参见[更新条件](https://morehardy.github.io/anki-forge/zh-cn/docs/updates/)、[媒体](https://morehardy.github.io/anki-forge/zh-cn/docs/media/)、[API 导航](https://morehardy.github.io/anki-forge/zh-cn/docs/api/)和[选库指南](https://morehardy.github.io/anki-forge/zh-cn/docs/anki-forge-vs-genanki/)。
