# 日本語 术语与五类样稿

日期：2026-11-01。状态：**样稿独立审校完成；非完整 locale，未启用，未达到发布条件。** 本批仅按已确认的并行范围准备样稿，不增加 releaseLocales，也不写入正式本语文档目录或共享发布配置。

五类样稿为：首页与 UI、Rust quickstart、JSON recipe、更新页、Python API。后四项为完整正文译稿；首页与 UI 为选出的50个真实词典键，覆盖标题、摘要、CTA、复制状态、搜索加载／错误和404恢复。术语见 [language-brief.json](language-brief.json)，其中的语言检查项是待办，不是验收结果。

## 来源与摘要

| 样稿 | 权威英文来源或结构来源 | 初次写稿输入 SHA-256 |
| --- | --- | --- |
| [quickstart.md](quickstart.md) | `website/src/content/docs/docs/quickstart.mdx` | `274439a4424dd2deeadfbc38cb315ec4bd2163485691e9ff8be79b00901632e9` |
| [json-to-anki.md](json-to-anki.md) | `docs/json-to-anki.md` | `4e3dda0107dbe9de5806f0503eed57583961f884b0c519b076b17f257616a021` |
| [updates.md](updates.md) | `docs/updates.md` | `26b6bd608bb16df94b0f20466face0f0c44af1599674166b38881ee06d5a5e9f` |
| [python-api.md](python-api.md) | `docs/python/api.md` | `95e882472db174e247682c64a656a716ca5c289637b746d4e99d4c3fbaf87004` |
| [home-ui.json](home-ui.json) | `website/src/i18n/en.json` | `1d2d066c0ba73071e92caede3f8997f4864e1c33a40ce4a2ac6307dee9669848` |
| [language-brief.json](language-brief.json) | `website/language-briefs.json`（`ad98a6f` 的结构及既有三语规范；本语术语为新稿） | `29a40b5553535668741756670ae786ef57a1b496b8b7daa55e7db865f85de335` |

Quickstart 的英文 MDX 引用组件读取 `anki_forge/examples/target_api_basic.rs`；样稿已将该程序展开为完整 Markdown 代码块，源码摘要为 `c915295c5a0ff998a30b6a2744c6091b87b87914c02f30d0075bbfd227084866`。不保留依赖原目录位置的 MDX import。另三篇文档的 fenced code 直接取自英文原文，连注释和示例字符串也不翻译。

稳定章节依据 `website/section-contracts.json`（SHA-256 `9e31308999c8aeda94dc90d3ee50fa6a89a342a2fbad249994b5846f5e4a377b`），通过显式 section marker 保留任务身份，不复制其他语言的历史别名。站内主题链接用统一 `/docs/` 路径表达，正式集成时由现有链接转换器映射到本语；共同下载资源保持 `/generated/` 路径。这些文件位于排除构建输入的证据目录，不是可直接上线的网页。

## 已执行的静态检查

[authoring-checks.json](authoring-checks.json)保留修改前的作者自检、当时的输入与输出摘要，没有刷新为复审记录。Node 24 调用了现有 `sectionMapping`，验证4篇文档合计21个必需语义章节；19个代码块与对应英文代码／quickstart源程序逐字一致；50个UI键在英文词典中存在，插值参数一致；未发现生成占位符，JSON解析通过。

上述作者自检不构成独立内容审校。后续由未参与日德写作的代理完成单语盲读、英文技术对照、问题修订后的复审，并独立重跑代码块、章节和UI参数检查；最终输入与输出摘要见[独立样稿审校报告](../independent-review.json)。当前共享语言规范新增了巴葡，报告分别记录其当前摘要和 `ad98a6f` 的三语结构基准，核实既有三语条目没有变化。

这些记录均不包含代码执行、整站构建、浏览器布局、字体、Pagefind排名或外部搜索效果证明，不批准日语或德语正式发布。

## 转入完整语种时的待办

1. 五类样稿的独立技术与表达审校已完成。按已确认节奏，待巴葡完成后再扩展全文；后续修改与新正文仍须独立审校。
2. 补齐其余25篇文档、2篇博客、业务词典剩余160键、11个框架／搜索提示键，以及全部页面元数据、5个侧栏组和3个导航标签。
3. 补齐本语AI提示、placeholder与prompt合同、31个文档／文章的搜索词、生产概念词表及独立冻结的30题查询。样稿中的术语不是测得的搜索需求。
4. 在隔离fixture先做语种检索与布局探针；内容及pending审核索引完整后，再做真实Pagefind、双配置浏览器和机器文本验收。
5. 由集成人逐语组装完整候选，固定发布集合和证据路径；完成当前全部发布语言的源审核与构建后任务，通过既有门槛后才能发布。
