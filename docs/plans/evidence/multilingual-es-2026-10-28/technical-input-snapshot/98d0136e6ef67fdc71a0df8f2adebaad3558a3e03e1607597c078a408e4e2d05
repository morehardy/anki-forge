# 图片与音频

`Media` 值拥有不可变快照。使用 `Media::file` 导入文件，或通过带显式 MIME 类型的 `Media::bytes` 导入原生字节。成功导入后，修改或删除源文件都不会改变该资源。克隆共享存储，可跨项目使用。

## 运行媒体流程

在仓库根目录运行：

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
cargo run --locked -p ankiforge --example target_api_media
```

该流程使用仓库 PNG/WAV 测试素材，写出包含一条笔记、一张卡片和三个资源的 `media.apkg`。第二个示例自行生成小图像和音频，包含 CSS 引用及显式声明的媒体，并写出 `spanish-media.apkg`。

<!-- source: anki_forge/examples/docs_workflow.rs#media -->
```rust
fn media(output: &Path, image: &Path, sound: &Path) -> anyhow::Result<()> {
    let image = Media::file(image)?.with_export_name("cell.png")?;
    let sound = Media::file(sound)?;
    let mut project = Project::new("docs-media")?;
    project.add("cell", Note::basic(image.image(), sound.sound()))?;
    let css = Media::bytes(b".card { color: navy; }".to_vec(), "text/css")?
        .with_export_name("course.css")?;
    project.add_asset(css)?;
    let built = project.build(BuildOptions::to(output.join("media.apkg")))?;
    verify(&built, 1, 1, 3)?;
    Ok(())
}
```
<!-- /source -->

## 组合内容与显式资源

这份完整程序使用 `fixtures/pixel.png`、`fixtures/silence.wav` 和 `fixtures/labels.woff`。在应用中请传入自己的源路径。测试字体是仓库中允许再分发的测试用字体。

```rust
use ankiforge::{BuildOptions, Content, Field, Media, NoteType, Project, Template};

fn main() -> anyhow::Result<()> {
    let image = Media::file("fixtures/pixel.png")?.with_export_name("badge.png")?;
    let sound = Media::file("fixtures/silence.wav")?;
    let font = Media::file("fixtures/labels.woff")?.with_export_name("labels.woff")?;
    let model = NoteType::builder("media-card")
        .field(Field::new("front"))
        .field(Field::new("back"))
        .template(Template::new("card")
            .front(r#"{{front}}<img src="badge.png">"#)
            .back("{{FrontSide}}<hr>{{back}}"))
        .css("@font-face { font-family: labels; src: url('labels.woff'); } .card { font-family: labels; }")
        .asset(font)
        .asset(image.clone())
        .build()?;
    let mut project = Project::new("media-guide")?;
    project.add("cell", model.note()
        .field("front", Content::sequence([Content::text("Cell "), image.image()]))
        .field("back", sound.sound()))?;
    let output = project.build(BuildOptions::to("media-guide.apkg"))?;
    assert_eq!(output.report().counts().media, 3);
    Ok(())
}
```

类型化的 `image()` 和 `sound()` 值会保留依赖直至导出。手写 HTML、CSS 或脚本引用需要 `builder.asset(media)` 或 `project.add_asset(media)`；库不会从任意字符串推断文件内容。即使没有可被静态识别的引用，显式资源仍会被包含。

向 `with_export_name` 传入原始文件名。类型化声音引用在渲染时会转义 HTML 实体，因此 `tone&copy;.mp3` 仍引用该原名资源。不要预先转义文件名。声音文件名中的百分号和片段字符保持字面含义，并不是 URL 路径。

## 名称、MIME 与预算

默认导出名称由内容决定，与源路径无关。需要固定文件名时，应在创建引用前使用 `with_export_name`。重命名某个克隆不会改变之前的克隆或内容节点。

名称不能包含路径、控制字符或不可移植的保留名称。冲突检查使用 Unicode NFC 加大小写折叠：仅大小写或规范化形式不同也会冲突。名称与内容完全相同的重复操作具有幂等性；同名不同内容会失败，不留下部分注册状态。

包括扩展名在内的完整导出文件名最多为 255 个 UTF-8 字节。Project 与模板包的 JSON Schema 检查文件名语法和字符数；加载器还检查字节预算。例如，重复 128 次的 `é` 能通过 schema 字符限制，却占 256 字节，加载时返回 `MEDIA.EXPORT_NAME_INVALID`。构建前通过 `tools::load_project` 或 `NoteType::from_bundle` 校验完整输入。

MIME 语法必须与可识别的内容一致。原生字节没有 64 KiB 的内联限制。默认导入预算为每个资源 256 MiB；`file_with_limits` 和 `bytes_with_limits` 在读取前接收 `MediaLimits { max_bytes }`。大快照可能写入临时存储，并在最后一个所有者消失后清理。快照所有权不保证在源文件同时被修改的导入过程中获得原子的文件系统视图。

Project JSON 解码内联 `bytes` 数组时使用相同的默认单资源预算。遇到首个超限字节便停止，不再收集剩余数组，并保留带限制和已观察字节数的 `MediaError`。拒绝时的计数是下界，并非尚未读取数组的总大小。

有效的媒体包并不证明所有 Anki 客户端都能播放其编解码格式。请在目标客户端测试图片、声音和字体。参见[排查问题](../troubleshooting.md#media-and-templates)。

### 类型化图片与声音的使用

构造函数保持不会失败。`Project::add` 检查导入时保留的 MIME：图片要求 `image/*`，声音引用接受 `audio/*` 和 `video/*`。`application/octet-stream`、文本、字体和 PDF 的类型化用法会返回 `NOTE.MEDIA_USAGE_INVALID`；用于原始 HTML/CSS 时应注册为显式资源。有效但未知的图片或视频 MIME 声明可以通过类别校验，但这并不认证解码器或播放器支持这些字节。重命名文件不会改变已保留的 MIME。构建仍会探测暂存字节并独立校验导出扩展名：名为 `wrong.mp3` 的 PNG 可以通过图片添加，却在构建时返回 `MEDIA.DECLARED_MIME_MISMATCH`。

<a id="batch-import-from-source"></a>

## 批量媒体导入

这些方法已包含在公开 `0.3.1` 包中。

以受限并发导入一组文件，保持输入顺序，并保留拥有所有权的媒体快照供之后导出：

```rust
use ankiforge::Media;
let images = Media::files(["first.png", "second.png"])?;
```

Node 使用 `await Media.files(['first.png', 'second.png'])`；Python 使用 `Media.files(['first.png', 'second.png'])`。

### 批量行为与预算

Rust 使用 `Media::files(paths)`，Node 使用 `await Media.files(paths)`，Python 使用 `Media.files(paths)`，按输入顺序导入文件，每次调用最多四个工作线程。少于 16 个文件时串行运行。Rust 还提供 `files_with_limits`；Node 和 Python 接受与 `file` 相同的限制。限制按每个资源计算。若某项导入失败，调用会等待已启动的工作完成、释放此批次的快照，并返回输入顺序中的第一个错误。已有媒体所有者继续有效。Python 在导入期间释放 GIL；Node 在事件循环之外执行工作。单文件导入仍立即产生拥有所有权的快照；调用方必须选择批量方法才能获得导入并发。

存活快照共享进程级 **64 MiB** 内存预算。单个大于 1 MiB 的资源仍流式写入临时文件。编码媒体在每次预备处理中共享延迟分配的 **16 MiB** 缓冲池，池满后写入临时存储。这些预算不包含其他工作内存，也不是 RSS 上限；并发构建各自可以拥有编码池。
