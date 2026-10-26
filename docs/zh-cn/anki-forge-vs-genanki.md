# Anki Forge 与 genanki：如何选择

准备新建或重新设计持续发布牌组的流程，希望在更新前检查变更、诊断问题并按风险策略决定是否发布，
可以考虑 Anki Forge。如果现有 genanki 脚本、模板和媒体流程稳定，新增需求也不明显，继续使用它通常更省事。

| 选择依据 | Anki Forge | genanki |
| --- | --- | --- |
| 开发语言 | Rust 核心，提供 Rust、Python、Node / TypeScript 接口 | Python |
| 共同能力 | Basic、Cloze、自定义模板、媒体、离线生成 APKG | Basic、Cloze、自定义模板、媒体、离线生成 APKG |
| 笔记身份 | 显式 namespace 和 note key；更新时保留原始 APKG | 可自定义 GUID，使用数字模型和牌组 ID；支持重新导入更新 |
| 发布前检查 | 比较变更、检查风险策略，检查待发布包后再发布 | 由自己的脚本和测试组织检查 |
| 媒体管理 | 图片和音频引用携带独立持有的媒体快照 | 脚本维护包中的媒体文件及引用 |
| 程序化图片遮挡 | 公开 API 支持带固定键的矩形遮挡 | 根据现有流程评估合适的模型或扩展 |
| 运行环境 | 需要原生依赖及受支持的平台、解释器 | 已有 Python 部署环境可能更适合保留 |
| 迁移成本 | 必须验证旧身份映射、模板和导入行为 | 已稳定运行的流程无需迁移 |

## 同一任务，两种流程

把三道 JSON 题目、一张图片和一段音频转成牌组时，两者都由 Python 解析 JSON。
genanki 流程需要确定模型和牌组 ID、填写字段、设置 GUID 规则、加入媒体文件，再写出 APKG。
Anki Forge 流程则确定 namespace 和固定业务键，把媒体加入笔记，生成 v1，
再基于原始 v1 APKG 比较变更、准备 v2，并在检查通过后发布。
[完整 JSON 教程](json-to-anki.md)提供 Rust、Python、TypeScript 三份程序和运行命令。

## 已有 genanki 牌组怎么处理

[genanki 可以自定义 GUID](https://github.com/kerrickstaley/genanki#note-guids)，也支持媒体和重新导入更新。
因此，不能把“有稳定身份”当成 Anki Forge 独有的优势。
genanki 的默认 GUID 规则可能依赖字段内容；字段会变化时，应有意识地选择 GUID 规则。

Anki Forge 不会自动恢复任意旧牌组的数字 ID 或 GUID。没有经过验证的身份映射时，
应把首份 Anki Forge 牌组视为新的身份起点。本文和成功生成的 APKG 都不代表无损迁移，
也不承诺保留全部复习记录。迁移前请阅读[迁移指南（英文）](../python/genanki-migration.md)。

## 怎样看待性能数据

2026-10-08 的历史比较测量了 Apple M1 Pro 上的 Rust Project API 与 genanki 0.13.1。
每组 1,000 条 Basic 笔记的五个工作负载中，Rust 导出耗时低 45.7–53.9%；
部分媒体场景使用了更多峰值内存。详见[测试方法、结果和限制](https://github.com/morehardy/anki-forge/blob/bef4aeb653fc875f216614e73d8a617be039b9cf/benchmarks/results/20261008-latest-commit-genanki/report.md)。
该测试没有测量当前 Python 或 Node 进程中的性能，不能据此承诺相同比例的提速，也不能推及其他工作负载。

Anki Forge 维护者于 2026-10-25 对照公开版本 0.3.1 API、
[genanki 官方说明](https://github.com/kerrickstaley/genanki)和历史测量复核。
开始新任务前，查看[运行环境（英文）](../compatibility.md)，再[选择开发语言](languages.md)。
