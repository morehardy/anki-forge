# 任务化页面描述：本地验收

源码提交 `89e4ae7056938b2e357f0e93fef20889130e7f9f`，每语 34 主题。**中英根路径与子路径均通过完整本地验收，包含真实重复构建和完整 `check:release`。尚未经 CI、未部署。**

本批覆盖三项改动：功能页的任务化导语、描述与快速开始标题（`3cf5aed`）；中文 Python 快速开始描述改为“问答卡片”，恢复中文核心搜索（`89e4ae7`）；`check:examples` 的检查器依赖改用工作区锁定版本，修复 CI 离线解析失败（`05dbba1`）。

| 项目 | 结果 |
| --- | --- |
| 源审核 | 58 项因导语与侧栏标题变化失效，独立技术与表达复审全部批准；中文 Python 描述 1 项再经独立复审；无 major／critical |
| 单元测试／Astro | 144 项通过；123 文件，0 错误、警告、提示 |
| 文档示例 | 在仅含锁定依赖的干净 cargo 缓存上复现 CI 失败并修复；14 个完整 Rust 程序、23 个 APKG 通过检查 |
| SDK 执行 | 复用 2026-10-29 对公开 0.3.1 包的真实执行；全部绑定的源码与依赖字节未变 |
| 实际重复构建 | 两个部署配置各 133 个 HTML、Markdown、llms 文件字节一致 |
| 发布门槛 | 两配置各八项工程检查及完整 `check:release` 通过 |
| 浏览器 | 每配置 16 项通过 |
| 搜索 | 两配置相同：英语核心 12/12、正例 25/25、负例 5/5；中文 12/12、24/25、5/5 |
| 独立构建后 AI | 24 次真实浏览器使用任务、16 次 llms→本语 Markdown 读取任务 |

首轮子路径构建被搜索门槛正确拒绝：`3cf5aed` 的中文卡片页描述使“Python 问答 生成牌组”不再返回 Python 快速开始。该次证据保留在 [rejected-search-regression](rejected-search-regression/engineering/prefix/release-evidence.json)。机器重启清空了临时目录中的冻结产物，因此重新构建并冻结；复审者针对新产物重新完成全部浏览和读取，没有复用旧产物上的观察。

中文变体“示意图 遮名称 复习题”仍未命中前三，满足预先固定的正例至少 23/25 门槛。已记录的 minor 观察包括：英文 API 章节锚点被固定页眉遮住标题（既有样式 `scroll-padding-top: 28px`）、若干“from source”遗留标题、更新页开头的源码检出命令，以及媒体页缺少 API 链接。没有声称已验证 CI、线上收录、流量、排名、外部 AI 推荐、生产 MIME 或新的 Anki GUI 导入。

- [结构化摘要](final-acceptance.json) · [文档示例日志](engineering/doc-examples.log)
- [根路径证明](engineering/root/release-evidence.json) · [真实重建](engineering/root/rebuild-results.json) · [使用任务](usage-tasks-root.json) · [AI 读取](machine-read-root.json)
- [子路径证明](engineering/prefix/release-evidence.json) · [真实重建](engineering/prefix/rebuild-results.json) · [使用任务](usage-tasks-prefix.json) · [AI 读取](machine-read-prefix.json)
- 源审核本轮文件位于 [multilingual-common-2026-10-27](../multilingual-common-2026-10-27/)：`source-review-task-descriptions-34.json`、`source-review-python-qa-34.json` 及对应审校记录
