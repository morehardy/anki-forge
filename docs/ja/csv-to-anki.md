# Python で CSV を Anki カードに変換する

Python と Anki Forge で、語彙の表を Anki の `.apkg` ファイルに変換します。この例は `id`、`front`、`back` 列を持つ UTF-8 の CSV を読み取り、Basic の質問・解答カードを作り、共有できるデッキをエクスポートします。表の読み取りには Python 標準ライブラリの CSV パーサーを使い、パッケージの生成は Anki Forge が担当します。エクスポートに Anki のインストールは必要ありません。

一度だけ表をインポートしたい場合、Anki 自体が[区切り文字付きのテキストファイル](https://docs.ankiweb.net/manual/importing/text-files)に対応しています。このレシピは、繰り返し使えるスクリプト、独自のノート ID、配布用 APKG が必要な場合に使います。Excel の `.xlsx` ファイルは、先に UTF-8 の CSV として保存してください。

<!-- section: prepare-csv -->
## CSV を用意する

次を `cards.csv` として保存するか、[サンプルをダウンロード](/generated/recipes/csv-to-anki/cards.csv)します。

<!-- source: docs/recipes/csv-to-anki/cards.csv -->
```csv
id,front,back
es:hola,hola,hello
es:gracias,gracias,thank you
es:adios,adiós,goodbye
```
<!-- /source -->

ヘッダーはこのとおりに保ち、区切りにはカンマを使います。カンマ、二重引用符、改行を含むフィールドは、[CSV の引用規則](https://docs.python.org/3/library/csv.html)に従って引用符で囲んでください。表計算ソフトの出力に含まれることがある UTF-8 のバイトオーダーマークも受け付けます。ID の重複、値の欠落、余分な列、空の入力は拒否します。

ID は一度割り当てたら、質問や解答を変更しても維持します。行の位置や編集可能なカード本文から ID を作らないでください。関係のないデータセットには別の名前空間を使い、このデッキの後続バージョンでは同じ名前空間を保持します。

<!-- section: install -->
## Anki Forge をインストールする

[対応プラットフォーム](/docs/compatibility/)で通常の CPython 3.11 以降を使用します。仮想環境を作成して有効にします。

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

Windows では `.venv\Scripts\Activate.ps1` で有効にします。この変換に表計算ライブラリや AI サービスは必要ありません。

<!-- section: convert -->
## APKG を生成する

次のプログラムを `cards.csv` と同じ場所に `csv_to_anki.py` として保存するか、[スクリプトをダウンロード](/generated/recipes/csv-to-anki/csv_to_anki.py)します。

<!-- source: bindings/python/examples/csv_to_anki.py -->
```python
"""Convert a UTF-8 id,front,back CSV into a persistent Basic-card APKG."""
import argparse
import csv
from pathlib import Path

from ankiforge import BuildOptions, Note, Project


def build_csv(source: Path, destination: Path, namespace: str, deck: str) -> int:
    if destination.exists():
        raise ValueError('Choose a new output path; keep previously distributed APKGs')
    project = Project(namespace, default_deck=deck)
    seen = set()
    with source.open(encoding='utf-8-sig', newline='') as handle:
        rows = csv.DictReader(handle, strict=True)
        if rows.fieldnames != ['id', 'front', 'back']:
            raise ValueError('Expected the header id,front,back in that order')
        for row in rows:
            if None in row or any(value is None or not value.strip() for value in row.values()):
                raise ValueError(f'Line {rows.line_num}: expected three nonempty fields')
            key = row['id'].strip()
            if key in seen:
                raise ValueError(f'Line {rows.line_num}: duplicate id {key!r}')
            project.add(key, Note.basic(row['front'], row['back']))
            seen.add(key)
    if not seen:
        raise ValueError('The CSV must contain at least one note')
    project.build(BuildOptions.to(destination))
    return len(seen)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('destination', type=Path)
    parser.add_argument('--namespace', default='csv-vocabulary')
    parser.add_argument('--deck', default='Vocabulary')
    args = parser.parse_args()
    count = build_csv(args.source, args.destination, args.namespace, args.deck)
    print(f'{args.destination}: {count} notes, {count} Basic cards')
```
<!-- /source -->

そのディレクトリで実行します。

```sh
python csv_to_anki.py cards.csv vocabulary-v1.apkg --namespace spanish-vocabulary --deck Spanish
```

サンプルは **Spanish** デッキに**ノート3件、Basic カード3枚**を持つ `vocabulary-v1.apkg` を生成します。メディアは含みません。出力は次のとおりです。

```text
vocabulary-v1.apkg: 3 notes, 3 Basic cards
```

まだ存在しない出力パスを選んでください。配布時のオリジナルパッケージを保存できるよう、スクリプトは既存ファイルの置き換えを拒否します。

<!-- section: verify-and-extend -->
## インポート、確認、拡張

`vocabulary-v1.apkg` を Anki で開きます。デッキに `hola → hello`、`gracias → thank you`、`adiós → goodbye` があることを確認し、カードをプレビューしてアクセント記号も確認します。配布前に内容をチェックしてください。特に AI が CSV の下書きを作成した場合は、解答の確認が必要です。

このレシピはテキストだけの Basic カードを作成します。他の作業には次のガイドがあります。

- [AI を使った Anki カード作成](/docs/ai-anki-cards/): 学習メモから CSV 行を下書きし、内容を確認します。
- [JSON から Anki を生成する](/docs/json-to-anki/): Python、Node / TypeScript、Rust で画像と音声を同梱し、次のリリースを確認します。
- [Basic と Cloze のカード](/docs/cards/): 質問・解答カードまたは穴埋めカードを選びます。
- [生成した Anki デッキの更新](/docs/updates/): 前回のオリジナル APKG を更新の基準に使い、比較結果を確認します。

後続リリースでは、名前空間とノート ID を維持し、公開前に更新手順を実行してください。安定 ID はノートの識別に役立ちますが、Anki のインポート設定や新しいローカル編集も更新結果に影響します。
