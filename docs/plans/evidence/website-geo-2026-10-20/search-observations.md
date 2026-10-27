# 公开检索样本说明

日期：2026-10-20（America/Los_Angeles）。工具：本会话公开 web 搜索/读取工具，不是各 AI 产品的端到端推荐实验。没有可靠的引擎覆盖、地理位置或排名总体统计，因此不计算所谓全平台 AI 份额。

本轮查询包括 `"Anki Forge" ankiforge`、`site:ankiforge.dev`、`genanki alternative Python TypeScript Anki deck generation`、`"Anki Forge" library Python Rust TypeScript`、`"genanki alternative" library`、`"TypeScript" "generate Anki" library`、`"ankiforge.dev" -site:ankiforge.dev -site:github.com`、`"ankiforge" "Python Product API"` 及 npm 定向查询。检索工具可能扩大/改写查询；返回结果不是可靠的精确排名清单。

已观察到：

1. 本轮可检索到 [官网](https://ankiforge.dev/)、[Rust 安装页](https://ankiforge.dev/docs/installation/)、[兼容性页](https://ankiforge.dev/docs/compatibility/)、[Python 包页](https://pypi.org/project/ankiforge/) 等来源。这与 9 月旧报告的检索样本不同，不证明所有页面已进入 Google/Bing 索引。
2. 品牌查询返回 [ankiforge.com 的账户页面](https://ankiforge.com/account) 和 [forge.anki.so](https://forge.anki.so/)。前者展示账户/套餐/Deck Studio 等内容，后者展示软件开发服务。仅据公开页面确认同名/歧义，不判断站点所有权或法律关系。
3. 读取 [Rust quickstart](https://ankiforge.dev/docs/quickstart/) 的检索版本仍包含 `git clone` 和旧示例，结果标记抓取时间约两周前；本轮直接 curl 同 URL 则是 `cargo add ankiforge@0.3.1`、`es:hola`。抓取时间标签不是发布日期，缓存差异也不代表实时页面部署错误。
4. 直接抓取当前 [Rust 入门文章](https://ankiforge.dev/blog/your-first-anki-deck/) 确认：代码使用 `es:hola`，解释仍是 `hola`，正文仍指向 source-checkout command。这是现行内容差异，与上一条搜索缓存落后不同。
5. 非品牌查询中可见 genanki 及其他库、示例项目和讨论；本轮样本不足以判断 Anki Forge 的真实曝光率，也不证明它从不被推荐。

建议下一轮分别建立品牌事实准确性和非品牌选型问题集，在相同平台/模式/语言条件下重复采样并保存完整回答与引用；账户内索引和引用数据仍需另行核对。
