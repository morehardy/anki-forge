# 兼容性

## 公开包与已验证环境

截至 2026-10-13，公开 Rust、npm 和 PyPI 包均为 `0.3.1`。从 [Rust](../installation.md)、[Node / TypeScript](../node/quick-start.md) 或 [Python](../python/quick-start.md) 开始。

| 接口 | 包 | 运行环境与验证 |
| --- | --- | --- |
| Rust | [ankiforge 0.3.1](https://crates.io/crates/ankiforge) | Rust 1.92.0 与当前稳定版；四个支持平台的打包消费者，以及全新的公开 crates.io 消费者 |
| Node / TypeScript | [ankiforge 0.3.1](https://www.npmjs.com/package/ankiforge) | Node 22.13.0、当前 22、24 和 26；16 项公开 npm 安装、ESM/CJS 和 TypeScript 检查 |
| Python | [ankiforge 0.3.1](https://pypi.org/project/ankiforge/) | 普通 CPython 3.11–3.14；16 项隔离的公开 PyPI wheel 安装检查 |

已发布的原生包覆盖 macOS arm64/x64、Linux x64 GNU/glibc 和 Windows x64。存在安装包并不等于在每台主机上都通过了运行测试。[Node 覆盖记录](../../bindings/node/COVERAGE.md)、[Python 覆盖记录](../../bindings/python/COVERAGE.md)和发布证据列出具体检查。成功的 `0.3.1` 发布运行分别为 [Rust](https://github.com/morehardy/anki-forge/actions/runs/38371221696)、[npm](https://github.com/morehardy/anki-forge/actions/runs/38371222114) 和 [PyPI](https://github.com/morehardy/anki-forge/actions/runs/38371222047)。其他 Python 实现、自由线程解释器与子解释器需要单独验证。浏览器、Electron、Bun、Alpine/musl 以及 ARM64 Linux/Windows 不在现有 Node 验证范围内。

[批量媒体](../media.md#batch-import-from-source)和[预备发布](../updates.md#build-once-review-then-publish-from-source)指南中的能力已包含在公开 `0.3.1` 包中。

## Anki 导入与播放

普通导出不需要安装 Anki。导入、渲染和播放媒体仍取决于目标客户端版本、编解码器和导入设置。2026-10-08 的基准测试通过了 40 项 Anki 导入、内容和代表性渲染检查（每个实现 20 项），并未覆盖所有客户端或 GUI/音频行为。

更新时保留之前的原始分发 APKG。Anki 重新导出的文件不保留所需身份证据。包中不再包含某条笔记，并不会将其从学习者的集合中删除。字段或模板变化、排序字段、本地编辑和复习历史需要针对客户端测试。

使用[手动客户端场景](../manual-validation/anki-desktop-v1/TEMPLATE.md)及各 oracle 记录的具体 Anki 版本和设置评估自己的分发。客户端条件与单文件发布行为见[更新](../updates.md)和[构建保证](../build-guarantees.md)。
