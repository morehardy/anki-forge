# 从源码构建 SDK

本指南用于仓库开发和未发布的更改。已发布的 `0.3.1` 包请参考 [Rust](../installation.md)、[Node](../node/quick-start.md) 或 [Python](../python/quick-start.md) 安装指南。

要求：Git、带 Cargo 的 Rust 1.92.0，以及对应 SDK 所需的 Node.js 22.13+ 与 npm，或 CPython 3.11/3.12。仓库固定了 Rust 工具链；必要时通过 `rustup toolchain install 1.92.0` 安装。

## 源码构建

构建当前源码目录：

```sh
git clone https://github.com/morehardy/anki-forge.git
cd anki-forge
```

Rust 应用可在 manifest 中使用 `ankiforge = { path = "/absolute/path/to/anki-forge/anki_forge" }`。通过 `cargo run --locked -p ankiforge --example target_api_basic` 运行示例。

从当前源码构建 Node addon：

```sh
cd bindings/node
npm run setup
npm run build
npm run pack:local
```

将 `pack:local` 实际输出的 facade 与主机原生 tarball 一起安装到单独的应用中：`npm install --offline --ignore-scripts --omit=optional /absolute/path/facade.tgz /absolute/path/native.tgz`。这些是源码构建，并非 registry 安装。TypeScript 配置见 [Node API](../node/api.md#typescript)。

Python 请激活 CPython 3.11/3.12 虚拟环境、安装 Maturin，再从仓库根目录运行：

```sh
python -m pip install maturin==1.15.0
maturin develop --manifest-path bindings/python/native/Cargo.toml --locked
```

普通安装的 wheel 在运行时不需要 Cargo。不要把源码包装层与较旧的 registry 原生二进制混用。
