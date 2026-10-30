# 图片遮挡

用代码批量生成 Anki 图片遮挡（图片挖空）卡片：提供图片和矩形坐标，Anki Forge 生成笔记，每个遮罩对应一张卡片，可直接导入。适合批量制作示意图题目，无需在 Anki 编辑器中逐个绘制遮罩。本库不会自动识别图中的标签，坐标由你提供。

使用拥有所有权的图像与矩形创建结构化图片遮挡笔记。每个遮罩需要稳定键，在后续版本中继续标识同一区域。笔记通过普通 `add(key, note)` 加入项目。示例使用 Rust；Python 和 Node.js 提供相同的构建器，参见 [Python API](../python/api.md#media-and-occlusion) 和 [Node API](../node/api.md#media-and-image-occlusion)。

## 运行示例

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

在 Anki 中打开 `target/docs-examples/diagram.apkg`。完整可执行程序加载仓库的遮挡图像，并检查一条笔记、一张卡片和一个资源。

<!-- source: anki_forge/examples/docs_workflow.rs#occlusion -->
```rust
fn occlusion(output: &Path, diagram: &Path) -> anyhow::Result<()> {
    let image = Media::file(diagram)?;
    let note = Note::image_occlusion(image)
        .mask(Mask::rect("region-1", 10, 10, 40, 25))
        .build()?;
    let mut project = Project::new("docs-diagram")?.default_deck("Diagram");
    project.add("diagram", note)?;
    let built = project.build(BuildOptions::to(output.join("diagram.apkg")))?;
    verify(&built, 1, 1, 1)?;
    Ok(())
}
```
<!-- /source -->

## 多个遮罩与模式

此独立示例需要 `fixtures/occlusion.png`，解码后的图像至少为 80×80 像素：

```rust
use ankiforge::{BuildOptions, Media, Note, Project};
use ankiforge::note::{Mask, OcclusionMode};

fn main() -> anyhow::Result<()> {
    let note = Note::image_occlusion(Media::file("fixtures/occlusion.png")?)
        .mask(Mask::rect("nucleus", 10, 10, 20, 20))
        .mask(Mask::rect("wall", 50, 50, 20, 20))
        .mode(OcclusionMode::HideOneGuessOne)
        .build()?
        .field("header", "Cell structure")
        .field("back_extra", "Review the diagram.");
    let mut project = Project::new("cell-diagram")?;
    project.add("cell", note)?;
    let output = project.build(BuildOptions::to("cell-diagram.apkg"))?;
    assert_eq!(output.report().counts().cards, 2);
    Ok(())
}
```

默认模式为 `HideAllGuessOne`，也支持 `HideOneGuessOne`。每个带键遮罩生成自己的 Cloze 卡片。坐标是应用 EXIF 方向后的显示图像上的有限像素值。库完整解码 PNG、JPEG、GIF、WebP 和 BMP；解码缓冲预算为 256 MiB，与压缩媒体的导入预算独立。矩形尺寸必须为正、位于图像内，并使用唯一的非空键。

构建器在返回 `Note` 前校验图像和遮罩。使用普通 `field` 为 `header`、`back_extra` 和 `comments` 赋值；自动生成的 `image` 和 `occlusion` 字段为保留字段，不能覆盖。

基于之前分发的包更新时，遮罩键会在顺序改变后保留卡片序号。已移除遮罩的序号仍被保留；恢复相同键会复用原序号。新键不会继承其他遮罩的身份。支持的分配范围为 1–500，用尽时明确失败。删除或添加遮罩会出现在[更新风险分析](../updates.md)中；APKG 中省略卡片不会删除学习者已有的卡片。
