# 用 Python 将 CSV 批量生成 Anki 卡片

用 Python 和 Anki Forge 将词汇表转换为 Anki `.apkg` 文件。本示例读取包含
`id`、`front`、`back` 三列的 UTF-8 CSV，生成 Basic 问答卡片，并导出可分发的牌组。
Python 标准库负责解析 CSV，Anki Forge 负责生成包；导出时不需要安装 Anki。

如果只需要一次性导入表格，可以直接使用 Anki 的
[分隔文本导入功能](https://docs.ankiweb.net/manual/importing/text-files)。
需要可重复运行的脚本、自定义笔记 ID 或可分发的 APKG 时，再使用本教程。
Excel `.xlsx` 文件需要先另存为 UTF-8 CSV。

<!-- section: prepare-csv -->
## 准备 CSV

保存为 `cards.csv`，或[下载示例](/generated/recipes/csv-to-anki/cards.csv)：

<!-- source: docs/recipes/csv-to-anki/cards.csv -->
```csv
id,front,back
es:hola,hola,hello
es:gracias,gracias,thank you
es:adios,adiós,goodbye
```
<!-- /source -->

保留示例中的表头和列顺序，使用英文逗号分隔。字段包含逗号、双引号或换行时，
请按 [CSV 引号规则](https://docs.python.org/3/library/csv.html)处理。
脚本支持部分表格软件导出的 UTF-8 BOM，拒绝重复 ID、缺失值、多余列和空数据。

ID 一旦分配，就应在修改问题或答案时保持不变。不要用行号或会被修改的卡片文本
生成 ID。无关的数据集使用不同的命名空间；同一牌组的后续版本保持命名空间不变。

<!-- section: install -->
## 安装 Anki Forge

在[受支持的平台](compatibility.md)上使用普通 CPython 3.11 或更高版本。
创建并激活虚拟环境：

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

Windows 使用 `.venv\Scripts\Activate.ps1` 激活。
本次转换不需要额外的表格处理库或 AI 服务。

<!-- section: convert -->
## 生成 APKG

将以下程序保存为 `csv_to_anki.py`，与 `cards.csv` 放在同一个目录，
也可以[下载脚本](/generated/recipes/csv-to-anki/csv_to_anki.py)：

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

在该目录运行：

```sh
python csv_to_anki.py cards.csv vocabulary-v1.apkg --namespace spanish-vocabulary --deck Spanish
```

示例生成 `vocabulary-v1.apkg`，其中 **Spanish** 牌组包含 **3 条笔记、3 张 Basic
卡片**，不包含媒体。程序输出：

```text
vocabulary-v1.apkg: 3 notes, 3 Basic cards
```

请选择尚不存在的输出路径。脚本会拒绝覆盖已有文件，便于保留原始分发包。

<!-- section: verify-and-extend -->
## 导入、检查与扩展

在 Anki 中打开 `vocabulary-v1.apkg`。确认牌组包含 `hola → hello`、
`gracias → thank you`、`adiós → goodbye`，预览卡片并检查重音符号。
分发前请核对内容，尤其是由 AI 起草的 CSV。

本教程生成纯文本 Basic 卡片。其他需求可参考：

- [AI 辅助生成 Anki 卡片](ai-anki-cards.md)：根据原始笔记起草并审核 CSV。
- [JSON 转 Anki](json-to-anki.md)：使用 Python、Node / TypeScript 或 Rust，打包图片和音频，并检查第二个版本。
- [Basic 与 Cloze 卡片](cards.md)：选择问答卡片或填空卡片。
- [更新生成的 Anki 牌组](updates.md)：以原始 APKG 为更新基线并检查比较结果。

制作后续版本时，保留命名空间和笔记 ID，按更新流程检查后再分发。
稳定 ID 帮助识别原有笔记；Anki 导入设置与较新的本地编辑仍然会影响更新。
