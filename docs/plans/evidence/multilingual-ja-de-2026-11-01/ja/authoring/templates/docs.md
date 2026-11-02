# コードから Anki デッキを生成する

Anki Forge は、ノート、テンプレート、メディアから Anki の `.apkg` ファイルを生成します。学習コンテンツをコードや構造化データで管理し、繰り返しデッキを生成・配布したいときに使えます。

<!-- section: start-with-your-language -->
## 使用するプログラミング言語から始める

公開パッケージをインストールし、最初のデッキをエクスポートします。[使用する言語を選んでください](/docs/languages/)。

| 言語 | 最初の成果物 | 次のステップ |
| --- | --- | --- |
| Rust | [最初のデッキを生成する例を実行する](/docs/quickstart/) | [アプリケーションに組み込む](/docs/installation/) |
| Node / TypeScript | [インストールして最初のデッキを生成する](/docs/node-quickstart/) | [API と非同期処理](/docs/node-api/) |
| Python | [インストールしてデッキをエクスポートする](/docs/python-quickstart/) | [API と出力の所有権](/docs/python-api/) |

先に完成形を確認したい場合は、[カードのサンプル](/examples/)を見て、生成済みのパッケージをダウンロードできます。

<!-- section: understand-the-pieces -->
## 構成要素を理解する

ノートはフィールドを保持し、ノートタイプとそのテンプレートがフィールドからカードを生成します。Anki では、生成されたカードをデッキにまとめます。[基本概念](/docs/concepts/)では、この関係、データを所有する Project の値、安定した識別情報を説明します。

<!-- section: build-your-next-feature -->
## 機能を追加する

- [CSV から Anki を生成する](/docs/csv-to-anki/): Python で語彙の表を APKG に変換します。
- [JSON から Anki を生成する](/docs/json-to-anki/): 3言語でメディア付きカードを生成し、更新内容を確認します。
- [AI を使ったカード作成](/docs/ai-anki-cards/): 学習メモからカードの下書きを作り、解答を確認して APKG をエクスポートします。
- [Basic と Cloze](/docs/cards/): 質問と解答のカードや穴埋めカードを作成します。
- [画像と音声](/docs/media/): 実際のファイルを登録してカードに追加します。
- [独自のノートタイプ](/docs/custom-notetypes/)と[テンプレートバンドル](/docs/templates/): フィールド、レイアウト、CSS を定義します。
- [画像穴埋め](/docs/image-occlusion/): 画像内の領域から問題を作成します。
- [更新と配布](/docs/updates/): リリース間で識別情報とリビジョンの証拠を引き継ぎます。

カードタイプのガイドでは実行可能な Rust の例を使います。CSV のレシピは Python、JSON のレシピは3言語すべてに対応しています。Node と Python の API ページでは、それぞれのインターフェースと違いを説明します。

<!-- section: find-answers-and-contribute -->
## 問題を解決し、開発に参加する

操作に失敗した場合は[トラブルシューティング](/docs/troubleshooting/)を、実行環境の対応状況やクライアントでのインポート条件は[検証済みの環境](/docs/compatibility/)を参照してください。

Anki Forge は [MIT ライセンス](https://github.com/morehardy/anki-forge/blob/main/LICENSE)で公開されています。再現可能な問題は [GitHub](https://github.com/morehardy/anki-forge/issues) で報告できます。開発に参加する場合は[コントリビューター向けガイド](https://github.com/morehardy/anki-forge/blob/main/docs/development.md)を参照してください。
