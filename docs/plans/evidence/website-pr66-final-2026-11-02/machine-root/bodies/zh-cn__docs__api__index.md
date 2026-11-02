# 公开 API 总览

Canonical page: https://ankiforge.dev/zh-cn/docs/api/
Language: zh-CN
Description: 查找 Rust、Node 和 Python 的内容编写、媒体、诊断与更新 API。

Anki Forge 是开源 Anki APKG 生成库，以 Rust 为核心，提供 Rust、Python 和 Node.js 接口。使用公开的 `ankiforge` 包编写笔记、打包媒体，并在分发新版牌组前检查更新。采用 MIT 许可证。

| 任务 | Rust | Python | Node / TypeScript |
| --- | --- | --- | --- |
| 编写笔记与模型 | [Rust API](https://ankiforge.dev/zh-cn/docs/rust-api/) | [Python API](https://ankiforge.dev/zh-cn/docs/python-api/) | [Node API](https://ankiforge.dev/zh-cn/docs/node-api/) |
| 图片、音频与遮挡 | [媒体](https://ankiforge.dev/zh-cn/docs/media/) | [拥有所有权的媒体](https://ankiforge.dev/zh-cn/docs/python-api/#media-and-occlusion) | [拥有所有权的媒体](https://ankiforge.dev/zh-cn/docs/node-api/#media-and-image-occlusion) |
| 诊断失败 | [排查问题](https://ankiforge.dev/zh-cn/docs/troubleshooting/) | [诊断](https://ankiforge.dev/zh-cn/docs/python-diagnostics/) | [错误](https://ankiforge.dev/zh-cn/docs/node-api/#errors) |
| 比较与发布 | [更新流程](https://ankiforge.dev/zh-cn/docs/updates/) | [构建与比较](https://ankiforge.dev/zh-cn/docs/python-api/) | [构建与比较](https://ankiforge.dev/zh-cn/docs/node-api/#build-compare-and-output) |

## 稳定笔记身份

为每条逻辑笔记保留稳定的项目命名空间和笔记键。更新已有版本还需要原始分发 APKG。Anki 重新导出的文件不能替代原包内的完整标识映射与修订信息。

## 从一个任务开始

[选择开发语言](https://ankiforge.dev/zh-cn/docs/languages/)，或完成[从 JSON 生成并更新牌组的示例](https://ankiforge.dev/zh-cn/docs/json-to-anki/)。公开运行环境见[兼容性](https://ankiforge.dev/zh-cn/docs/compatibility/)，文件所有权见[输出保证](https://ankiforge.dev/zh-cn/docs/build-guarantees/)。
