# Rust API

crate 根提供常用值：`Project`、`Note`、`NoteType`、`Field`、`Template`、`Content`、`Media`、`BuildOptions` 和 `BuildOutput`。公开领域模块为 `note`、`schema`、`media`、`build`、`update` 与 `diagnostics`。仓库工具是独立且由 feature 控制的接口；消费者应用不需要它们。

## 内容编写

| 入口 | 职责 |
| --- | --- |
| `Project::new(namespace)` | 校验稳定发布命名空间 |
| `project.default_deck(name)` | 设置默认目标牌组 |
| `project.add(key, note)` | 原子收集笔记、模型和媒体 |
| `project.add_asset(media)` | 包含显式原始 HTML/CSS/脚本资源 |
| `Note::basic(front, back)` / `Note::cloze(text)` | 创建持有内置模型的笔记 |
| `model.note().field(key, content)` | 创建持有不可变模型的自定义笔记 |
| `note.deck(name).tag(tag)` / `.tags(iterable)` | 添加前配置笔记 |
| `NoteType::builder(key)...build()` | 校验并完成不可变模型 |
| `NoteType::from_bundle(path)` | 从 bundle-v2 加载模型及其拥有所有权的资源 |

`Field::new(key).name(display)` 和 `Template::new(key).name(display)` 分离身份与展示。模板绑定字段键，包括过滤器、条件区块和浏览器模板。`Field::required()` 检查非空内容；`Field::sort()` 选择排序字段。`schema::GenerationRule::all` 和 `any` 接收字段键。`builder.cloze_field(key)` 声明 Cloze 模型。

`schema::FieldKey` 和 `TemplateKey` 是不同类型。可组合字符串字面量、拥有所有权或借用的字符串、`Cow<str>` 以及拥有所有权或借用的键。转换保留原值；schema 完成时校验键。模板键不会被隐式接受为字段键。

## 内容与媒体

普通字符串均转换为 `Content::text`。HTML 要求 `Content::html`。`Content::sequence` 组合语义值，直到构建时才渲染。`Media::file(path)` 与 `Media::bytes(bytes, mime)` 拥有快照；对应 `with_limits` 变体在读取前应用 `media::MediaLimits { max_bytes }` 预算，默认为每个资源 256 MiB。`with_export_name(name)` 返回共享同一快照的重命名值；已有克隆保留自己的名称。

`media.image()` 和 `media.sound()` 保留资源依赖。固定文件名必须可移植且无冲突。显式资源即使没有静态可见引用也会被包含。参见[媒体](../media.md)。

`Note::image_occlusion(media)` 返回 `note::ImageOcclusionBuilder`。`.mask(Mask::rect(key, x, y, width, height)).mode(mode).build()` 校验解码图像尺寸、矩形和唯一稳定键。支持 `HideAllGuessOne` 与 `HideOneGuessOne`，参见[图片遮挡](../image-occlusion.md)。

## 构建选项

`project.build(BuildOptions::to(path))` 发布持久 APKG。`BuildOptions::temporary()` 返回拥有所有权的临时输出。两者均接受 `.update_from(previous_apkg)`、`.inspect_limits(limits)` 和 `.update_policy(policy)`。没有基线却设置策略属于配置错误；setter 顺序不影响此规则。

`build::InspectLimits::default()` 提供有限预算，可修改其公开字段。值均为 `u64`；零就是零，并非无限制。

| 字段 | 默认值 |
| --- | --- |
| `max_archive_bytes` | 2 GiB |
| `max_entries` | 100,000 |
| `max_central_directory_bytes` | 16 MiB |
| `max_zip_entry_bytes` / `max_zip_total_bytes` | 1 GiB / 4 GiB |
| `max_meta_bytes` / `max_media_map_bytes` | 64 KiB / 16 MiB |
| `max_identity_bytes` | 64 MiB |
| `max_collection_bytes` / `max_media_bytes` | 512 MiB / 256 MiB |
| `max_decoded_total_bytes` / `max_zstd_window_bytes` | 4 GiB / 64 MiB |

限制独立应用于基线和候选包检查。失败检查报告资源、限制、观察量以及可用的相关条目。提高限制是调用方的显式操作；媒体导入有自己的更早预算。

## 构建、比较与输出

`BuildOutput::artifact()` 返回拥有所有权的 `build::ApkgArtifact` 引用。`BuildOutput::report()` 返回观察结果。产物克隆共享临时文件寿命；`persist_to` 返回持久所有者，或返回带发布事实与底层 I/O 来源的 `build::PersistError`。

`project.compare(update::CompareOptions::against(path))` 返回完成的 `ComparisonReport`，即使高风险发现项阻止其策略。它提供 `findings()`、`highest_risk()`、`policy()`、`diagnostics()` 和 `snapshot()`。`UpdatePolicy::default()` 阻止 High/Critical；`.fail_on(level)` 改变阈值，`.allow(RiskCode)` 接受整个已注册风险类别。它不能接受身份证据缺失或损坏等硬错误。

## 报告与错误

`BuildReport::snapshot()` 返回 `build::json::ReportSnapshot`，只有观察结果，没有成功/失败或文件所有权。`BuildOutput::snapshot()` 与 `BuildError::snapshot()` 返回 `build::json::BuildSnapshot`，包含实际结果和报告。`update::json::ComparisonSnapshot` 将原始发现项与策略评估分开。所有快照 DTO 实现 `serde::Serialize`，且不拥有文件。不需要扩展 trait。

Rust 中的快照路径字段保留原生 `PathBuf` 值。JSON 对有效 Unicode 使用字符串，否则使用无损原生编码：非 UTF-8 Unix 名称为 `{"encoding":"unix_bytes","bytes":[...]}`，含未配对 UTF-16 代理项的 Windows 名称为 `{"encoding":"windows_wide","units":[...]}`。成功产物路径和失败发布路径均适用。在原平台恢复 JSON 路径时，反序列化为 `build::json::PathSnapshot`，再调用 `into_path_buf()`；来自不同平台的编码路径会被拒绝。自行序列化错误中的原生路径时，`PathSnapshot::new(path)` 提供相同表示。

错误与所属操作放在一起：schema 和模板包错误在 `schema`，添加/IO 错误在 `note`，媒体错误在 `media`，构建/持久化错误在 `build`，比较/策略错误在 `update`。它们均提供 `kind()` 和 `code()`，实现 `std::error::Error + Send + Sync + 'static`，并保留真实来源错误。

## 添加上下文

添加失败不会改变项目。结构化上下文记录原始笔记或模型键和目标，并可附带冲突或媒体用法详情。Rust 通过 `AddError::context()` / `detail()` 提供 `note::{AddContext, AddTarget, AddDetail}`；SDK 通过 `AddError.details` 提供类型化 context/detail。Node 字段使用 camelCase 且深度冻结；Python 使用 snake_case TypedDict。标签保留插入下标；字段位置保留原始序列下标（null/None 指字段，[] 指其根）以及可选 UTF-8 字节范围。构建产生的默认牌组错误不带笔记或模型键，仍可从来源链读取。

添加时，类型化图片要求 image/* MIME，声音引用接受 audio/* 或 video/*。其他类别产生 `NOTE.MEDIA_USAGE_INVALID`。构造函数保持不会失败；显式资源和原始 HTML 不受影响。检查使用导入时保留的 MIME，不认证播放能力。构建的独立 MIME/扩展名校验仍然适用。完整 target/detail 表见[设计](../plans/2026-09-28-rust-api-validation-and-errors-design.md)。

## 审阅一次，发布一次

预备发布 API 已包含在公开 `0.3.1` 包中。


```rust,no_run
use ankiforge::{BuildOptions, Project};
# fn publish(project: &Project) -> Result<(), Box<dyn std::error::Error>> {
let prepared = project.prepare_publication(
    BuildOptions::to("next.apkg").update_from("previous.apkg"),
)?;
let report = prepared.report().clone(); // observations; owns no file
if report.comparison().is_some_and(|c| !c.policy().allows_publication()) {
    drop(prepared); // deletes the unpublished candidate; reprepare with a new policy
    return Ok(());
}
match prepared.publish() { // consumes the owner on every outcome
    Ok(output) => println!("Published {}", output.artifact().path().display()),
    Err(error) => {
        // Replacement may have succeeded even if durability confirmation failed.
        eprintln!("{:?}", error.publications());
        return Err(error.into());
    }
}
# Ok(())
# }
```

预备阶段执行完整的受限检查与比较。直到 `publish` 才发布目标；它使用同一私有候选包并复查基线别名。相对路径在预备调用时绑定。项目编辑不能改变候选包。报告耗时不含审阅等待。释放未发布所有者会清理它；已发布临时输出遵循通常的最后产物所有者清理规则。不能从 JSON 恢复所有者，也不能对已消耗所有者重试。原生构建省略未使用的暂存 manifest，但保留内嵌身份证据与媒体验证。内部暂存工具仍接收完整 manifest 和指纹。
