# Anki Forge 与 genanki

如果正在设计可重复的发布流程，需要拥有所有权的媒体、结构化诊断，以及发布更新前的比较与策略检查，可以考虑 Anki Forge。如果已有 genanki 的 Python 脚本、模板和媒体满足需求，迁移成本高于收益，则应保留经过验证的流程。

| 决策点 | Anki Forge | genanki |
| --- | --- | --- |
| 编写语言 | Rust 核心；Rust、Python 和 Node / TypeScript 接口 | Python |
| 共同基础能力 | Basic、Cloze、自定义模板、媒体与离线 APKG 生成 | Basic、Cloze、自定义模板、媒体与离线 APKG 生成 |
| 身份 | 显式命名空间和笔记键；更新需保留原始 APKG | 可配置 GUID 和数字模型/牌组 ID；支持重新导入更新 |
| 发布前 | 比较发现项、风险策略和已检查的预备候选包 | 由自己的脚本和测试定义发布检查 |
| 媒体所有权 | 类型化图片/声音携带拥有所有权的快照 | 脚本提供包媒体文件和引用 |
| 编程式图片遮挡 | 通过公开 API 使用带键矩形遮罩 | 按流程评估合适模型或扩展 |
| 运行成本 | 需要原生依赖；使用受支持平台和解释器 | 保留现有 Python 部署可能成本更低 |
| 迁移成本 | 显式验证旧身份映射、模板和导入 | 已正常工作的 genanki 流程无需迁移成本 |

## 同一个小任务

将三个 JSON 问题生成带图片和声音的牌组时，两种库都由 Python 解析 JSON 并编写笔记。genanki 需要选择模型和牌组 ID、设置字段值与 GUID 规则、将媒体文件加入包，再写出 APKG。Anki Forge 需要选择命名空间和持久业务键，将拥有所有权的媒体组合进笔记，构建 v1，再基于原始 v1 APKG 比较和预备 v2，最后发布。[完整 JSON 示例](../json-to-anki.md)提供 Rust、Python 和 TypeScript 实现及运行命令。

## 已有 genanki 牌组

[genanki 支持自定义 GUID](https://github.com/kerrickstaley/genanki#note-guids)、媒体和重新导入更新。不能将稳定身份视为 Anki Forge 独有能力。默认 GUID 规则可能依赖字段；字段会变化时，应有意选择 GUID 规则。

Anki Forge 不会自动恢复任意 genanki 数字 ID 或 GUID。没有单独验证的映射时，应将首次 Anki Forge 分发视为身份边界。此比较及成功 APKG 构建均不承诺无损迁移或保留全部复习历史。迁移已有牌组前阅读[迁移指南](../python/genanki-migration.md)。

## 历史性能证据

2026-10-08 的比较在 Apple M1 Pro 上测量 Rust Project API 与 genanki 0.13.1。每组 1,000 条 Basic 笔记时，五个实测 Rust 工作负载的导出耗时低 45.7–53.9%；部分媒体场景的峰值内存更高。[方法、原始结果与限制](https://github.com/morehardy/anki-forge/blob/bef4aeb653fc875f216614e73d8a617be039b9cf/benchmarks/results/20261008-latest-commit-genanki/report.md)。该实验未测量当前 Python 或 Node 宿主性能，也不保证这些 SDK 或其他工作负载获得相同加速。

Anki Forge 维护者 · 2026-10-25 根据公开 Anki Forge 0.3.1 API、[genanki 文档](https://github.com/kerrickstaley/genanki)和归档测量复核。开始新任务前，查看[运行环境兼容性](../compatibility.md)并[选择开发语言](https://ankiforge.dev/docs/languages/)。
