# Rust 编写指南

一个 `Project` 描述一次发布。用稳定命名空间创建项目，以稳定键添加笔记，再构建 APKG。常用类型直接从 `ankiforge` 导入；高级选项位于公开的领域模块。

## 项目与诊断

```rust
use ankiforge::{BuildOptions, Content, Note, Project};

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("jp-core")?
        .default_deck("Japanese::Core");
    project.add("taberu", Note::basic("食べる", Content::html("<b>to eat</b>")))?;
    let output = project.build(BuildOptions::to("jp-core.apkg"))?;
    println!("{} notes", output.report().counts().notes);
    println!("{}", serde_json::to_string_pretty(&output.snapshot())?);
    Ok(())
}
```

成功构建返回包含确定产物的 `BuildOutput`。`BuildReport` 仅含观察结果：数量、诊断、耗时和已完成的比较。它不包含操作结果，也不拥有产物。警告不会把成功输出变为失败。

模型完成时检查 schema 错误。`Project::add` 原子地检查笔记键、字段引用、必填字段、牌组、标签与依赖冲突。失败不改变项目。构建执行剩余的卡片、包和更新检查。没有独立可变注册表，也不要求额外的校验检查点。

错误提供固有的 `kind()` 和 `code()` 方法，并实现 `std::error::Error`。请使用机器码，不要解析显示消息。`BuildError::snapshot()` 包含真实失败、观察结果、来源链文本和发布事实。标准错误包装器可以将原始错误保留为来源。

## 模型与资源

完成的 `NoteType` 不可变且可共享。调用 `model.note()`，按稳定键赋值，再把结果加入项目。笔记持有模型；添加时自动收集模型和媒体。声明与模板规则见[自定义笔记类型](../custom-notetypes.md)。

`Media::file` 与 `Media::bytes` 获取拥有所有权的快照。导入成功后不再需要保留源文件。`image.image()` 和 `audio.sound()` 生成结构化内容；`Content::sequence` 将它们与文本或 HTML 组合。原始 HTML、CSS 或脚本资源必须通过 `builder.asset` 或 `project.add_asset` 显式包含。参见[媒体](../media.md)和[模板包](../template-bundles.md)。

## 输出所有权

持久输出选择 `BuildOptions::to(path)`，拥有所有权的临时文件选择 `BuildOptions::temporary()`。访问临时路径时保留输出对象或 `output.artifact()` 的克隆。最后一个所有者会删除文件。`persist_to(path)` 返回持久产物；失败时原所有者仍可用。JSON 快照包含路径，但不拥有文件。

## 稳定笔记身份

为 `project.add(key, note)` 使用持久的源记录键。项目命名空间、模型键、字段键和模板键独立于显示名称。内容变化不会创建新逻辑笔记。重复键会失败，而非覆盖。源记录重新排序或编辑时，保存并复用所选键。

## 更新已分发的牌组

向 `update_from` 提供上一次原始分发 APKG。完整的内嵌证据保留身份和修订映射。独立的 `project.compare(CompareOptions::against(path))` 分析候选包，并报告策略是否允许发布。高风险变更、客户端导入限制和显式放行见[双版本流程](../updates.md)。

从源码目录运行以下示例，它们执行相同的公开 API：

```sh
cargo run --locked -p ankiforge --example target_api_basic
cargo run --locked -p ankiforge --example target_api_custom_notetype
cargo run --locked -p ankiforge --example target_api_media
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

### 添加错误与媒体类别

`Project` 使用稳定命名空间和可选默认牌组，没有显示标题。`NoteType`、`Field` 与 `Template` 仍支持显示名称。`Project::add` 是原子操作。通过 `AddError::context()` 查看笔记或模型键，通过 `note::AddTarget` 查看原始位置；`detail()` 返回 `note::AddDetail`，说明冲突或不兼容媒体用法。匹配这些非穷尽枚举时，对数据变体使用 `..`，并提供兜底分支。字段的 `content_path` 为 `None` 时指字段本身，`Some([])` 指其根，序列下标则指嵌套节点。分隔符字节范围指向原始 UTF-8 Text/Html 叶节点。

`Media::image()` 和 `sound()` 保持不会失败。添加时，图片要求 `image/*`，声音引用要求 `audio/*` 或 `video/*`。其他类别产生 `NOTE.MEDIA_USAGE_INVALID`。这检查保留的 MIME，并非解码或播放。原始 HTML 和显式资源保持原有行为。导出重命名不会改变 MIME：PNG 改名为 `wrong.mp3` 后可以通过图片添加，但构建的独立探测或扩展名检查仍可能返回 `MEDIA.DECLARED_MIME_MISMATCH`。

工具输入现在要求 `ankiforge-project-v2`，且没有顶层 `name`；v1 被拒绝。契约包为 2.1.0；包身份格式仍为 `ankiforge-identity-v1`。
