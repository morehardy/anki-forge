使用 Rust 1.92 或更高版本。其他语言可选择 [Node / TypeScript 或 Python](/docs/languages/)。

## 安装

```sh
cargo new anki-deck
cd anki-deck
cargo add ankiforge@0.3.1
```

## 导出第一个牌组

将这份完整程序保存为 `src/main.rs`：

<!-- source: anki_forge/examples/target_api_basic.rs -->
```rust
use ankiforge::{BuildOptions, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", "hello"))?;
    let output = project.build(BuildOptions::to("spanish.apkg"))?;
    println!("{}", output.artifact().path().display());
    Ok(())
}
```
<!-- /source -->

```sh
cargo run
```

程序写出持久保存的 `spanish.apkg`。在 Anki 中打开并导入后，可在 **Spanish** 牌组中学习 **hola → hello**。导出不需要安装 Anki。

命名空间 `spanish` 和稳定键 `es:hola` 在之后的内容修改中标识这条笔记。也可以[下载本站 Basic 示例](/generated/spanish.apkg)。

## 继续学习

[JSON 到 Anki 示例](/docs/json-to-anki/#rust)提供完整的媒体与更新任务。

添加[图片与音频](/docs/media/)、[自定义笔记类型](/docs/custom-notetypes/)或[图片遮挡](/docs/image-occlusion/)。[比较并发布更新](/docs/updates/)前，请保留原始分发 APKG。详情见 [Rust API](/docs/rust-api/)和[已验证环境](/docs/compatibility/)。
