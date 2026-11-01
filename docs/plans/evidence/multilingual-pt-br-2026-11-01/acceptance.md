# 巴葡首批实现与验收

2026-11-01。范围遵循用户确认：巴葡先完成，日德仅准备术语与五类样稿。源码冻结提交为 `24da571c26da12fcbcbccb5007e090900c9bff71`，工作分支为 `codex/multilingual-common`。当前状态：**双配置工程验收及80个独立产物任务已通过，正在进行最终代码审查。未推送或部署新语言。**

## 实现范围

- `pt-BR` 已加入候选发布集合。新增29篇文档、2篇博客、210个业务UI键、11个框架键，以及首页／示例／博客列表、文档元数据、导航、提示模板、语言规范与检索数据。每语34个正式主题；四语产物为136个正式HTML、另1个404、124份Markdown、4份llms。
- 修复三个依赖历史状态的测试：planned语言使用私有fixture，不要求产品永久保留未发布语言；首次迁移验证使用两份原始不可变capture，而不读取会推进的生产基线。
- 实际探针发现窄屏200%文字下长语言名、移动导航弹层和短屏八语菜单不可达。修复工具栏换行、文档页行序、菜单水平边界和原生滚动；菜单保留原生details和链接，无JS仍可使用。新增真实浏览器回归。
- 巴葡旧schema负例最初未达门槛。仅补齐来源正文已经说明的project-schema检索概念及Rust指南主题词；查询、意图、目标页面和阈值不变。初次失败与中间诊断均保留。
- 日德各有术语brief、50键UI样稿和4篇完整正文样稿，独立复审已关闭5个minor。未加入正式发布集合。检索探针明确记录假名变体、德语复合词与停用词数据缺口，不宣称完整Pagefind或字体验收通过。

## 验收结果

两套最终产物的 `inputSha256` 均为 `035f635c8d91691475f8e8be98bd915bc3b02652046dbc3e154876f06317d99d`，releaseLocales 均为 en、zh-CN、es、pt-BR。

| 配置 | site / base / customDomain | 原始manifest SHA-256 | 完整交付文件 |
| --- | --- | --- | ---: |
| prefix | `https://morehardy.github.io` / `/anki-forge` / 空 | `05c8e5e987153fbf8da2b6a934cdaa8ae2e27e9fff14af5d7134b601222aff8e` | 490 |
| root | `https://ankiforge.dev` / `/` / `ankiforge.dev` | `e402cf153a6f60548799a7dfcd486fd32fd2b0b2b31d604d2fa18d6de7dcbf46` | 491，含CNAME |

- 完整单元测试147/147；严格类型检查123文件，0错误／警告／提示。源码文档验证、AI合同及技能quickstart检查通过；实际执行14个文档程序并独立检查23个APKG输出。后续布局和检索修复未修改这些程序或依赖。
- 双配置分别严格构建，并实际复建确认265个HTML／Markdown／llms文件逐字节一致。每套八项工程检查均通过：内容、站点、历史公开面、机器文本、结构化数据、门禁反例、35项浏览器检查和真实Pagefind。
- 每套搜索：en 12/12核心、25/25正例、5/5负例；zh-CN 12/12、24/25、5/5；es 12/12、25/25、5/5；pt-BR 12/12、23/25、5/5。原门槛为核心全过、正例至少23/25、负例全过。中文1条和巴葡2条正例缺口保留，不改题目掩盖。巴葡负例另逐条读取真实命中正文检查能力边界。
- 源审校：136条技术及表达记录全部绑定最终输入。前轮完整审阅保留，最终独立增量复审读取共享菜单／布局与巴葡词典实际差分；各表达维度至少4/5，保留35条既有minor观察，无未决major／critical。最终源报告为 `technical-source-review-final.json`、`linguistic-source-review-final.json`，并保留完整输入快照及原始报告。

## 独立产物任务与证据边界

每配置实际执行4语×6个使用任务及4语×4个机器读取任务，共48个浏览器任务和32个机器任务；浏览器共48次完整程序复制、32次输入下载。两配置均已通过 `release-evidence.mjs finish` 与 `check:release`。最终报告位于 `usage-tasks-{prefix,root}.json` 和 `machine-read-{prefix,root}.json`。原始观察使用 `*-final-prefix`／`*-final-root` 独立目录；最初失败候选的观察未改写成新inputSha。

机器任务共112个实际HTTP响应，分别从本语llms出发，读取本语Markdown，保留URL、状态、MIME、UTF-8、完整正文与哈希。root对比只替换精确的部署URL，未把GitHub源链接或任意路径泛化。浏览器使用任务保留页面导航、正文、真实剪贴板复制、下载和截图。

SDK消费程序执行证据是**显式复用**：独立代理重新核对23项源码／依赖输入、14份既有执行日志、10个真实APKG的ZIP／Zstandard／SQLite／媒体及6份比较报告。AI提示新增语言的文案差分单独核对；不宣称重新运行了这些SDK消费程序。它与上面的14个本轮文档程序实际执行是不同验证范围。

报告绑定全部128个必需机器资源及各任务明确读取的HTML／recipe资源；绑定范围不表示逐页人工阅读。工程收据另核验全部490／491个交付文件。生成APKG时间、showcase provenance和Pagefind入口JSON键序可能在重建时变化，不能把这些未在任务中语义读取的资源自动纳入可复用报告范围；实际网络观察仍保留。没有修改或放宽现有验证器。

工程日志及收据在 `engineering/final-{prefix,root}/`，原始manifest为 `final-{prefix,root}-origin-manifest.json`。受审产物分别保存在 `target/remaining-locales/pt-br/final-{prefix,root}-artifact`，均位于构建输入之外。证据提交本身只增加被排除的计划／审阅文件；后续CI仍须按其真实HEAD重新构建并执行工程门禁。

## 历史保护与发布边界

三语生产基线来自已独立下载并核验的 `main@ad98a6f`、Website workflow `41314923919`，包含真实线上资源与404检查。P0先推进三语公开面并保留旧机器合同，随后仅回补不可变bootstrap测试fixture。其独立分支 `codex/remaining-locales-baseline` 的完整验收见[三语基线记录](../remaining-locales-baseline-2026-11-01/acceptance.md)：147测试、双配置60个新真实任务及全部工程检查通过。

P0应先进入主线，确保巴葡PR的immutable base包含已发布西语的历史保护。巴葡当前是本地验收候选，尚无新CI部署或线上四语核验声明。巴葡实际发布后，再从独立verified artifact推进四语公开面；该维护仍须遵守原有验收合同。日德全文及法俄批次按既定顺序继续。

日德样稿与最初八语失败探针见[样稿目录](../remaining-locales-2026-11-01/samples/ja/sample-index.md)、[德语样稿](../remaining-locales-2026-11-01/samples/de/sample-index.md)及[P2探针](../remaining-locales-2026-11-01/samples/probes/README.md)。八语回归是在隔离浏览器DOM中展示全部已配置选项，不是八种语言已经上线。
