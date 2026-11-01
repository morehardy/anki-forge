# Rust アプリケーションに Anki Forge を組み込む

Rust 1.92 以降を使用します。既存の Cargo アプリケーションに公開 crate をインストールします。

@@CODE_0@@

通常の APKG エクスポートに Anki のインストールは必要ありません。最初の完全なプログラムと `cargo run` の手順は、[Rust クイックスタート](/docs/quickstart/)を参照してください。

<!-- section: cargo-dependencies -->
## Cargo の依存関係

公開 `ankiforge` crate には、Basic、Cloze、独自のノートタイプ、データを所有するメディア、画像穴埋め、比較、準備済み公開が含まれます。デフォルトの feature は空です。通常のアプリケーションコードでリポジトリ専用の feature を有効にする必要はありません。

crate はネイティブの Rust ライブラリです。Rust ツールチェーンに必要な、利用環境のリンカーとビルドツールをインストールしてください。デプロイ先を選ぶ際は[検証済みの環境](/docs/compatibility/)を参照してください。最小要件は Rust 1.92 です。一覧に載っているプラットフォームでも、すべてのターゲットやサードパーティ製ツールチェーンの設定で動くことを保証するものではありません。

JSON を入力に使う場合は、呼び出し側でパース用の依存関係を明示的に追加します。

@@CODE_1@@

[JSON から APKG を生成するレシピ](/docs/json-to-anki/#rust)には、パーサー、メディア処理、更新手順を含む完全な例があります。SDK 自体は汎用の JSON・CSV 変換機能を提供しません。

<!-- section: choose-the-next-guide -->
## 次のガイドを選ぶ

- [Rust での作成ガイド](/docs/rust-guide/): プロジェクトとモデルの設計
- [Rust API](/docs/rust-api/): メソッド、レポート、エラー
- [メディア](/docs/media/): 画像・音声データの所有
- [更新と配布](/docs/updates/): 基準パッケージと公開の条件
- [ソースからのビルド](/docs/development/): リポジトリでの開発
