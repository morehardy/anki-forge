# プログラミング言語を選ぶ

Rust、Node / TypeScript、Python を使って、手元のデータから Anki デッキを生成できます。いずれも Anki をインストールせずに永続ファイル `spanish.apkg` をエクスポートできます。

| 言語 | 公開パッケージ | インストール | 最初のデッキを生成する完全な例 |
| --- | --- | --- | --- |
| Rust | [ankiforge 0.3.1](https://crates.io/crates/ankiforge) | `cargo add ankiforge@0.3.1` | [Rust クイックスタート](/docs/quickstart/) |
| Node / TypeScript | [ankiforge 0.3.1](https://www.npmjs.com/package/ankiforge) | `npm install --include=optional ankiforge@0.3.1` | [Node クイックスタート](/docs/node-quickstart/) |
| Python | [ankiforge 0.3.1](https://pypi.org/project/ankiforge/) | `python -m pip install ankiforge==0.3.1` | [Python クイックスタート](/docs/python-quickstart/) |

Rust は 1.92 以降が必要です。Node は 22.13 以降と、環境に一致するネイティブのオプション依存パッケージが必要です。Python は通常の CPython インタープリター用ネイティブ wheel を使います。記録済みのリリース検証では CPython 3.11〜3.14 を確認しています。[検証済みの環境](/docs/compatibility/)を参照してください。

<!-- section: next-tasks -->
## 次の作業

[共通の JSON・画像・音声データ](/docs/json-to-anki/)からデッキを生成・更新するか、[Anki Forge と genanki](/docs/anki-forge-vs-genanki/)を比較して、自分の作業に合うライブラリを選べます。

[画像と音声](/docs/media/)を追加したり、[独自のノートタイプ](/docs/custom-notetypes/)を作成したり、[配布済みデッキの比較と更新](/docs/updates/)を行えます。

TypeScript を使う場合は[コンパイラーの設定](/docs/node-api/#typescript)を、genanki から移行する場合は[移行ガイド](/docs/genanki-migration/)を参照してください。リポジトリでの開発とソースでのみ使える機能については、[ソースからのビルド手順](/docs/development/#source-builds)を用意しています。
