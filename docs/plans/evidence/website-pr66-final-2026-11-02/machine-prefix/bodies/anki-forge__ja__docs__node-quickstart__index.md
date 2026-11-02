# Node.js で Anki カードを生成する

Canonical page: https://morehardy.github.io/anki-forge/ja/docs/node-quickstart/
Language: ja
Description: 公開 Node パッケージをインストールし、完全な JavaScript プログラムで最初の永続 APKG をエクスポートします。

Node.js 22.13 以降を使います。新しいアプリケーションのディレクトリで実行します。

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.1
```

[npm の公開パッケージ](https://www.npmjs.com/package/ankiforge)と、環境に一致するネイティブのオプション依存パッケージをインストールします。対応プラットフォームは[検証済みの環境](https://morehardy.github.io/anki-forge/ja/docs/compatibility/)を参照してください。

## デッキをエクスポートする

次を `main.mjs` に保存します。

```js
import { Project, Note, BuildOptions } from 'ankiforge';

const project = new Project('spanish').defaultDeck('Spanish');
project.add('es:hola', Note.basic('hola', 'hello'));
const output = await project.build(BuildOptions.to('spanish.apkg'));
console.log(output.artifact.path);
await output.artifact.close();
```

```sh
node main.mjs
```

永続ファイル `spanish.apkg` を Anki で開きます。**Spanish** デッキに **hola → hello** が含まれています。後で編集する際も、名前空間 `spanish` とキー `es:hola` を維持してください。永続成果物を閉じてもファイルはディスクに残ります。エクスポートに Anki のインストールは必要ありません。

## TypeScript と次の作業

[JSON から Anki を生成するレシピ](https://morehardy.github.io/anki-forge/ja/docs/json-to-anki/#node)には、TypeScript のコンパイルから実行までの完全な手順があります。

ESM には `.mjs`、CommonJS には `require('ankiforge')` を使います。パッケージには TypeScript の型宣言が同梱されています。[TypeScript の設定](https://morehardy.github.io/anki-forge/ja/docs/node-api/#typescript)を参照してください。

-   [画像と音声](https://morehardy.github.io/anki-forge/ja/docs/media/)と [Node のメディア API](https://morehardy.github.io/anki-forge/ja/docs/node-api/#media-and-image-occlusion)
-   [独自のノートタイプ](https://morehardy.github.io/anki-forge/ja/docs/custom-notetypes/)と[テンプレートバンドル](https://morehardy.github.io/anki-forge/ja/docs/templates/)
-   [画像穴埋め](https://morehardy.github.io/anki-forge/ja/docs/image-occlusion/)
-   [比較と更新](https://morehardy.github.io/anki-forge/ja/docs/updates/)
-   [Node API](https://morehardy.github.io/anki-forge/ja/docs/node-api/) と[トラブルシューティング](https://morehardy.github.io/anki-forge/ja/docs/troubleshooting/)

ソースからのビルドとローカルパッケージの作成は、[ソースからのビルド](https://morehardy.github.io/anki-forge/ja/docs/development/)を参照してください。
