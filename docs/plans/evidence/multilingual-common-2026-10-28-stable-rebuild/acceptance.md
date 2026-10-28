# 中英共性改造：稳定重建后的最终本地验收

源码提交 `fff473910afea60b6d653ca525cd4dc6ef07c289`，每语 34 主题。**中英根路径与子路径均通过完整本地验收，包含真实重复构建。尚未部署。** 本批替代上一批的最终验收状态；旧原始观察保留。

原发布门槛正确拒绝了提交后新建的 HTML：旧主题控件在构建时生成随机 UUID。差分确认全部 69 个 HTML 的差异仅为这一名称。修复使用 `header`／`docs` 两个稳定调用域，避免文档页两套控件互相取消选中；浏览器同时验证独立组选中和同步切换。

| 项目 | 结果 |
| --- | --- |
| 源审核 | 68 项技术与表达审核，稳定控件增量经过独立复审；无未关闭 major／critical |
| 单元测试／Astro | 141 项通过；122 文件，0 错误、警告、提示 |
| 实际重复构建 | 两个部署配置分别重新构建，全部 133 个 HTML（含 404）、Markdown、llms 文件字节一致；不归一化差异 |
| 发布门槛 | 两配置各八项工程检查及完整 `check:release` 通过；旧公开面与 60 个既有机器资源保留 |
| 浏览器 | 每配置 16 项通过，含双主题控件、无 JS、窄屏、复制及搜索故障恢复 |
| 搜索 | 两配置相同：英语核心 12/12、正例 25/25、负例 5/5；中文 12/12、24/25、5/5 |
| 独立构建后 AI | 新产物完成 24 次真实使用任务、16 次 llms→本语 Markdown 读取任务；SDK 执行只在摘要一致时明确复用 |

中文变体“示意图 遮名称 复习题”仍未命中前三，满足预先固定的正例至少 23/25 门槛；23 条既有 minor 表达观察保留。没有声称已验证线上收录、流量、排名、外部 AI 推荐、生产 MIME 或新的 Anki GUI 导入。

- [结构化摘要](final-acceptance.json)
- [根路径证明](engineering/root/release-evidence.json) · [真实重建](engineering/root/rebuild-results.json) · [使用任务](usage-tasks-root.json) · [AI 读取](machine-read-root.json)
- [子路径证明](engineering/prefix/release-evidence.json) · [真实重建](engineering/prefix/rebuild-results.json) · [使用任务](usage-tasks-prefix.json) · [AI 读取](machine-read-prefix.json)

同源证明必须保留原 manifest。提交报告后重建时，只有源码输入和受审资源的实际字节都相同才允许复用；当前产物仍须独立通过工程检查。这一规则由完整发布门槛执行，缺证或摘要变化即拒绝。
