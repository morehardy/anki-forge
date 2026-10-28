# Anki 搜索场景覆盖：实施与验收

调研开始：2026-10-27；最终验收：2026-10-28（America/Los_Angeles）。关键词证据、优先级和量化方法见
[搜索需求调研](2026-10-27-anki-search-demand-research.md)。本次修改在现有网站和多语言工作之上增量完成；没有发布网站或修改外部平台信息。

## 实施逻辑

`anki` 的可见搜索需求很大，但包含下载、登录与复习软件等意图。本项目定位是开源
APKG 生成库，优先承接能真正完成的任务：数据批量制卡、Python/Node/Rust 集成、
AI 草稿导出、题型与媒体、牌组更新。场景词尚无可比月量，实施优先级依据能力匹配
和可验证内容，不是把推测包装成“高搜索量排名”。

| 搜索场景 | 本次处理 | 落地页 |
| --- | --- | --- |
| Anki card / flashcard generator；Anki 卡片生成工具 | 首页标题、主标题、摘要直接说明制卡用途和开发者定位 | `/`、`/zh-cn/` |
| CSV / Excel to Anki；Python 批量制卡 | 新增完整双语教程、可下载 CSV 和 Python 程序；明确 Excel 先导出 CSV、普通表格可由 Anki 直接导入 | `/docs/csv-to-anki/` 及中文对应页 |
| AI 生成 Anki 卡片；notes to Anki | 新增材料→提示词→人工核对→CSV/JSON→APKG 流程，连接现有编程助手入口 | `/docs/ai-anki-cards/` 及中文对应页 |
| Basic / Cloze；图片遮挡；图片音频；模板；更新 | 保留既有路径，把五个功能页标题改为明确的 Anki 任务描述 | `cards`、`image-occlusion`、`media`、`templates`、`updates` |
| 从首页进入具体任务 | 首页任务链接、文档目录、文档入口和 Python 入门页增加真实正文内链 | 按现有注册表生成双语链接 |

新页面接入既有页面注册表，复用 canonical、hreflang、站点地图、结构化数据、
Markdown 与机器目录的生成流程。站内词表只用于本地查找，不作为 Google 排名配置。
没有新增关键词 meta、虚构评分或批量同义词页面。

AI 页面说明助手负责起草内容或编写程序，Anki Forge 负责构建卡片包；PDF 提取、
OCR 与图片自动识别需要单独工具。CSV 程序验证三列、非空值、唯一 ID 和输出路径，
不会覆盖旧分发包。稳定 ID 与完整更新流程的关系沿用原有兼容性条件。

视觉检查发现文档侧栏当前页的文字被共享链接颜色覆盖；添加一条作用于当前页链接的
CSS，恢复 Starlight 的对比色，让读者看清当前任务。
已有 `workflow-heading` 锚点的章节名保持原样，新增任务链接放在原章节内。
同时修正网站验收器对语言切换控件的计数：Starlight 的页头与移动菜单可以各有一套控件，
验收器检查目标语言覆盖，并逐个检查每个控件的页面与 `hreflang`；新增了移动端链接错误的回归测试。
子路径验收又发现页脚外链被错误地套用站内 base 规则；现在外链保留完整地址作比较，
站内链接单独移除 base 和语言前缀。新增“外链域名被改写”的真实产物回归测试，
修复前失败、修复后通过，避免误把不同主机上的同一路径当作相同链接。

浏览器回归还发现原生搜索输入框在非空时会消费第一次 Esc、只清空文本而不关闭弹窗。
最小复现覆盖英文/中文、空/非空查询和三种原焦点，确认后在现有键盘处理器显式关闭
弹窗。既有两条失败测试恢复通过，焦点回到原控件，查询文本保留。

## 可复现验证

- Python：`python bindings/python/tests/test_csv_recipe.py`，在隔离环境安装公开 PyPI
  `ankiforge==0.3.1` 后运行。覆盖真实命令、BOM/中文/引号/换行、行序与身份、9 类坏输入、
  已有文件保护，共 5 项测试通过；已接入网站的 Python 示例 CI 步骤。
- 独立产物检查：解压真实 APKG 的 `collection.anki21b`，查询 SQLite 的字段和数量，
  确认 3 条笔记、3 张卡片、0 媒体，内容与词汇表一致。
  [公开包与产物证据](evidence/anki-search-demand-2026-10-27/csv-public-package-validation.json)
- 网站单元测试：117 项通过；根路径构建产物门禁 15 项通过，包含新增的外链域名回归测试；Astro 检查 118 个文件，0 错误、0 警告。
- 正式构建验收：2 个语言 × 3 个页面 × 3 个宽度（320、390、1440），18 组无横向溢出；
  6 个独立设计的查询在 Pagefind 和实际搜索界面均通过。它们是验收样本，不是真实用户查询或外部排名。
  [查询定义](evidence/anki-search-demand-2026-10-27/discovery-queries.json) ·
  [正式构建结果](evidence/anki-search-demand-2026-10-27/release-discovery-results.json)
- 侧栏当前页链接在浅色与深色下的对比度分别约 6.13:1、8.88:1。
- 完整浏览器回归 16 项通过；额外 12 组真实键盘操作验证 Esc 关闭与焦点恢复。
  [修复前观察](evidence/anki-search-demand-2026-10-27/pre-final-regression-observations.json) ·
  [修复后复验](evidence/anki-search-demand-2026-10-27/search-escape-regression.json)
- 全站站内检索达到既定门槛：英文正向 25/25、中文正向 24/25，两语言核心查询均
  12/12、边界查询均 5/5。仍有 `示意图 遮名称 复习题` 未在前三命中图片遮挡页，保留为
  非阻断的检索改进项；没有修改独立查询或通过阈值来隐藏失败。
  [完整查询结果](evidence/anki-search-demand-2026-10-27/full-search-results.json)

源码技术与语言两轴独立审校已完成并按真实报告登记；历史及增量审校保留 24 条 minor，
无未解决的 major/critical。审校记录只证明实际审阅过对应源码，不代替运行与发布验收。
[最终审校与闭包旁证](evidence/anki-search-demand-2026-10-27/final-input-closure-delta-audit.json)

验证使用 `/tmp/anki-seo-validated-20261028` 的隔离副本，避免另一个并行任务覆盖构建
目录。最终登记后，主工作区与该副本的全部构建输入哈希一致：
`bbab11014509239802abb7cdaba37264bde12304210a47b832940527542c7756`。
主工作区 `check:content` 同样通过。
[源码一致性证据](evidence/anki-search-demand-2026-10-27/workspace-source-comparison.json)

根路径正式构建检查 69 个 HTML 页面（含 404）、68 页结构化数据、62 份 Markdown
和 2 个 AI 索引，元信息、内部链接/锚点、站点地图、下载及历史公开路径保护通过。
`https://morehardy.github.io/anki-forge/` 子路径配置也完成独立构建，相同的页面检查、
历史路径保护和 15 项产物门禁通过；两种构建具有相同的源码输入哈希。
站内排序器本身包含并行任务的修改；本轮验收覆盖整合结果，SEO 改动不等同于排序器改造。
[最终验收汇总及清单哈希](evidence/anki-search-demand-2026-10-27/final-acceptance.json)

## 上线后的效果判断

发布后提交或确认 sitemap，并用 Search Console 检查新页是否被收录、Google 选择的
canonical 是否正确。分别记录英文与中文、国家、设备以及非品牌场景查询的展示、点击、
CTR、落地页后续动作。先建立发布前 28 天基线，索引稳定后与下一完整窗口比较。
可用的生成式搜索展示报告单独统计；站内查询通过、页面抓取成功和安装命令复制都不等于
被外部搜索推荐或成功采用。

本轮未取得本站 Search Console / Bing 账户数据，没有测出排名提升，也没有进行
Anki 客户端导入验收。实际索引、推荐和流量效果须以上线后的平台数据为准。
