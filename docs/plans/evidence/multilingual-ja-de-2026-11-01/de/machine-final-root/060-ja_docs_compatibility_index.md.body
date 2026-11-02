# 互換性と検証済みの環境

Canonical page: https://ankiforge.dev/ja/docs/compatibility/
Language: ja
Description: 公開パッケージ、検証した実行環境、Anki クライアントでのインポートと再生の条件を確認します。

## パッケージと検証済みの環境

2026-10-13の確認時点で、Rust・npm・PyPI の公開パッケージは `0.3.1` です。[Rust](https://ankiforge.dev/ja/docs/installation/)、[Node / TypeScript](https://ankiforge.dev/ja/docs/node-quickstart/)、[Python](https://ankiforge.dev/ja/docs/python-quickstart/) から始めてください。

| インターフェース | パッケージ | 実行環境と検証 |
| --- | --- | --- |
| Rust | [ankiforge 0.3.1](https://crates.io/crates/ankiforge) | Rust 1.92.0 と検証時の stable。対応する4プラットフォームすべてでパッケージ利用側を検査し、公開 crates.io パッケージの新規利用側も確認 |
| Node / TypeScript | [ankiforge 0.3.1](https://www.npmjs.com/package/ankiforge) | Node 22.13.0、検証時の22・24・26。公開 npm からのインストール、ESM/CJS、TypeScript の16件の検査 |
| Python | [ankiforge 0.3.1](https://pypi.org/project/ankiforge/) | 通常の CPython 3.11〜3.14。隔離環境で公開 PyPI wheel をインストールする16件の検査 |

公開ネイティブパッケージは macOS arm64/x64、Linux x64 GNU/glibc、Windows x64 に対応します。パッケージが存在するだけで、すべてのホストで実行テストが成功したことにはなりません。[Node の検証範囲](https://github.com/morehardy/anki-forge/blob/main/bindings/node/COVERAGE.md)、[Python の検証範囲](https://github.com/morehardy/anki-forge/blob/main/bindings/python/COVERAGE.md)、リリースの証拠に、実施した検査を記録しています。`0.3.1` の成功したリリース実行は [Rust](https://github.com/morehardy/anki-forge/actions/runs/38371221696)、[npm](https://github.com/morehardy/anki-forge/actions/runs/38371222114)、[PyPI](https://github.com/morehardy/anki-forge/actions/runs/38371222047) です。他の Python 実装、free-threaded インタープリター、サブインタープリターは別途検証が必要です。ブラウザー、Electron、Bun、Alpine/musl、ARM64 Linux/Windows は、記録済みの Node 検証範囲外です。

[メディアの一括インポート](https://ankiforge.dev/ja/docs/media/#batch-media-import)と[準備済み公開](https://ankiforge.dev/ja/docs/updates/#build-once-review-then-publish)のガイドで扱う機能は、公開 `0.3.1` パッケージで使用できます。

## Anki でのインポートと再生

通常のエクスポートに Anki のインストールは必要ありません。インポート、描画、メディアの再生は、対象クライアントのバージョン、コーデック、インポート設定に依存します。2026-10-08のベンチマークでは、Anki へのインポート・内容・代表的な描画の検査40件（各実装20件）が成功しました。すべてのクライアントや GUI・音声動作を検証したものではありません。

更新用に、前回配布時のオリジナル APKG を保持してください。Anki から再エクスポートしたファイルには、必要な識別情報の証拠が残りません。パッケージからノートを省いても、学習者のコレクションからは削除されません。フィールド・テンプレートの変更、並べ替え用フィールド、ローカルの編集、復習履歴は、クライアントごとのテストが必要です。

自分の配布物は、[手動クライアント検証シナリオ](https://github.com/morehardy/anki-forge/blob/main/docs/manual-validation/anki-desktop-v1/TEMPLATE.md)と、それぞれの参照検証で指定した正確な Anki のバージョン・設定で評価してください。クライアント側の条件とファイル単位の公開動作は、[更新](https://ankiforge.dev/ja/docs/updates/)と[ビルドの保証](https://ankiforge.dev/ja/docs/build-guarantees/)を参照してください。
