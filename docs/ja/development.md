# ソースから SDK をビルドする

このガイドはリポジトリでの開発と未リリースの変更を扱います。公開済みの `0.3.1` パッケージについては、[Rust](/docs/installation/)、[Node](/docs/node-quickstart/)、[Python](/docs/python-quickstart/) のインストールガイドを参照してください。

Git、Cargo を含む Rust 1.92.0、および対象 SDK に応じて npm を含む Node.js 22.13 以降、または CPython 3.11/3.12 が必要です。リポジトリでは Rust ツールチェーンを固定しています。必要に応じて `rustup toolchain install 1.92.0` でインストールしてください。

<!-- section: source-builds -->
## ソースからのビルド

チェックアウトしたソースをビルドするには、次を実行します。

```sh
git clone https://github.com/morehardy/anki-forge.git
cd anki-forge
```

Rust アプリケーションのマニフェストでは `ankiforge = { path = "/absolute/path/to/anki-forge/anki_forge" }` を指定できます。サンプルは `cargo run --locked -p ankiforge --example target_api_basic` で実行します。

このチェックアウトから Node アドオンをビルドします。

```sh
cd bindings/node
npm run setup
npm run build
npm run pack:local
```

`pack:local` が表示する実際の facade とホスト用ネイティブパッケージの両方の tarball を、別のアプリケーションに `npm install --offline --ignore-scripts --omit=optional /absolute/path/facade.tgz /absolute/path/native.tgz` でインストールします。これはソースからのビルドであり、レジストリからのインストールではありません。TypeScript の設定は [Node API](/docs/node-api/#typescript) を参照してください。

Python では CPython 3.11/3.12 の仮想環境を有効にし、Maturin をインストールしてから、リポジトリのルートで実行します。

```sh
python -m pip install maturin==1.15.0
maturin develop --manifest-path bindings/python/native/Cargo.toml --locked
```

通常のインストール済み wheel は実行時に Cargo を必要としません。ソースのラッパーと、レジストリにある古いネイティブバイナリーを混在させないでください。
