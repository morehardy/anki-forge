# 自定义笔记类型

先完成模型，再从模型创建笔记。返回的 `NoteType` 不可变，可在函数或多个项目中复用。笔记持有自己的模型，因此不会意外引用尚未注册的模型名称。

## 可执行示例

本例分别定义稳定键和显示名称，按键赋值，并在模板中使用相同的键。

<!-- source: anki_forge/examples/target_api_custom_notetype.rs -->
```rust
use ankiforge::schema::GenerationRule;
use ankiforge::{BuildOptions, Field, NoteType, Project, Template};

fn main() -> anyhow::Result<()> {
    let vocab = NoteType::builder("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("expr").name("Expression").sort().required())
        .field(Field::new("meaning").name("Meaning").required())
        .template(
            Template::new("recognition")
                .name("Recognition")
                .front("{{expr}}")
                .back("{{FrontSide}}<hr id=\"answer\">{{meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .build()?;
    let mut project = Project::new("jp-core")?.default_deck("Japanese::Core");
    project.add(
        "taberu",
        vocab
            .note()
            .field("expr", "食べる")
            .field("meaning", "to eat"),
    )?;
    project.build(BuildOptions::to("jp-core.apkg"))?;
    Ok(())
}
```
<!-- /source -->

在源码目录运行 `cargo run --locked -p ankiforge --example target_api_custom_notetype`，然后在 Anki 中打开 `jp-core.apkg`。

## 键与名称

`Field::new` 和 `Template::new` 接收稳定键。`.name` 提供显示名称；未设置时名称等于键。不会从名称派生 slug。空键、空白键以及冲突或保留键会使模型完成失败。名称可以包含中文或其他 Unicode 文本，不影响身份键。

模板绑定键，并编译成 Anki 字段名称。该过程涵盖正反面、浏览器模板、条件区块、过滤器和 Cloze。其他 HTML、空白和脚本文本保持不变。未知引用会返回包含源字节范围的错误。字段赋值只接受键，不会静默去除空白或忽略大小写。

## 校验与生成

`required()` 拒绝缺失或空的笔记内容。未设置它的字段可以省略，并渲染为空。`sort()` 选择排序字段；最多只能有一个。模板未显式设置 `GenerationRule::all` 或 `any` 时，使用 Anki 的默认生成规则。这些规则引用字段键。`builder.cloze_field(key)` 为自定义模型选择 Cloze 语义。

`builder.asset(media)` 声明拥有所有权的 CSS、字体、脚本或原始 HTML 依赖。笔记的类型化图片与声音内容自动收集资源。即使没有发现静态引用，显式资源仍保留在输出中。

同一项目中复用模型键时，定义必须相同。添加笔记时，冲突的定义会原子地失败。后续版本中更改显示名称、字段或模板，即使稳定键不变，也需要进行[更新分析](../updates.md)。

需要复用文件和资源时，使用[模板包](../template-bundles.md)。
