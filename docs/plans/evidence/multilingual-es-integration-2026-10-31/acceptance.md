# 西班牙语与最新 main 的集成验收

2026-10-31。源码提交 `c828b2d895c8d6cda143862bcd2daaff3a8a279a`，最终产物构建于证据提交 `a9f3facef528369976712b81ead4c99c162b2907`（源码输入相同），基于 `main` 的 `e5d3d66` 合并西语 `991187c`。**根路径及 `/anki-forge/` 均通过完整本地验收和最终 `check:release`；生产部署尚未执行。** 本记录只证明下面列出的本地结果，GitHub CI 结果以关联 PR 为准。

## 集成结果

- 解决 10 个冲突文件，保留 main 的 locale maps、动态词典加载、AI 合同校验、中文搜索修复及文档示例锁定依赖修复。
- 将西语标题、摘要、导航和侧栏迁入现有语言映射，删除候选的 `locale-metadata.json`，不恢复二元语言分支。
- 将西语提示要求放入 `docs/ai-quickstart.json.promptContracts.es`，保留缺失合同的回归检查；额外语言测试继续验证只添加数据即可发布下一种语言。
- 同步五篇西语功能文档的最新任务说明、外部媒体生成边界、坐标输入限制和 Python/Node API 链接，并修正西语迁移页的品牌拼写。
- 为当前 102 个页面重新取得独立技术和语言审核，旧审核记录未通过修改哈希冒充复审。

## 实际验证

| 项目 | 结果 |
| --- | --- |
| 单元测试／Astro | 144 项通过；123 文件，零错误、警告、提示 |
| 内容与代码 | 34 主题 × 三语，源内容、稳定章节、代码与版本校验通过 |
| 文档程序 | 14 个完整 Rust 程序、23 个 APKG 输出重新执行并检查 |
| 公开包 | 公开 0.3.1 Rust、Python、Node 基础及 JSON/媒体/更新程序重新执行；重复键恢复通过 |
| 正式产物 | 每配置 102 正常 HTML + 404、93 Markdown、3 llms |
| 稳定构建 | 两配置各真实重复构建；199 个 HTML/Markdown/llms 文件逐字节一致 |
| 工程门槛 | 两配置各八项工程检查、23 项浏览器测试及最终发布检查通过 |
| 搜索 | 两配置相同：en 核心12/12、正例25/25、负例5/5；zh-CN 12/12、24/25、5/5；es 12/12、25/25、5/5 |
| 独立使用验收 | 三语 × 六任务 × 两配置，共36次真实浏览器任务；复制、下载与10个实际输出APKG的独立解包检查通过 |
| 独立机器读取 | 三语 × 四任务 × 两配置，共24次从本语llms到本语Markdown的实际HTTP读取及回答 |

源语言审核保留32项既有中英minor观察，并记录2项西语非阻塞编辑观察；机器读取保留既有中英文“source-only”小节标题观察。没有未关闭的critical/major。中文一条扩展查询仍未进前三，满足预先固定的至少23/25门槛；未修改题集来提高结果。没有声称新增Anki GUI导入验证、线上MIME、搜索引擎收录或真实用户采用。

## CI 环境差异与复核

首次 PR CI 在最终发布检查中拒绝了本地审核的 HTML：本地跨工作区 `node_modules` 符号链接改变了 Astro 的组件路径、样式标识和 CSS 文件名。保持源码及依赖字节不变，使用当前工作区的普通依赖目录后，出错页面 `/anki-forge/docs/` 的 SHA-256 与 CI 精确一致。两种产物均已重建并通过全部工程门槛，独立审核者重新执行36个浏览器任务并检查新截图。原 HTTP 读取报告不冒充重跑：独立复核确认两种产物的96个 Markdown／llms 资源均逐字节未变，现有严格校验器允许复用。旧产物 manifest 和原报告保留，发布校验规则未放宽。

构建验收时应在该工作区运行 `npm ci`，不要将其他工作区的 `node_modules` 整体符号链接进来。这里的回归证据是同一输入的差分构建、与 CI HTML 哈希精确比较及最终两配置发布门槛；未增加不能覆盖该环境差异的浅层单元测试。

## 可复核证据

- [结构化摘要](final-acceptance.json)
- [环境差分诊断](build-environment-diagnosis.json) · [机器资源复用独立审计](machine-artifact-reuse-audit.json)
- [根路径新产物](root-ci-origin-manifest.json) · [子路径新产物](prefix-ci-origin-manifest.json)
- [独立技术源审核](technical-source-review.json) · [独立语言源审核](linguistic-source-review.json)
- [根路径发布证明](engineering/root/release-evidence.json) · [子路径发布证明](engineering/prefix/release-evidence.json)
- [根路径使用任务](usage-tasks-root.json) · [子路径使用任务](usage-tasks-prefix.json)
- [根路径机器读取](machine-read-root.json) · [子路径机器读取](machine-read-prefix.json)
- [公开包执行](executions/) · [使用任务独立输出检查](usage-ci-execution-audit.json)
