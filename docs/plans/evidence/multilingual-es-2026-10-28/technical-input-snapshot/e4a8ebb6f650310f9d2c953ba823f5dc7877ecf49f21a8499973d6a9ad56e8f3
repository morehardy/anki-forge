# Basic 与 Cloze 卡片

Basic 笔记有正面和背面。Cloze 笔记针对每个受支持的填空编号生成一张卡片。本例创建两条笔记和三张卡片。

在仓库根目录运行：

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

在 Anki 中打开 `target/docs-examples/cards.apkg`。Basic 的问题为 **hola**；两张 Cloze 卡片分别隐藏 **Madrid** 和 **Spain**。

## 编写笔记

此函数摘自[完整可执行程序](../../anki_forge/examples/docs_workflow.rs)。其中的 `verify` 辅助函数检查成功输出的笔记、卡片与媒体数量。

<!-- source: anki_forge/examples/docs_workflow.rs#cards -->
```rust
fn cards(output: &Path) -> anyhow::Result<()> {
    let mut project = Project::new("docs-spanish")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", "hello"))?;
    project.add(
        "capital",
        Note::cloze("{{c1::Madrid}} is in {{c2::Spain}}.")
            .field("back_extra", "A city and its country."),
    )?;
    let built = project.build(BuildOptions::to(output.join("cards.apkg")))?;
    verify(&built, 2, 3, 0)?;
    Ok(())
}
```
<!-- /source -->

`Project::new` 的第一个参数是稳定命名空间。每次 `add` 都需要稳定的源记录键。把同一个键用于另一条笔记会失败；库不会根据内容自动分配键。基于之前分发的包构建更新时，在同一键下修改答案仍表示同一条逻辑笔记。

## 文本、字段与卡片身份

Basic 和 Cloze 的字符串均为 Text。有意使用 HTML 时，调用 `Content::html`；Text 内仍会识别 Cloze 填空语法。`back_extra` 是内置 Cloze 附加字段键。文本、HTML、图片与声音都使用同一个 `Note::field` 方法。

使用 `{{c1::answer}}` 或 `{{c1::answer::hint}}`。普通句子不会提供 Cloze 问题。支持的填空编号为 1–500；修改编号可能改变卡片身份。应通过[比较和更新策略](../updates.md)检查此类更改。

接下来可以添加[媒体](../media.md)、[自定义模型](../custom-notetypes.md)或[图片遮挡](../image-occlusion.md)。
