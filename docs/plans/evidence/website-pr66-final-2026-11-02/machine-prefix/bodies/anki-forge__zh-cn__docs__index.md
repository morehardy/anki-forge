# 用代码生成 Anki 牌组

Canonical page: https://morehardy.github.io/anki-forge/zh-cn/docs/
Language: zh-CN
Description: 选择开发语言，生成第一个牌组，再添加媒体、模板和可重复的更新。

Anki Forge 根据笔记、模板和媒体文件生成 Anki `.apkg` 文件。当学习内容保存在代码或结构化数据中，需要反复构建并分发牌组时，可以使用它。

## 从你的开发语言开始

安装公开包，导出第一个牌组；[选择入门方式](https://morehardy.github.io/anki-forge/zh-cn/docs/languages/)。

| 开发语言 | 第一个结果 | 继续学习 |
| --- | --- | --- |
| Rust | [运行首个牌组示例](https://morehardy.github.io/anki-forge/zh-cn/docs/quickstart/) | [集成到应用](https://morehardy.github.io/anki-forge/zh-cn/docs/installation/) |
| Node / TypeScript | [安装并构建第一个牌组](https://morehardy.github.io/anki-forge/zh-cn/docs/node-quickstart/) | [API 与异步行为](https://morehardy.github.io/anki-forge/zh-cn/docs/node-api/) |
| Python | [安装并导出牌组](https://morehardy.github.io/anki-forge/zh-cn/docs/python-quickstart/) | [API 与输出所有权](https://morehardy.github.io/anki-forge/zh-cn/docs/python-api/) |

想先看结果？[探索卡片示例](https://morehardy.github.io/anki-forge/zh-cn/examples/)，并下载生成的包。

## 理解各个部分

笔记保存字段；笔记类型及其模板把字段转换成卡片。牌组在 Anki 中组织生成的卡片。[核心概念](https://morehardy.github.io/anki-forge/zh-cn/docs/concepts/)说明这些关系、Project 的值所有权和稳定身份。

## 构建下一项功能

-   [CSV 转 Anki](https://morehardy.github.io/anki-forge/zh-cn/docs/csv-to-anki/)：用 Python 将词汇表批量生成 APKG。
-   [JSON 转 Anki](https://morehardy.github.io/anki-forge/zh-cn/docs/json-to-anki/)：用三种开发语言生成带媒体的卡片，并检查更新。
-   [AI 辅助制卡](https://morehardy.github.io/anki-forge/zh-cn/docs/ai-anki-cards/)：根据笔记起草卡片，核对答案后导出 APKG。
-   [Basic 与 Cloze](https://morehardy.github.io/anki-forge/zh-cn/docs/cards/)：创建问答和填空卡片。
-   [图片与音频](https://morehardy.github.io/anki-forge/zh-cn/docs/media/)：注册真实资源并附加到卡片。
-   [自定义笔记类型](https://morehardy.github.io/anki-forge/zh-cn/docs/custom-notetypes/)与[模板包](https://morehardy.github.io/anki-forge/zh-cn/docs/templates/)：定义字段、布局与 CSS。
-   [图片遮挡](https://morehardy.github.io/anki-forge/zh-cn/docs/image-occlusion/)：根据图像区域创建问题。
-   [更新与分发](https://morehardy.github.io/anki-forge/zh-cn/docs/updates/)：在版本之间传递身份和修订证据。

卡片类型指南使用可运行的 Rust 示例，CSV 教程使用 Python，JSON 教程覆盖三种开发语言。 Node 和 Python API 页面说明相应接口及其差异。

## 查找答案与参与贡献

操作失败时参考[排查问题](https://morehardy.github.io/anki-forge/zh-cn/docs/troubleshooting/)，运行环境支持和客户端导入条件见[已验证环境](https://morehardy.github.io/anki-forge/zh-cn/docs/compatibility/)。

Anki Forge 采用 [MIT](https://github.com/morehardy/anki-forge/blob/main/LICENSE) 许可证。可在 [GitHub](https://github.com/morehardy/anki-forge/issues) 提交可复现的问题，或阅读[贡献者指南](https://github.com/morehardy/anki-forge/blob/main/docs/development.md)。
