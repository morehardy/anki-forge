# 模板包

模板包是一组可复用的模型声明、模板文本、CSS 和显式媒体依赖闭包。`NoteType::from_bundle(directory)` 校验完整模板包并取得拥有所有权的快照，再返回与内联编写相同的不可变 `NoteType`。它不会部分修改项目。

## 运行模板包示例

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

该流程加载[自定义 Cloze 测试模板](../../contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml)，写出 `target/docs-examples/template.apkg`。模板目标为 **Languages::Cloze**，一条笔记生成两张卡片。

## Manifest 与模板引用

Manifest 文件名为 `anki-template.yaml`。只接受新版格式：

<!-- source: contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml -->
```yaml
format_version: template-bundle-v2
note_type:
  key: language-cloze
  name: Language Cloze
  cloze_field: text
  fields:
    - key: text
      name: Sentence
      sort: true
      required: true
    - key: extra
      name: Extra
  templates:
    - key: cloze
      name: Cloze
      front_file: front.html
      back_file: back.html
      browser_front_file: browser-front.html
      target_deck: Languages::Cloze
css_file: style.css
```
<!-- /source -->

`note_type.key`、字段键和模板键是必填稳定标识符。`name` 可选，默认为对应键。`cloze_field` 选择 Cloze 模型，不需要单独声明 kind。未设置 `required: true` 的字段可以省略。未知 manifest 属性会被拒绝。

所有模板都使用**字段键**，包括正面、背面和浏览器变体：`{{cloze:text}}`、`{{extra}}` 以及 `{{#extra}}` 等条件区块。Sentence 和 Extra 等显示名称在绑定完成后为 Anki 生成。加载器在错误中保留原始文件路径和模板字节范围。

## 加载与复用模型

<!-- source: anki_forge/examples/docs_workflow.rs#bundle -->
```rust
fn bundle(output: &Path, template_directory: &Path) -> anyhow::Result<()> {
    let model = NoteType::from_bundle(template_directory)?;
    let mut project = Project::new("docs-languages")?;
    project.add(
        "capital",
        model
            .note()
            .field("text", "{{c1::Madrid}} is in {{c2::Spain}}.")
            .field("extra", "A city and its country."),
    )?;
    let built = project.build(BuildOptions::to(output.join("template.apkg")))?;
    verify(&built, 1, 2, 0)?;
    Ok(())
}
```
<!-- /source -->

在独立应用中，以下完整示例要求把同一模板包复制到 `fixtures/template-bundle`：

```rust
use ankiforge::{BuildOptions, NoteType, Project};

fn main() -> anyhow::Result<()> {
    let model = NoteType::from_bundle("fixtures/template-bundle")?;
    let mut project = Project::new("bundle-example")?;
    project.add("capital", model.note()
        .field("text", "{{c1::Madrid}} is in {{c2::Spain}}.")
        .field("extra", "A city and its country."))?;
    let output = project.build(BuildOptions::to("bundle-example.apkg"))?;
    assert_eq!(output.report().counts().cards, 2);
    Ok(())
}
```

返回的模型不依赖源目录。可以与另一项目共享，也可在源目录删除后继续使用。加载失败不会返回部分模型，并会清理未共享的快照。

## 资源与预算

显式声明所有原始模板、CSS 和脚本资源。每个 `assets` 条目使用相对于模板包的路径和固定导出名称：

```yaml
assets:
  - path: assets/badge.png
    export_as: badge.png
  - path: assets/labels.woff
    export_as: labels.woff
```

然后模板可使用 `<img src="badge.png">`，CSS 可使用 `url('labels.woff')`。`css_file` 指定 CSS 源文件；资源不会从任意 HTML、CSS 或 JavaScript 中自动推断。声明的资源即使没有检测到静态引用也保留在 APKG 中。导出名称遵守与[媒体](../media.md)相同的可移植名称及 Unicode/大小写冲突规则。包括扩展名在内的完整导出名称最多为 255 个 UTF-8 字节。JSON Schema 检查语法和字符数；`NoteType::from_bundle` 还会在返回已校验模型前执行字节预算、文件内容与引用检查。仅通过 schema 检查并不能保证加载成功。

绝对路径、目录穿越和指向模板包外部的符号链接会被拒绝。Manifest 上限为 256 KiB；每个模板或 CSS 文本文件上限为 2 MiB，且必须为 UTF-8。资源快照使用默认媒体预算，也可由调用方通过 `NoteType::from_bundle_with_limits` 提供 `MediaLimits`。预检长度和实际读取字节数都会检查。

模板包错误保留 kind/code、源路径、适用时的字节偏移，以及底层 schema、I/O、YAML、UTF-8 或媒体错误。模型完成时校验 schema；添加笔记时另外检查项目级冲突。参见[自定义笔记类型](../custom-notetypes.md)和[排查问题](../troubleshooting.md)。
