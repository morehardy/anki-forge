# Python API 对齐最终审查

固定基线：`51a44ad6ac74c0031d36f7edc436f74d504b7696`。
初次候选：`37f3593`；修复候选：`5f6ac7d`。范围为已接受方案的 T0–T9 / PY-01–PY-20。
两个独立审查者分别审查 Standards 与 Spec，修复后复查对应变更。
用户现有的根 README、Node 计划和 website 修改不属于本次审查。

## Standards

初审发现 1 项实际问题和 1 项启发式建议：

- **字段别名覆盖（P2）**：先写 `Front` 再修改 `front` 时，添加边界的字典映射可能保留旧值。现由统一 setter 规范 stock 别名；直接修改公开字典产生双别名时，在操作核心前明确拒绝。补充 text/html/typed content 和直接字典编辑的失败回归，修复后通过。
- **可能的重复代码（判断项）**：Project 快照与 IO builder 分别解码 Rust Note，stock key 观察不一致。统一为 `Note._from_native`，所有 stock 快照保持 Python 小写字段写法。

复查结论：未解决项 **0**。未发现额外的所有权、并发、Rust 公共边界等文档标准违规。

## Spec

初审发现 3 项 P2 问题及 1 项验收覆盖缺口：

- **有效核心 key 无法回读**：换行模板 key 和 Unicode 派生空 key 被 Python 的 ID 校验拒绝。Field/Template、identity、generation 和 Cloze 字段引用现保留原字符串，语义由核心校验；新增快照重用及完整 Rust APKG 对照。
- **字段显示名称被修改**：带前后空白或换行的名字在 setter 中被 trim/拒绝。现保留精确名称，覆盖 text/html/typed content、隐式 key、显式空白/空 key 和 Unicode。
- **历史样本来源证据**：原脚本固定 Python 源码却复用当前核心 executable。现从完整固定提交构建核心并使用同提交 contracts；真实重新生成的两个产物与原样本字节一致，并记录 SHA256。
- **模板包覆盖不足**：补齐 Cloze、坏 manifest、无效 UTF-8、三类大小上限、路径穿越、越界 symlink、重复类型；Cloze 同样比较独立 Rust 作者 API 产生的完整 APKG 观察。

复查结论：未解决项 **0**，未发现未要求的范围扩张。新增模板包/名称对等/迁移 30 项验证通过。

Standards 初审 2 项、最终 0 项；Spec 初审 4 项、最终 0 项。两轴原最高严重度均为 P2。
