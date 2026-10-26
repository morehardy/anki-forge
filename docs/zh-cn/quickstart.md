# 用 Rust 生成第一个 Anki 牌组

使用 Rust 1.92 或更新版本。

## 安装

在新的项目目录中安装公开版本 0.3.1：

```sh
cargo new anki-deck
cd anki-deck
cargo add ankiforge@0.3.1
```

具体平台范围见[兼容性说明（英文）](../compatibility.md)。

## 生成 APKG

将下面的完整程序保存为 `src/main.rs`：

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

运行：

```sh
cargo run
```

程序会写出持久保存的 `spanish.apkg`。在 Anki 中导入后，可以在 Spanish 牌组学习 **hola → hello**。
生成 APKG 不需要安装 Anki。后续修改答案时，保持 namespace `spanish` 和 note key `es:hola` 不变。

## 继续完成实际任务

[JSON 教程](json-to-anki.md#rust)包含图片、音频和更新流程。
发布更新时请保留上一份原始分发 APKG，不能用 Anki 重新导出的文件代替。
查看[更新条件（英文）](../updates.md)、[API 参考（英文）](../api-overview.md)，
或[选择其他开发语言](languages.md)。
