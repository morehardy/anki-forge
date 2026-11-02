# 独自のノートタイプ

モデルを一度完成させてから、そのモデルを使ってノートを作成します。返される `NoteType` は不変で、関数や複数のプロジェクトで再使用できます。ノート自体がモデルを保持するため、未登録のモデル名を誤って参照することはありません。

<!-- section: executable-example -->
## 実行可能な例

この例では安定キーと表示名を別々に定義し、キーで値を代入して、テンプレート内でも同じキーを使います。

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

チェックアウトしたリポジトリから `cargo run --locked -p ankiforge --example target_api_custom_notetype` を実行します。`jp-core.apkg` を Anki で開いてください。

<!-- section: keys-and-names -->
## キーと名前

`Field::new` と `Template::new` は安定キーを受け取ります。`.name` で表示名を指定でき、省略時はキーが表示名になります。名前からスラッグを生成することはありません。空のキー、空白だけのキー、前後に空白のあるキー、競合するキー、予約済みキーは、モデルの完成時にエラーになります。表示名には中国語などの Unicode テキストを使えます。識別キーは変わりません。

テンプレートはキーを参照し、Anki のフィールド名へコンパイルされます。対象は表面・裏面、ブラウザーテンプレート、条件セクション、フィルター、Cloze です。それ以外の HTML、空白、スクリプトのテキストは保持されます。不明な参照があると、ソース内のバイト範囲を示して失敗します。フィールドへの代入はキーだけを受け付け、暗黙の空白除去や大文字・小文字の正規化はしません。

<!-- section: validation-and-generation -->
## 検証と生成

`required()` はノートの内容が未設定または空の場合にエラーにします。指定しないフィールドは省略でき、空として表示されます。`sort()` は並べ替え用フィールドを選びます。選択できるのは最大1つです。テンプレートに `GenerationRule::all` または `any` を明示しない限り、生成には Anki の既定ルールを使います。これらのルールはフィールドキーを参照します。`builder.cloze_field(key)` は独自モデルで Cloze の動作を選択します。

`builder.asset(media)` は、データを所有する CSS・フォント・スクリプト・直接記述した HTML の依存ファイルを宣言します。ノート上の型付き画像・音声コンテンツはアセットを自動収集します。明示したアセットは静的な参照が見つからなくても出力に残ります。

プロジェクト内で同じモデルキーを再使用できるのは、定義が同一の場合だけです。定義が競合すると、ノート追加時に状態を部分的に変更することなく失敗します。後続リリースで表示名・フィールド・テンプレートを変更するときは、安定キーが変わらなくても[更新の解析](/docs/updates/)が必要です。

ファイルとアセットを再使用するには、[テンプレートバンドル](/docs/templates/)を使います。
