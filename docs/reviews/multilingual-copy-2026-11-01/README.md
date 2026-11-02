# Anki Forge 多语言文案审阅与修改记录

审阅时，6 种发布语言都有可用的内容基础，但自然度和术语质量并不一致。西班牙语整体较顺畅；中文技术正文、日语抽象名词和德语复合表达更需要编辑；巴西葡萄牙语需要重点统一模板与数据所有权术语。英文也有应先修正的过时标题，不能直接当作无误的翻译基准。

共整理 **143 条可替换建议**：**19 处 P2** 涉及事实范围或容易误解的表达，**124 处 P3** 涉及自然度、清晰度或术语一致性。同一原因在不同语言或位置分别列出，因此不是 143 个独立错误。仅凭成稿不能判断是否使用过机器翻译；以下结论针对现有文字本身。

**应用状态：已完成。** 已采用全部 143 条建议，并补齐同类术语在相关页面的其他用法，共修改 66 个产品源文件。正文中的问题、原文与位置保留为审阅时记录；最终措辞以当前源文件为准。

## 逐语言结论

| 语言 | 判断 | 优先处理 | 完整原文与替换建议 |
| --- | --- | --- | --- |
| 英语 | 整体准确，需清理源文遗留 | `from source` 与公开包支持范围冲突；`Application language` 指代不明 | [19 条建议](/Users/hp/Desktop/2026/anki-forge/docs/reviews/multilingual-copy-2026-11-01/en.md) |
| 简体中文 | 首页较自然，技术正文直译感重 | 源码限定、身份证据、拥有所有权的快照、消费者应用 | [30 条建议](/Users/hp/Desktop/2026/anki-forge/docs/reviews/multilingual-copy-2026-11-01/zh-CN.md) |
| 西班牙语 | 整体自然，局部歧义 | `trabajadores` 应指线程；`Audio único` 与搜索范围不清 | [20 条建议](/Users/hp/Desktop/2026/anki-forge/docs/reviews/multilingual-copy-2026-11-01/es.md) |
| 巴西葡萄牙语 | 整体可读，术语一致性不足 | `mídia exclusiva`、独立副本的含义、modelo 与 template 的区分 | [27 条建议](/Users/hp/Desktop/2026/anki-forge/docs/reviews/multilingual-copy-2026-11-01/pt-BR.md) |
| 日语 | 首页较好，正文名词生硬 | `任意の成果物`、`準備済み公開`、`Project のみ` 的范围 | [24 条建议](/Users/hp/Desktop/2026/anki-forge/docs/reviews/multilingual-copy-2026-11-01/ja.md) |
| 德语 | 语法基本顺畅，名词堆叠明显 | `Stapelimport` 与牌组术语冲突，性能标题和状态提示不自然 | [23 条建议](/Users/hp/Desktop/2026/anki-forge/docs/reviews/multilingual-copy-2026-11-01/de.md) |

## 优先修正的具体问题

1. **源码限定已经过时。** 英文 `Prepared publication from source`、`Source API details` 及中文对应标题仍让人以为要用源码版；相邻正文已说明公开 `0.3.1` 包包含这些功能。改为功能名称，同时保留既有锚点。
2. **葡语性能标签没有说清 unique。** `Imagens exclusivas`、`Áudios exclusivos` 容易让人想到独家资源。这里的比较是笔记之间是否共享媒体，建议 `Uma imagem diferente por nota`、`Um áudio diferente por nota`。
3. **日语 optional 误读风险。** `任意の成果物` 容易读成任意选择的产物，原意是返回值可能没有产物。建议 `成果物がない場合もある戻り値`。
4. **德语 batch 与 deck 混用同一词。** 文中 `Stapelimport` 其实是批量导入媒体，而 Stapel 已用于 Anki 牌组。建议 `Import mehrerer Mediendateien`。
5. **西语 worker 缺少技术限定。** `el funcionamiento de los trabajadores` 应改为 `el funcionamiento de los hilos de trabajo`。
6. **西语和葡语把检查上限本身写成错误。** 这里应是超过检查上限才出错，分别补上 `la superación de` 和 `a ultrapassagem dos`。

## 六种语言的代表性优化

| 语言 | 原文 | 优化文字 |
| --- | --- | --- |
| 英语 | Your coding assistant supports Skills? | Does your coding assistant support Skills? |
| 简体中文 | 无发现项 | 未检测到变化 |
| 西班牙语 | Dar un paquete de referencia a la siguiente generación | Usar el paquete anterior como referencia |
| 巴西葡萄牙语 | Imagens exclusivas | Uma imagem diferente por nota |
| 日语 | 準備済み公開 | 候補ファイルを準備し、確認後に保存する |
| 德语 | Der Export, gemessen. | Gemessene Exportzeiten |

## 统一术语时的判断

“笔记”“卡片”“牌组”应各自对应 note、card、deck。正文先说明笔记存储字段、模板据此生成卡片，再介绍稳定键，避免把所有对象都简称为卡片。

identity 在首页通常可以说明为“保持相同的笔记 ID”，在 API 文档中则可能包含命名空间、GUID、字段或模板映射，不宜一律缩成 ID。“身份证据”建议按内容写成“标识映射和修订信息”；涉及与实际包核验时，再说明这些信息如何作为更新依据。

ownership 是技术语义，需要说明哪个对象持有数据、何时释放临时文件、克隆是否共享存储。“拥有所有权的快照”“eigenem Datenbesitz”可以在首次定义后保留简写，但不宜直接作为面向新用户的卖点。不要把 owned 自动改成“独立副本”，也不要引入版权含义。

publish 在构建 API 中指把候选 APKG 写入目标路径，并不代表上传网站或发送给用户。建议在功能首次出现时说明这个动作，再保留 API 标识符 `publish` 和公开的类型名称。

葡语建议把 NoteType 写成 `tipo de nota`，把卡片模板写成 `template de cartão`；`modelo` 可在明确指 NoteType 模型的上下文中使用。对应界面和元数据的替换已一起列出。日语的 prepared publication 建议用“先准备候选文件，确认后保存”的动作说明，不强造一个对应英文的复合名词。

## 应保留的文字

- `machine.optional = "Optional"` 是机器阅读索引的固定标题，本仓库生成器明确禁止翻译，不能算漏译。
- `Basic`、`Cloze`、API 名、文件名、风险码和稳定键是标识符或共享示例，不应为了全本地化而改写。`hola → hello` 是西语到英语的卡片示例，应在所有界面语言下保留。
- 概念页中的 `正面` 是显示名称与稳定字段键分离的示例，不是西语、葡语或德语页面的意外漏译。
- 各语言博客标题已经做过不同程度的改写。英语双关不要求其他语言逐字复刻；已经自然的中文、西语和日语写法可以保留。
- 目前复数形式、中文量词和日语计数短语有明确实现及测试，不能仅因它们不与英语词序一致就认定有误。

## 审阅范围和验证

审阅日期为 2026 年 11 月 1 日，依据当前工作区文件，包含开始审阅前已经存在的未提交改动。发布范围取自 `website/release-locales.json`：en、zh-CN、es、pt-BR、ja、de。fr 和 ru 只有配置，本次不把它们列为已发布或已有译文。

- 逐项阅读 6 份 UI 目录，每份 210 个键，共 1,260 个键，包含复数分支；同时阅读 6 份搜索覆盖目录，每份 11 个键，共 66 个键。
- 核对页面注册表的标题、摘要与导航，以及 6 份 AI 任务模板。
- 29 篇文档和 2 篇博客各有 6 个语言版本，共 186 个源文件。所有文件核对导语和标题；完整阅读核心概念、更新、构建保证、图片遮挡、Python API 迁移、genanki 迁移及两篇博客的自然语言正文，共 48 份。其余长篇 API 和教程未声明已逐句审校。中文 CSV、AI 和 Rust 指南等另有针对性复查。
- 围栏代码按共享示例处理，未翻译标识符；本报告不声称完成 204 个发布页面的逐字终审或线上浏览器验收。覆盖深度逐文件记录在 [coverage.json](/Users/hp/Desktop/2026/anki-forge/docs/reviews/multilingual-copy-2026-11-01/coverage.json)。
- 已通过 `scripts/messages.test.mjs` 与 `scripts/locale-model.test.mjs` 的 7 项测试。另逐条核对建议原文存在、源文件指纹一致、替换文案非空，以及 UI 插值参数保持不变。测试通过只说明结构与格式有效，不代表语言自然度已自动验证。

全部候选文案已应用；保留了开始修改前已有的工作区改动，未更新 `content-reviews.json` 或 `post-build-reviews.config.json` 的审阅凭据。机器可读替换清单见 [suggestions.json](/Users/hp/Desktop/2026/anki-forge/docs/reviews/multilingual-copy-2026-11-01/suggestions.json)。其中每条记录包含源路径、行号、原文、替换文字、问题说明和源文件指纹。

修改已覆盖 P2/P3 建议及关联术语；标题和摘要在实际来源中更新，文档展示标题使用 `site-model.mjs`，博客使用 frontmatter。13 个变更标题的旧锚点已保留，并检查了全部 186 篇文档的原有章节映射。正式发布前仍需按仓库要求重新独立审阅修改后的内容，不能直接刷新既有审阅哈希来视为通过。


## 应用后的验证

- 全部 143 条建议逐条核对，优化文字均已出现在实际源文件中；同时补齐中文标识映射、葡语卡片模板、日语候选文件准备等术语的相关用法，并更新 `language-briefs.json`。
- 保留原有消息键、插值参数、协议标题 `Optional` 和全部可执行围栏代码。中文 README 中的 AI 任务模板已按源 JSON 同步。
- `npm test`：152 项通过。AI 模板与文档同步检查通过，覆盖 175 份源文档、103 个源码摘录。
- `CONTENT_REVIEW_MODE=preview` 内容检查与 `prepare:site` 通过；Rust 文档示例、卡片示例及更新示例均成功运行。隔离目录中的 Astro 构建生成 205 个 HTML 页面（含 404），未生成发布清单。
- 186 个 Markdown 页面与 6 个 AI 索引通过内容、代码和链接检查；204 个公开 HTML 页面通过结构化数据检查；6 个搜索目录成功生成。
- 在实际 HTML 中验证全部 186 篇文档的 687 个原有章节锚点和语义目标，没有重复 ID。13 个改名标题均保留旧锚点。
- 使用已安装的 Chrome 对隔离预览运行 19 项现有浏览器检查，全部通过：公开路由、六种语言的复制与失败恢复、AI 任务、搜索、章节导航、320/390/768/1440 像素布局、无 JavaScript 内容和 404 恢复。
- `git diff --check` 通过。正式发布的严格内容检查按预期提示 `home/en: sourceHash changed; independent source review required`。本轮没有更新或伪造审阅凭据；发布前需完成新的独立内容审阅。
