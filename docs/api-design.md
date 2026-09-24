# Rust API 设计入口

当前唯一实施方案是 [Rust API clean-slate design](plans/2026-09-23-rust-api-clean-slate-design.md)。原先的多阶段 Deck/Project/prelude 方案已删除，避免两份设计同时约束实现。

日常用法见 [Rust guide](rust-guide.md) 和 [API reference](rust-api.md)。关键边界如下：

- `Project` 是唯一创作容器。`Note` 拥有不可变 `NoteType` 和结构化 `Content`；`Media` 拥有读取时的快照。
- 常用类型由 crate 根导出；高级接口按 `note/schema/media/build/update/diagnostics` 组织。
- namespace、note key、field/template/mask key 与显示名称分开。字符串统一作为文本，HTML 显式声明。
- `BuildOutput` 代表成功且必有产物；报告仅记录观察，错误保留机器代码、原始原因和实际发布状态。
- 更新以原始分发 APKG 的完整包内身份为依据；比较保留全部风险，发布应用明确策略。
- 私有 lowering/normalization/writer 可以复用，普通消费者不能取得内部 IR。`internal-tools` 仅提供仓库实际使用的精选操作。

方案第 6 节记录消费者、分发和导入验收要求；第 8 节记录当前实施证据及尚未完成的检查。
