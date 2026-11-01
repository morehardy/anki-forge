# 公開 API の概要

Anki Forge は Rust コアと Rust・Python・Node.js のインターフェースを持つ、オープンソースの Anki APKG 生成ライブラリです。公開 `ankiforge` パッケージでノートを作成し、メディアを同梱して、新しいデッキを配布する前に更新内容を確認できます。MIT ライセンスで公開されています。

| 作業 | Rust | Python | Node / TypeScript |
| --- | --- | --- | --- |
| ノートとモデルの作成 | [Rust API](/docs/rust-api/) | [Python API](/docs/python-api/) | [Node API](/docs/node-api/) |
| 画像、音声、画像穴埋め | [メディア](/docs/media/) | [メディアの所有](/docs/python-api/#media-and-occlusion) | [メディアの所有](/docs/node-api/#media-and-image-occlusion) |
| エラーの診断 | [トラブルシューティング](/docs/troubleshooting/) | [診断](/docs/python-diagnostics/) | [エラー](/docs/node-api/#errors) |
| 比較と公開 | [更新の手順](/docs/updates/) | [ビルドと比較](/docs/python-api/) | [ビルドと比較](/docs/node-api/#build-compare-and-output) |

<!-- section: stable-note-identity -->
## ノートの安定した識別情報

各論理ノートについて、プロジェクトの名前空間とノートキーを安定して維持します。既存リリースを更新するには、配布時のオリジナル APKG も必要です。Anki から再エクスポートしたファイルは、オリジナルに含まれる完全な識別情報の証拠の代わりにはなりません。

<!-- section: start-with-a-task -->
## 作業から始める

[使用する言語を選ぶ](/docs/languages/)か、[JSON から生成・更新する完全なレシピ](/docs/json-to-anki/)に沿って進めてください。公開パッケージの実行環境は[互換性](/docs/compatibility/)、ファイルの所有権は[出力の保証](/docs/build-guarantees/)を参照してください。
