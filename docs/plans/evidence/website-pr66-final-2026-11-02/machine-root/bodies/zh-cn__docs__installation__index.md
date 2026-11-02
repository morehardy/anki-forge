# 集成到 Rust 应用

Canonical page: https://ankiforge.dev/zh-cn/docs/installation/
Language: zh-CN
Description: 安装公开 Rust 包，配置 Cargo 依赖并选择下一步指南。

使用 Rust 1.92 或更高版本。在现有 Cargo 应用中安装公开 crate：

```sh
cargo add ankiforge@0.3.1
```

普通 APKG 导出不需要安装 Anki。完整的第一个程序和 `cargo run` 操作见 [Rust 入门](https://ankiforge.dev/zh-cn/docs/quickstart/)。

## Cargo 依赖

公开的 `ankiforge` crate 包含 Basic、Cloze、自定义笔记类型、拥有所有权的媒体、图片遮挡、比较与预备发布。默认 feature 为空；普通应用代码不需要仓库专用 feature。

这是原生 Rust 库。请按 Rust 工具链要求安装平台的链接器和构建工具。选择部署主机时参考[已验证环境](https://ankiforge.dev/zh-cn/docs/compatibility/)。Rust 1.92 是最低要求；列出某个平台并不代表保证支持所有目标或第三方工具链配置。

需要 JSON 输入时，显式添加调用方解析依赖：

```sh
cargo add serde --features derive
cargo add serde_json
```

[JSON 到 APKG 示例](https://ankiforge.dev/zh-cn/docs/json-to-anki/#rust) 包含完整解析器、媒体与更新流程。SDK 不提供通用 JSON 或 CSV 转换器。

## 选择下一篇指南

-   [Rust 编写指南](https://ankiforge.dev/zh-cn/docs/rust-guide/)：Project 与模型设计
-   [Rust API](https://ankiforge.dev/zh-cn/docs/rust-api/)：方法、报告与错误
-   [媒体](https://ankiforge.dev/zh-cn/docs/media/)：拥有所有权的图片与音频
-   [更新与分发](https://ankiforge.dev/zh-cn/docs/updates/)：基线与发布条件
-   [源码构建](https://ankiforge.dev/zh-cn/docs/development/)：仓库开发
