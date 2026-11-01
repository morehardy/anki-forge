# Anki Forge と genanki の比較

繰り返し公開する手順を設計し、メディアの所有、構造化された診断、更新のリリース前の比較・ポリシー確認が必要なら、Anki Forge を検討してください。genanki の Python スクリプト、テンプレート、メディアがすでに要件を満たし、移行の費用が利点を上回るなら、実績のある既存の手順を維持する選択肢もあります。

| 判断項目 | Anki Forge | genanki |
| --- | --- | --- |
| 作成に使う言語 | Rust コアと Rust・Python・Node / TypeScript のインターフェース | Python |
| 共通の基本機能 | Basic、Cloze、独自テンプレート、メディア、オフラインでの APKG 生成 | Basic、Cloze、独自テンプレート、メディア、オフラインでの APKG 生成 |
| 識別情報 | 明示的な名前空間とノートキー。更新にはオリジナル APKG を保持 | 設定可能な GUID と数値のモデル・デッキ ID。再インポートによる更新に対応 |
| リリース前の確認 | 比較の検出事項、リスクポリシー、検査済みの準備済み候補 | スクリプトとテストでリリース確認を定義 |
| メディアの所有 | 型付きの画像・音声がデータを所有するスナップショットを保持 | スクリプトがパッケージのメディアファイルと参照を渡す |
| プログラムからの画像穴埋め | 公開 API でキー付き矩形マスクを作成 | 作業に適したモデルや拡張を評価する |
| 実行環境のコスト | ネイティブ依存関係があり、対応プラットフォームとインタープリターを使う | 既存の Python 環境を維持する方が低コストな場合がある |
| 移行のコスト | 旧識別情報の対応、テンプレート、インポートを明示的に検証する | 動作中の genanki パイプラインなら移行コストは不要 |

<!-- section: the-same-small-task -->
## 同じ小さな作業で比べる

JSON の3つの質問を、画像と音声を1つずつ含むデッキにする場合、どちらのライブラリも Python で JSON をパースしてノートを作成できます。genanki ではモデル ID とデッキ ID を選び、フィールド値と GUID のルールを設定し、メディアファイルをパッケージに追加して APKG を書き出します。Anki Forge では名前空間と永続的な業務キーを選び、データを所有するメディアをノートに組み込み、v1 をビルドします。その後、公開前にオリジナル v1 APKG を基準として v2 を比較・準備します。[完全な JSON レシピ](/docs/json-to-anki/)には、Rust、Python、TypeScript の実装と実行コマンドがあります。

<!-- section: existing-genanki-decks -->
## 既存の genanki デッキ

[genanki は独自 GUID](https://github.com/kerrickstaley/genanki#note-guids)、メディア、再インポート時の更新に対応しています。安定した識別情報は Anki Forge だけの機能ではありません。既定の GUID ルールはフィールドに依存することがあるため、フィールドを変更する予定なら、意図した GUID のルールを選んでください。

Anki Forge は任意の genanki の数値 ID や GUID を自動復元しません。別途検証済みの対応付けがなければ、最初の Anki Forge 配布物は識別情報を引き継がない境界として扱ってください。この比較も APKG のビルド成功も、情報を失わない移行や復習履歴の完全な保持を約束するものではありません。既存デッキを移す前に[移行ガイド](/docs/genanki-migration/)を読んでください。

<!-- section: historical-performance-evidence -->
## 過去の性能測定

2026-10-08の比較では、Apple M1 Pro 上で Rust の Project API と genanki 0.13.1 を測定しました。Basic ノート1,000件では、測定した5種類の Rust ワークロードでエクスポート時間が45.7〜53.9%短縮しました。一部のメディア処理ではピークメモリ使用量が増えました。[測定方法、元データ、制約](https://github.com/morehardy/anki-forge/blob/bef4aeb653fc875f216614e73d8a617be039b9cf/benchmarks/results/20261008-latest-commit-genanki/report.md)を確認できます。この測定は現行 Python・Node ホストの性能を対象とせず、それらの SDK や別の処理で同等の速度を保証しません。

Anki Forge メンテナーが2026-10-25に、公開 Anki Forge 0.3.1 API、[genanki のドキュメント](https://github.com/kerrickstaley/genanki)、保存済みの測定結果に照らして確認しました。新しい作業を始めるには、[実行環境の互換性](/docs/compatibility/)を確認し、[言語を選んでください](/docs/languages/)。
