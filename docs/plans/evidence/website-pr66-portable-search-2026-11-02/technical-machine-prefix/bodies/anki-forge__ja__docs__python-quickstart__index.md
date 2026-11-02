# Python で Anki カードを生成する

Canonical page: https://morehardy.github.io/anki-forge/ja/docs/python-quickstart/
Language: ja
Description: 仮想環境に公開 Python パッケージをインストールし、短い完全なプログラムで永続 APKG を作成します。

仮想環境を作成して有効にし、[PyPI の公開パッケージ](https://pypi.org/project/ankiforge/)をインストールします。

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

Windows では `.venv\Scripts\Activate.ps1` で有効にします。通常の CPython 3.11 以降と、プラットフォームに一致する wheel を使用してください。記録済みのリリース検証では CPython 3.11〜3.14 を確認しています。[検証済みの環境](https://morehardy.github.io/anki-forge/ja/docs/compatibility/)を参照してください。

## デッキをエクスポートする

次を `main.py` に保存します。

```python
from ankiforge import Project, Note, BuildOptions

project = Project('spanish', default_deck='Spanish')
project.add('es:hola', Note.basic('hola', 'hello'))
output = project.build(BuildOptions.to('spanish.apkg'))
print(output.artifact.path)
```

```sh
python main.py
```

永続ファイル `spanish.apkg` を Anki で開きます。**Spanish** デッキに **hola → hello** が含まれています。後で編集する際も、名前空間 `spanish` とノートキー `es:hola` を維持してください。エクスポートに Anki のインストールは必要ありません。

## 次のステップ

語彙の表を使う場合は [CSV から Anki を生成する](https://morehardy.github.io/anki-forge/ja/docs/csv-to-anki/)を参照してください。学習メモからカードの下書きを作る場合は、[AI を使ったカード作成](https://morehardy.github.io/anki-forge/ja/docs/ai-anki-cards/)から始められます。

メディアと更新を含む一連の作業は、[JSON から Anki を生成する](https://morehardy.github.io/anki-forge/ja/docs/json-to-anki/#python)で確認できます。

-   [画像と音声](https://morehardy.github.io/anki-forge/ja/docs/media/)と [Python のメディア API](https://morehardy.github.io/anki-forge/ja/docs/python-api/#media-and-occlusion)
-   [独自のノートタイプ](https://morehardy.github.io/anki-forge/ja/docs/custom-notetypes/)と[テンプレートバンドル](https://morehardy.github.io/anki-forge/ja/docs/templates/)
-   [画像穴埋め](https://morehardy.github.io/anki-forge/ja/docs/image-occlusion/)
-   [比較と更新](https://morehardy.github.io/anki-forge/ja/docs/updates/)
-   [Python API](https://morehardy.github.io/anki-forge/ja/docs/python-api/) と[診断](https://morehardy.github.io/anki-forge/ja/docs/python-diagnostics/)
-   [genanki からの移行](https://morehardy.github.io/anki-forge/ja/docs/genanki-migration/)

ネイティブ拡張をソースからビルドするには、[ソースからのビルド](https://morehardy.github.io/anki-forge/ja/docs/development/)を参照してください。
