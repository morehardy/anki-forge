# 全新的 Python API

使用 `python -m pip install ankiforge` 和 `from ankiforge import ...`。旧 `anki-forge` 发行名和 `anki_forge` 导入名不是兼容别名。源码目录在修改名称后需用 Maturin 重建原生扩展。参见[安装指南](../../bindings/python/README.md)。

不保留旧兼容层。将旧 Deck、注册表和 runtime API 替换为拥有值所有权的公开模型：

| 旧流程 | 当前流程 |
| --- | --- |
| 可选稳定 ID 的 Deck 或 Project | `Project(namespace, default_deck=...)` |
| `add_note(note)` 与基于内容的身份 | `add(stable_key, note)` |
| 可变 NoteType 与 add_notetype | `NoteType.builder(key)...build()`，再调用 `model.note()` |
| 字符串模型引用 | 笔记持有已校验模型 |
| 项目媒体注册表 | 独立 `Media.file` / `Media.bytes` 快照 |
| 内联渲染媒体和字段专用 setter | `Content.sequence`、`media.image()`、`media.sound()`、`Note.field` |
| 隐式 HTML | 字符串是 Text；使用 `Content.html` |
| 没有稳定身份的遮罩 | `Mask.rect(key, x, y, width, height)` |
| 可选产物与 ensure_success | `BuildOutput` 保证包含产物；失败抛结构化错误 |
| 根据诊断猜测报告结果 | Output/Error 快照提供实际结果；Report 只有观察结果 |
| Diff、lockfile 与更新安全开关 | `compare(CompareOptions.against(...))`、`BuildOptions.update_from(...)` |
| Runtime 子进程包 | 进程内 PyO3 绑定，调用默认 Rust API |

只有携带当前身份证据的完整原始分发 APKG 才能作为更新基线。不会自动恢复旧模板包和旧 APKG 证据。新流程见 README 与可执行示例。
