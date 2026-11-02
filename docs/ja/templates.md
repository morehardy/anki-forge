# テンプレートバンドル

ノートタイプのテンプレート、CSS、メディアを1つのディレクトリにまとめ、通常カードや Cloze カード用に読み込むことで、同じ Anki カードのデザインを複数のデッキで再使用できます。例では Rust を使います。Python と Node.js では `NoteType.from_bundle` と `NoteType.fromBundle` でバンドルを読み込めます（[Python API](/docs/python-api/#models-and-templates) と [Node API](/docs/node-api/#immutable-models) を参照）。

バンドルは、再使用可能なモデル宣言、テンプレートのテキスト、CSS、明示的に宣言したすべての依存メディアをまとめたものです。`NoteType::from_bundle(directory)` はバンドル全体を検証し、データを所有するスナップショットを取得してから、コード内での作成と同じ不変の `NoteType` を返します。プロジェクトを部分的に変更することはありません。

<!-- section: run-the-bundled-example -->
## 同梱サンプルを実行する

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

この手順は[独自 Cloze のフィクスチャー](https://github.com/morehardy/anki-forge/blob/main/contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml)を読み込み、`target/docs-examples/template.apkg` を書き出します。テンプレートの配置先は **Languages::Cloze** です。1件のノートから2枚のカードを生成します。

<!-- section: manifest-and-template-references -->
## マニフェストとテンプレートの参照

マニフェストのファイル名は `anki-template.yaml` です。受け付けるのは新しい形式だけです。

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

`note_type.key`、フィールドキー、テンプレートキーは必須の安定識別子です。`name` は任意で、省略時は対応するキーになります。`cloze_field` で Cloze モデルを選択します。種類の宣言を別途指定する必要はありません。`required: true` がないフィールドは省略できます。不明なマニフェストのプロパティは拒否されます。

表面・裏面・ブラウザー用を含むすべてのテンプレートで**フィールドキー**を使います。例は `{{cloze:text}}`、`{{extra}}`、`{{#extra}}` などの条件セクションです。Sentence や Extra などの表示名は、参照を結び付けた後に Anki 用として生成されます。ローダーはエラーに元のファイルパスとテンプレートのバイト範囲を保持します。

<!-- section: load-and-reuse-a-model -->
## モデルを読み込んで再使用する

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

独立したアプリケーションで使う次の完全な例では、同じバンドルを `fixtures/template-bundle` にコピーしておきます。

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

返されたモデルは元のディレクトリから独立しています。別のプロジェクトと共有でき、ディレクトリを削除した後も使えます。読み込みに失敗した場合は、部分的なモデルを返さず、他から共有されていないスナップショットを削除します。

<!-- section: assets-and-budgets -->
## アセットと上限

直接記述したテンプレート・CSS・スクリプトで使うアセットは、すべて明示的に宣言します。各 `assets` 項目には、バンドルからの相対パスと固定のエクスポート名を指定します。

```yaml
assets:
  - path: assets/badge.png
    export_as: badge.png
  - path: assets/labels.woff
    export_as: labels.woff
```

これにより、テンプレートで `<img src="badge.png">`、CSS で `url('labels.woff')` を使えます。`css_file` プロパティは CSS ソースファイルを指定します。任意の HTML・CSS・JavaScript からアセットを推測することはありません。宣言したアセットは静的参照が検出されなくても APKG に残ります。エクスポート名には、[Media](/docs/media/) と同じ移植可能な名前の規則と、Unicode・大文字小文字の衝突規則が適用されます。拡張子を含めたエクスポート名全体の上限は UTF-8 で255バイトです。JSON Schema は構文と文字数を検査します。`NoteType::from_bundle` は検証済みモデルを返す前に、バイト数、ファイル内容、参照も検査します。スキーマ検査の成功だけで読み込み成功が保証されるわけではありません。

絶対パス、バンドル外へのパストラバーサル、バンドル外を指すシンボリックリンクは拒否されます。マニフェストの上限は256 KiBです。各テンプレート・CSS テキストファイルの上限は2 MiBで、UTF-8 である必要があります。アセットのスナップショットには既定のメディア上限、または `NoteType::from_bundle_with_limits` で呼び出し側が指定した `MediaLimits` を使います。事前のファイル長と実際に読み取ったバイト数の両方を検査します。

バンドルのエラーには kind/code、ソースパス、該当する場合はバイトオフセット、および原因となったスキーマ・I/O・YAML・UTF-8・メディアのエラーが保持されます。モデル完成時にスキーマを検証し、ノートの追加時にはプロジェクト全体での競合を別に検査します。[独自のノートタイプ](/docs/custom-notetypes/)と[トラブルシューティング](/docs/troubleshooting/)を参照してください。
