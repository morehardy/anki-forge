# 选择开发语言

Anki Forge 是开源的 Anki APKG 生成库，以 Rust 为核心，提供 Rust、Python 和 Node.js 接口。
三种语言都可以离线生成包含媒体的牌组，并在发布更新前检查变更；生成过程不需要安装 Anki。

| 开发语言 | 安装公开版本 0.3.1 | 完整入门程序 |
| --- | --- | --- |
| Rust | `cargo add ankiforge@0.3.1` | [用 Rust 生成第一个牌组](quickstart.md) |
| Python | `python -m pip install ankiforge==0.3.1` | [用 Python 生成第一个牌组](python-quickstart.md) |
| Node / TypeScript | `npm install --include=optional ankiforge@0.3.1` | [用 Node.js 生成第一个牌组](node-quickstart.md) |

Rust 需要 1.92 或更新版本；Node.js 需要 22.13 或更新版本及对应平台的原生可选依赖。
Python 使用普通 CPython 3.11 或更新版本和对应平台的 wheel，发布检查覆盖 3.11–3.14。
原生包支持 macOS arm64/x64、Linux x64 GNU/glibc 和 Windows x64。
具体测试范围见[兼容性说明（英文）](../compatibility.md)。

## 下一步

用[同一份 JSON 数据、图片和音频生成并更新牌组](json-to-anki.md)，
或先了解 [Anki Forge 与 genanki 的选择依据](anki-forge-vs-genanki.md)。
按方法查找时，使用 [API 参考（英文）](../api-overview.md)。
