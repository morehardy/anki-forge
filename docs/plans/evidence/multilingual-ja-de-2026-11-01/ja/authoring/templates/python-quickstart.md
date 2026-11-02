# Python で Anki カードを生成する

仮想環境を作成して有効にし、[PyPI の公開パッケージ](https://pypi.org/project/ankiforge/)をインストールします。

@@CODE_0@@

Windows では `.venv\Scripts\Activate.ps1` で有効にします。通常の CPython 3.11 以降と、プラットフォームに一致する wheel を使用してください。記録済みのリリース検証では CPython 3.11〜3.14 を確認しています。[検証済みの環境](/docs/compatibility/)を参照してください。

<!-- section: export-a-deck -->
## デッキをエクスポートする

次を `main.py` に保存します。

<!-- source: bindings/python/examples/quickstart.py -->
@@CODE_1@@
<!-- /source -->

@@CODE_2@@

永続ファイル `spanish.apkg` を Anki で開きます。**Spanish** デッキに **hola → hello** が含まれています。後で編集する際も、名前空間 `spanish` とノートキー `es:hola` を維持してください。エクスポートに Anki のインストールは必要ありません。

<!-- section: continue -->
## 次のステップ

語彙の表を使う場合は [CSV から Anki を生成する](/docs/csv-to-anki/)を参照してください。学習メモからカードの下書きを作る場合は、[AI を使ったカード作成](/docs/ai-anki-cards/)から始められます。

メディアと更新を含む一連の作業は、[JSON から Anki を生成する](/docs/json-to-anki/#python)で確認できます。

- [画像と音声](/docs/media/)と [Python のメディア API](/docs/python-api/#media-and-occlusion)
- [独自のノートタイプ](/docs/custom-notetypes/)と[テンプレートバンドル](/docs/templates/)
- [画像穴埋め](/docs/image-occlusion/)
- [比較と更新](/docs/updates/)
- [Python API](/docs/python-api/) と[診断](/docs/python-diagnostics/)
- [genanki からの移行](/docs/genanki-migration/)

ネイティブ拡張をソースからビルドするには、[ソースからのビルド](/docs/development/)を参照してください。
