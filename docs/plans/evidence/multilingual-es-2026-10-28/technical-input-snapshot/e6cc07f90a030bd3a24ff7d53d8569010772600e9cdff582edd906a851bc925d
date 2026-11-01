# 公开 API 总览

Anki Forge 是开源 Anki APKG 生成库，以 Rust 为核心，提供 Rust、Python 和 Node.js 接口。使用公开的 `ankiforge` 包编写笔记、打包媒体，并在分发新版牌组前检查更新。采用 MIT 许可证。

| 任务 | Rust | Python | Node / TypeScript |
| --- | --- | --- | --- |
| 编写笔记与模型 | [Rust API](../rust-api.md) | [Python API](../python/api.md) | [Node API](../node/api.md) |
| 图片、音频与遮挡 | [媒体](../media.md) | [拥有所有权的媒体](../python/api.md#media-and-occlusion) | [拥有所有权的媒体](../node/api.md#media-and-image-occlusion) |
| 诊断失败 | [排查问题](../troubleshooting.md) | [诊断](../python/diagnostics.md) | [错误](../node/api.md#errors) |
| 比较与发布 | [更新流程](../updates.md) | [构建与比较](../python/api.md) | [构建与比较](../node/api.md#build-compare-and-output) |

## 稳定笔记身份

为每条逻辑笔记保留稳定的项目命名空间和笔记键。更新已有版本还需要原始分发 APKG。Anki 重新导出的文件不能替代原包内的完整身份证据。

## 从一个任务开始

[选择开发语言](https://ankiforge.dev/docs/languages/)，或完成[从 JSON 生成并更新牌组的示例](../json-to-anki.md)。公开运行环境见[兼容性](../compatibility.md)，文件所有权见[输出保证](../build-guarantees.md)。
