# Rust で Anki カードを生成する

Canonical page: https://ankiforge.dev/ja/docs/quickstart/
Language: ja
Description: Rust の公開パッケージをインストールし、スペイン語のカードを作成して、後から使える APKG ファイルとして保存します。

Rust 1.92 以降を使用します。別のプログラミング言語を使う場合は、[Node／TypeScript または Python](https://ankiforge.dev/ja/docs/languages/)を選んでください。

## インストール

```sh
cargo new anki-deck
cd anki-deck
cargo add ankiforge@0.3.1
```

## 最初のデッキをエクスポートする

次の完全なプログラムを `src/main.rs` に保存します。

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

```sh
cargo run
```

プログラムは永続ファイル `spanish.apkg` を書き出します。Anki で開いてインポートすると、**Spanish** デッキの **hola → hello** を学習できます。エクスポート時に Anki のインストールは必要ありません。

名前空間 `spanish` と安定キー `es:hola` がノートを識別するため、後で内容を変更しても同じノートとして扱えます。[サイトの Basic サンプルをダウンロード](https://ankiforge.dev/generated/spanish.apkg)することもできます。

## 次のステップ

メディアと更新を含む一連の手順は、[JSON から Anki を生成する例](https://ankiforge.dev/ja/docs/json-to-anki/#rust)で確認できます。

[画像と音声](https://ankiforge.dev/ja/docs/media/)、[独自のノートタイプ](https://ankiforge.dev/ja/docs/custom-notetypes/)、[画像穴埋め](https://ankiforge.dev/ja/docs/image-occlusion/)を追加できます。[更新を比較して公開する](https://ankiforge.dev/ja/docs/updates/)前に、配布時のオリジナル APKG を保存しておいてください。詳しくは [Rust API](https://ankiforge.dev/ja/docs/rust-api/) または[検証済みの環境](https://ankiforge.dev/ja/docs/compatibility/)を参照してください。
