#!/bin/zsh
# Independent machine-read reviewer: the exact order of plain-HTTP reads for one configuration.
# Usage: fetch-sequence.sh <outDir> <basePrefix>   (basePrefix is /anki-forge for prefix, empty for root)
# Each line is one agent decision: llms.txt first, then that locale's Markdown chosen from its links.
set -euo pipefail
D=$1; B=$2; F="$D/fetch.mjs"
cd /private/tmp
node $F $D "en entry point: read the English llms.txt first" $B/llms.txt
node $F $D "en python-install: llms Quickstarts link 'Generate Anki flashcards with Python'" $B/docs/python-quickstart/index.md
node $F $D "en python-install/compatibility-constraints: quickstart 'verified environments' and llms Start-here 'Compatibility and releases', read as Markdown" $B/docs/compatibility/index.md
node $F $D "en node-api: llms API references link 'Node and TypeScript API'" $B/docs/node-api/index.md
node $F $D "en node-api: llms Constraints link 'Build and output guarantees' for temporary artifacts and candidate publication" $B/docs/build-guarantees/index.md
node $F $D "en node-api: llms Quickstarts link 'Generate Anki flashcards with Node.js' for install and first build" $B/docs/node-quickstart/index.md
node $F $D "en json-update: llms Task guides link 'Generate and update Anki decks from JSON'" $B/docs/json-to-anki/index.md
node $F $D "en json-update: llms Task guides link 'Update Anki decks with stable note IDs' and the JSON guide's 'update conditions', read as Markdown" $B/docs/updates/index.md
node $F $D "en json-update: shared inputs linked from the JSON guide (v1.json, v2.json, cell.png, tone.wav)" $B/generated/recipes/json-to-anki/v1.json $B/generated/recipes/json-to-anki/v2.json $B/generated/recipes/json-to-anki/cell.png $B/generated/recipes/json-to-anki/tone.wav
node $F $D "en json-update: llms Task guides link 'Add images and audio to Anki cards' (JSON guide 'media' link) for media rules" $B/docs/media/index.md
node $F $D "zh-CN entry point: read the Chinese llms.txt first (no English fallback)" $B/zh-cn/llms.txt
node $F $D "zh-CN python-install: zh llms 快速开始 link '用 Python 生成 Anki 卡片'" $B/zh-cn/docs/python-quickstart/index.md
node $F $D "zh-CN python-install/compatibility-constraints: quickstart '已验证环境' and zh llms '兼容性与发布', read as Markdown" $B/zh-cn/docs/compatibility/index.md
node $F $D "zh-CN node-api: zh llms API 参考 link 'Node 与 TypeScript API'" $B/zh-cn/docs/node-api/index.md
node $F $D "zh-CN node-api: zh llms 限制与保障 link '构建与输出保证'" $B/zh-cn/docs/build-guarantees/index.md
node $F $D "zh-CN node-api: zh llms 快速开始 link '用 Node.js 生成 Anki 卡片' for install and first build" $B/zh-cn/docs/node-quickstart/index.md
node $F $D "zh-CN json-update: zh llms 任务指南 link '用 JSON 数据生成并更新 Anki 牌组'" $B/zh-cn/docs/json-to-anki/index.md
node $F $D "zh-CN json-update: zh llms 任务指南 link '用稳定笔记 ID 更新 Anki 牌组' and the JSON guide's '更新条件', read as Markdown" $B/zh-cn/docs/updates/index.md
node $F $D "zh-CN json-update: zh llms 任务指南 link '为 Anki 卡片添加图片与音频' (JSON guide '媒体' link)" $B/zh-cn/docs/media/index.md
node $F $D "zh-CN json-update: shared inputs linked from the zh JSON guide (v1.json, v2.json, cell.png, tone.wav)" $B/generated/recipes/json-to-anki/v1.json $B/generated/recipes/json-to-anki/v2.json $B/generated/recipes/json-to-anki/cell.png $B/generated/recipes/json-to-anki/tone.wav
node $F $D "en python-install: llms API references link 'Python API' (quickstart 'Python API'), read as Markdown to cross-check the quickstart calls" $B/docs/python-api/index.md
node $F $D "zh-CN python-install: zh llms API 参考 link 'Python API' (quickstart 'Python API'), read as Markdown to cross-check the quickstart calls" $B/zh-cn/docs/python-api/index.md
