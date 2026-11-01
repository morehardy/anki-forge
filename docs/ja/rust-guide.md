# Rust での作成ガイド

1つの `Project` は1つの配布物を表します。安定した名前空間で作成し、安定キーでノートを追加して APKG をビルドします。よく使う型は `ankiforge` から直接インポートできます。詳細なオプションは公開されている機能別モジュールにあります。

<!-- section: projects-and-diagnostics -->
## プロジェクトと診断

```rust
use ankiforge::{BuildOptions, Content, Note, Project};

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("jp-core")?
        .default_deck("Japanese::Core");
    project.add("taberu", Note::basic("食べる", Content::html("<b>to eat</b>")))?;
    let output = project.build(BuildOptions::to("jp-core.apkg"))?;
    println!("{} notes", output.report().counts().notes);
    println!("{}", serde_json::to_string_pretty(&output.snapshot())?);
    Ok(())
}
```

ビルドに成功すると、成果物の存在が保証された `BuildOutput` を返します。`BuildReport` は件数、診断、経過時間、完了した比較などの観測結果を保持します。成功・失敗の結果や成果物の所有権は保持しません。警告があっても、成功した出力が失敗に変わることはありません。

スキーマのエラーはモデルの完成時に検査されます。`Project::add` は、ノートのキー、フィールド参照、必須フィールド、デッキ、タグ、依存関係の競合をアトミックに検査します。失敗してもプロジェクトは変わりません。残りのカード・パッケージ・更新の検査はビルド時に行います。別途変更可能なレジストリや、必須の検証チェックポイントはありません。

エラーには固有の `kind()` と `code()` メソッドがあり、`std::error::Error` を実装しています。表示メッセージを解析せず、機械処理用コードを使ってください。`BuildError::snapshot()` には実際の失敗、観測結果、原因チェーンのテキスト、公開についての事実が含まれます。標準的なエラーラッパーでは、元のエラーを source として保持できます。

<!-- section: models-and-assets -->
## モデルとアセット

完成した `NoteType` は不変で共有可能です。`model.note()` を呼び出し、安定キーでフィールドを設定して、結果をプロジェクトに追加します。ノートはモデルを保持し、追加時にモデルとメディアを自動収集します。宣言とテンプレートのルールは[独自のノートタイプ](/docs/custom-notetypes/)を参照してください。

`Media::file` と `Media::bytes` はデータを所有するスナップショットを取得します。インポートに成功した後は、元ファイルを残しておく必要はありません。`image.image()` と `audio.sound()` は構造化コンテンツを生成し、`Content::sequence` はそれらをテキストや HTML と組み合わせます。直接記述した HTML・CSS・スクリプトのアセットは、`builder.asset` または `project.add_asset` で明示的に追加する必要があります。[メディア](/docs/media/)と[テンプレートバンドル](/docs/templates/)を参照してください。

<!-- section: output-ownership -->
## 出力の所有権

永続出力には `BuildOptions::to(path)`、所有権によって寿命を管理する一時ファイルには `BuildOptions::temporary()` を選びます。一時パスにアクセスしている間は、出力または `output.artifact()` のクローンを保持してください。最後の所有者が解放されるとファイルが削除されます。`persist_to(path)` は永続成果物を返します。失敗した場合も元の所有者は使えます。JSON スナップショットにはパスが含まれますが、ファイルは所有しません。

<!-- section: stable-note-identity -->
## ノートの安定した識別情報

`project.add(key, note)` には永続的なソースレコードのキーを使います。プロジェクトの名前空間、モデルキー、フィールドキー、テンプレートキーは表示名から独立しています。内容の変更によって新しい論理ノートが作られることはありません。キーが重複すると、上書きせず失敗します。ソースレコードの並べ替えや編集後も、選んだキーを保存・再使用してください。

<!-- section: updating-distributed-decks -->
## 配布済みデッキを更新する

`update_from` に前回配布時のオリジナル APKG を渡します。内部の完全な証拠によって、識別情報とリビジョンの対応を保持します。独立した `project.compare(CompareOptions::against(path))` は候補を解析し、ポリシーが公開を許可するかどうかを報告します。高リスクの変更、クライアントでのインポートの制約、明示的な許可は、[2つのバージョンを使う手順](/docs/updates/)を参照してください。

チェックアウトしたリポジトリでは、次の例で同じ公開 API を実行できます。

```sh
cargo run --locked -p ankiforge --example target_api_basic
cargo run --locked -p ankiforge --example target_api_custom_notetype
cargo run --locked -p ankiforge --example target_api_media
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

<!-- section: addition-errors-and-media-categories -->
### 追加時のエラーとメディアの種類

`Project` は安定した名前空間と任意の既定デッキを使い、表示タイトルは持ちません。`NoteType`、`Field`、`Template` は引き続き表示名をサポートします。`Project::add` はアトミックです。ノート・モデルのキーは `AddError::context()`、元の位置は `note::AddTarget` で確認します。`detail()` は競合や互換性のないメディア用途に対する `note::AddDetail` を返します。これらの非網羅的な列挙型をマッチする際は、データ付きバリアントに `..` を使い、フォールバックの分岐を用意してください。フィールドの `content_path` は、フィールド自体なら `None`、ルートなら `Some([])`、入れ子のノードならシーケンスのインデックスになります。区切り文字のバイト範囲は元の UTF-8 Text/Html リーフを指します。

`Media::image()` と `sound()` 自体は引き続き失敗しません。ノートへの追加では、画像に `image/*`、音声参照に `audio/*` または `video/*` が必要です。それ以外の種類は `NOTE.MEDIA_USAGE_INVALID` になります。これは保持された MIME の検査であり、デコードや再生の確認ではありません。直接記述した HTML と明示的アセットの動作は変わりません。エクスポート名を変更しても MIME は変わりません。PNG を `wrong.mp3` に改名しても画像としての追加は通りますが、ビルド時の独立した内容判定・拡張子検査で `MEDIA.DECLARED_MIME_MISMATCH` になることがあります。

ツール用レシピにはトップレベルの `name` を持たない `ankiforge-project-v2` が必要です。v1 は拒否されます。契約バンドルは 2.1.0 です。パッケージの識別情報は引き続き `ankiforge-identity-v1` です。
