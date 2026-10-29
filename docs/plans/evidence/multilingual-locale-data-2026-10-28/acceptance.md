# 语言数据化改造：本地验收

基于提交 `6780f26e23e88c702dcbfd88e74bc29de60d95df` 的工作区改动，每语 34 主题。**中英根路径与子路径均通过完整本地验收，包含真实重复构建和完整 `check:release`。尚未提交、未经 CI、未部署。**

本批把页面标题、描述、侧栏与导航文案改为按语言 ID 存放的数据，词典按发布语言加载，AI 提示的空请求占位和检查规则移入 `docs/ai-quickstart.json`，历史章节别名按发布它的语言存放。新增语言因此只需增加数据。全部 69 个 HTML、62 份 Markdown 和 2 个 llms 与上一已验收产物逐字节一致。

| 项目 | 结果 |
| --- | --- |
| 源审核 | 指纹代码变化使 68 项记录全部失效；独立技术与表达增量复审各自批准 68/68，无 major／critical；技术复审另作指纹代码审查 |
| 单元测试／Astro | 144 项通过；123 文件，0 错误、警告、提示 |
| SDK 执行 | 对公开 0.3.1 包重新执行：Rust、Python、Node 基础任务，重复键恢复，三 SDK 的 JSON＋媒体与仅改答案更新；全部 APKG 经检查器验证 |
| 实际重复构建 | 两个部署配置各 133 个 HTML、Markdown、llms 文件字节一致 |
| 发布门槛 | 两配置各八项工程检查及完整 `check:release` 通过 |
| 浏览器 | 每配置 16 项通过 |
| 搜索 | 两配置相同：英语核心 12/12、正例 25/25、负例 5/5；中文 12/12、24/25、5/5 |
| 独立构建后 AI | 新产物完成 24 次真实浏览器使用任务、16 次 llms→本语 Markdown 读取任务 |

中文变体“示意图 遮名称 复习题”仍未命中前三，满足预先固定的正例至少 23/25 门槛；23 条既有 minor 表达观察保留。源审核记录 3 条 minor 技术观察（其中 README 清单已补全），构建后复审仅有 minor／preference 文档观察。没有声称已验证 CI、线上收录、流量、排名、外部 AI 推荐、生产 MIME 或新的 Anki GUI 导入。

- [结构化摘要](final-acceptance.json) · [执行脚本](executions/run-basic-tasks.mjs) · [公开配方日志](executions/public-recipe.log)
- [根路径证明](engineering/root/release-evidence.json) · [真实重建](engineering/root/rebuild-results.json) · [使用任务](usage-tasks-root.json) · [AI 读取](machine-read-root.json)
- [子路径证明](engineering/prefix/release-evidence.json) · [真实重建](engineering/prefix/rebuild-results.json) · [使用任务](usage-tasks-prefix.json) · [AI 读取](machine-read-prefix.json)
- 源审核本轮文件位于 [multilingual-common-2026-10-27](../multilingual-common-2026-10-27/)：`source-review-locale-data-34.json`、`technical-locale-data-34-audit.json`、`linguistic-locale-data-review.json`

构建后证明保留原 manifest。之后重建时，只有源码输入和受审资源的实际字节都相同才允许复用；当前产物仍须独立通过工程检查。
