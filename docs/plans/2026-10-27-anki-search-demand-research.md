# Anki 搜索需求与项目发现路径调研

调研日期：2026-10-27（America/Los_Angeles）。对象：[Anki Forge](https://ankiforge.dev/) / `morehardy/anki-forge`。本文区分已取得的数据、公开搜索观察和实施判断；不把站内检索通过、抓取成功或竞品页面数量当成搜索量或排名证据。

**建议优先争取“用代码批量生成 Anki 卡片”及其具体输入、语言、题型、更新场景。** `anki` 的需求规模明显值得关注，但它包含寻找 Anki 本体、登录、下载和使用教程的意图。当前产品更能完整满足 `Python Anki`、`CSV to Anki with Python`、`JSON to Anki`、`genanki alternative`、`Anki 图片遮挡批量生成` 等问题。这里的优先级依据是任务适配度与现有可验证能力，不能解读成这些词已有最高搜索量。

## 1. 已取得的搜索量证据

本轮取得以下供应商直接公布的估算，足以说明大词的量级，但不足以给所有 Anki 词排序：

| 精确展示的关键词 | 平均月搜索量估算 | 市场 / 快照 | 对本站的意义 |
| --- | ---: | --- | --- |
| `anki` | 110,000 | 美国 / 2026-09 | 大范围品牌与产品需求；页面需清楚说明自己是生成库 |
| `ankiweb` | 27,100 | 美国 / 2026-09 | 导航意图明显，本站不能满足登录/同步入口需求 |
| `anki flashcard software` | 14,800 | 美国 / 2026-09 | 查找闪卡软件；可用准确产品定义建立关联，但不应冒充复习应用 |

来源：[Semrush 的 ankiweb.net 公开概览](https://www.semrush.com/website/ankiweb.net/overview/)，页面标注更新日期 2026-10-14，表格明确为 United States、Sep 2026。数据是该供应商的关键词估算，不是 AnkiWeb 官方流量、Google 官方原始计数、全球搜索量或 Anki Forge 的预期访问量。该表只是其公开展示的少量词，不是完整关键词库。

Semrush 将月搜索量定义为过去 12 个月搜索次数的月均估算，地区数据库和更新月份会影响数值；9 月快照并不表示“9 月恰好发生这么多次搜索”。`anki flashcards` 与表中的 `anki flashcard software` 也是不同字符串，不能把同一个数抄给前者。[Semrush 搜索量口径](https://www.semrush.com/kb/683-what-is-search-volume-in-semrush)

**尚未取得可比月量的词：** `anki flashcards`、`anki card generator`、`ai anki generator`、`pdf to anki`、`csv to anki`、`json to anki`、`genanki`、`anki image occlusion`，以及中文“Anki 制卡”“Anki 批量生成卡片”“AI 生成 Anki 卡片”等。应记录为“待测”，不能填成 0，也不能按搜索结果数排列“热度”。本轮没有访问本项目的 Search Console 或付费关键词账户。

尝试了 Google Trends 的全球与美国、近 12 个月、五词同组比较页面，但研究工具未能读取图表数据，因此未生成 Trends 曲线、指数或词间倍数。Trends 的 0–100 是按地区和时间归一化的相对兴趣，低量词可能显示 0；自动补全也不是完整的热门词榜。[Google Trends 数据说明](https://support.google.com/trends/answer/4365533?hl=en)

## 2. 搜索意图观察：用户期待什么

以下是 2026-10-27 公开网页检索样本，采用服务提供者自己发布的页面或 Anki 官方手册作为证据。检索工具未提供固定地区、个性化状态和完整 Google 排名，因此只记录意图与页面类型，不记录“Google 第几名”。竞品的功能文案仅证明它们如何介绍自己，不代表本轮独立验证过功能。

| 查询组 | 可观察的第一手页面 | 可以得出的结论 |
| --- | --- | --- |
| `anki`、`anki flashcards` | [Anki 官网](https://apps.ankiweb.net/)展示复习软件、下载、AnkiWeb、共享牌组等入口 | 大词有广泛产品意图；本站应明确生成 APKG 后交给 Anki 学习的分工 |
| `anki card generator`、`ai anki generator` | [Ankify 生成器页](https://www.ankify.app/anki-card-generator)、[Memsticks](https://www.anki.memsticks.com/)以材料输入、编辑和牌组导出为主 | 无限定的 generator 容易引来期待网页直接生成结果的用户；本站标题宜带 `library` / `for developers` / “用代码” |
| `Anki 自动生成卡片` | [AnkiClip 中文页](https://ankiclip.com/zh)描述选文字或图像后生成卡片；[AnkiForge NG 插件](https://ankiweb.net/shared/info/597444116)描述 AI 生成 | 中文“自动制卡”也包含终端工具与插件意图；存在同名产品，不能只靠 `AnkiForge` 区分身份 |
| `csv to anki` | [Anki 文本导入手册](https://docs.ankiweb.net/importing/text-files.html)、[Neurako CSV 工具](https://tools.neurako.com/csv-to-anki)分别承接原生导入与网页转换 | 应先告诉读者 Anki 可直接导入 CSV；教程的独特价值在可重复的程序化 APKG 输出 |
| `python anki generate`、`genanki` | [genanki 原仓库](https://github.com/kerrickstaley/genanki)直接提供 Python 生成牌组的概念和示例 | Python 库选型是清晰匹配场景，比较页要回答 API、媒体、平台与身份更新取舍 |
| `anki image occlusion` | [Anki 添加/编辑手册](https://docs.ankiweb.net/editing.html#image-occlusion)解释原生图片遮挡 | 本项目应突出“程序化/批量生成”，避免与普通 Anki 图形界面操作教程混同 |

中文“卡片生成”还会命中 Anki 的模板生成规则，而不一定是 AI 自动生成问答。页面需要把“笔记变成卡片”“原始材料生成内容”“数据打包为 APKG”讲清楚。本站的直接能力属于后两者中的结构化数据打包与卡片编写部分，AI 内容提取由调用方完成。

## 3. 关键词簇与实施优先级

P1 表示当前最值得完整回答的任务；P2 表示相邻场景，需有足够独特内容再扩展。下表除第 1 节列出的精确词外，**所有词的月搜索量均待测**。同簇词应由一篇完整内容自然覆盖，避免每个变体建立近似页面。

| 优先级 | 英文候选词 | 中文候选词 | 用户任务 / 产品适配 | 页面职责 |
| --- | --- | --- | --- | --- |
| P1 | anki flashcard generator library; anki deck generator; apkg generator | Anki 卡片生成工具；Anki 制卡库；APKG 生成 | 了解能否嵌入自己的数据流程；直接适配 | 首页：清晰定义、三语言入口、真实例子 |
| P1 | python anki; generate anki cards python; python flashcard generator | Python 生成 Anki 卡片；Python 批量制卡 | 写一个可运行脚本；直接适配 | Python 快速开始；下一步进入 CSV/JSON |
| P1 | csv to anki; csv to apkg; excel to anki python | CSV 转 Anki；Excel 批量制卡；表格转 APKG | 表格资料转可导入牌组；需调用方解析 CSV | 新增 CSV 教程，明确 Excel 先导出 UTF-8 CSV |
| P1 | json to anki; json to apkg; TypeScript anki generator | JSON 转 Anki；Node.js 生成 Anki；结构化数据制卡 | 应用数据批量构建；直接适配 | 复用已有 JSON 完整教程与 Node 快速开始 |
| P1 | genanki alternative; anki forge vs genanki | genanki 替代；Anki 制卡库对比 | 选择生成库；直接适配 | 复用比较页，提供真实迁移边界 |
| P1 | AI generated flashcards to Anki; ChatGPT to Anki; AI Anki workflow | AI 生成 Anki 卡片；AI 辅助制卡；聊天记录转 Anki | AI 输出整理成可验证牌组；工作流部分适配 | 新增 AI 辅助流程，材料→核对→CSV/JSON→APKG |
| P1 | anki cloze generator; generate cloze cards | Anki 挖空卡片；Cloze 填空；批量生成问答卡 | 创建具体题型；直接适配 | 现有 cards 页明确 Basic、Cloze、笔记与卡片数量 |
| P1 | anki image occlusion programmatically; batch image occlusion | Anki 图片遮挡；图片挖空；批量遮挡制卡 | 从已知图片与坐标生成题目；直接适配 | 现有 image-occlusion 页，说明矩形及支持模式 |
| P1 | update generated anki decks; stable note ids anki | 更新 Anki 牌组；稳定笔记 ID；避免重复卡片 | 分发后的内容维护；直接适配但受客户端导入条件约束 | 现有 updates 页，原始 APKG 基线与验证实例 |
| P2 | anki audio flashcards generator; vocabulary to anki | Anki 音频卡片；单词表制卡；带图片的卡片 | 包含已有媒体的词汇/题库制卡；直接适配打包 | 强化 media 页与已有示例，区分音频打包和语音合成 |
| P2 | Rust anki library; Rust apkg | Rust Anki 生成库；Rust 制卡 | Rust 应用集成；直接适配 | 保留现有 Rust 快速开始作为唯一主入口 |
| 暂不承接独立工具页 | pdf to anki; image to anki; OCR anki; free online anki generator | PDF 一键制卡；图片识别制卡；在线免费 Anki 生成器 | 需要上传、OCR/解析或托管生成体验；当前产品不直接提供 | 只在 AI 工作流中解释前置整理步骤，不能宣传已有转换器 |
| 不作为获客目标 | anki download; ankiweb login; AnKing deck download; FSRS settings | Anki 下载；Anki 登录；现成牌组下载；复习参数 | 找官方应用、内容包或学习设置；本站不满足 | 必要时给官方入口，不建立蹭词页面 |

“搜索量 × 意图适配 × 内容质量 × 访问后成功率”可作为运营思路，但缺失数据时不应计算伪精确机会分数。现阶段直接按“是否解决真实任务、是否可验证、是否已有内容可复用”排优先级。

## 4. 对现有项目的具体处理建议

已有 [10 月 20 日 SEO 审计](2026-10-20-website-seo-audit.md)、[10 月 25 日 GEO 方案](2026-10-25-geo-optimization-plan.md)以及正在进行的多语言与机器可读改造。本轮不应把旧审计中的“尚无中文/结构化数据/JSON 教程”原样当成当前缺陷；应以当前源码与发布结果为准。

### 4.1 首页让人和搜索系统迅速识别用途

建议标题方向：`Anki flashcard generator for developers | Anki Forge`；中文可用 `Anki 卡片生成工具：用 Python、Node.js 或 Rust 制卡`，最终长度根据正文及显示效果调整。首段明确它是开源生成库、输出 `.apkg`、支持三语言接口，链接具体数据导入任务。

这属于表达产品已具备的能力，不是保证匹配某个词就能获得排名。Google 建议把用户会使用的词放在标题、主标题和描述性链接中；标题应具体、与内容一致，避免堆叠同义词。[Search Essentials](https://developers.google.com/search/docs/essentials)、[标题链接指南](https://developers.google.com/search/docs/appearance/title-link)

### 4.2 新增两条有独立价值的使用路径

| 内容 | 必须交付的实际信息 | 与既有内容的关系 |
| --- | --- | --- |
| CSV → Anki / APKG 教程，中英对应 | 可下载 UTF-8 CSV、完整 Python 程序、公开包安装、执行命令、输出计数、稳定业务 ID、重复/空字段处理；说明普通 CSV 可原生导入 Anki | 从 Python 快速开始、首页与 JSON 页链接；无需为 Excel/表格/CSV 各复制一页 |
| AI 辅助生成 Anki 卡片流程，中英对应 | 用户材料、约束提示词、人工核对示例、结构化 CSV/JSON 输出、连接真实构建程序；说明助手生成内容与本库打包的责任 | 复用现有 prompt builder、Skill、CSV/JSON 教程；没有 PDF 上传器时不制造上传入口 |

CSV 教程的“稳定 ID”应是源记录身份，不能用可变的答案文本、行号或每次重新生成的随机值冒充。首次构建教程无需暗示完成了完整更新验证；若宣传重导入不重复，应链接已有更新流程及兼容性边界。产品依据见 [JSON 教程](../json-to-anki.md)、[更新指南](../updates.md)。

AI 流程应能回答两类问题：“我只有学习材料，如何整理成卡片？”和“我已有数据，如何让编码助手写生成程序？”。现有网站 prompt builder 只在本地组合文字，不调用 AI 服务；源码已明确这一点，新增内容应保持一致。[现有 AI 助手说明](../ai-assistants.md)

### 4.3 把已有功能页与任务词连接

保留 URL，修改准确的标题、描述和可见引言，例如 `Create Basic and Cloze Anki cards`、`Add images and audio to Anki cards`、`Generate Anki Image Occlusion cards`、`Update generated Anki decks with stable note IDs`。中英文各自使用自然语言；“图片遮挡/图片挖空”“问答/填空”可在正文解释一次，不必做词串。

在首页、语言选择页及相关文档添加描述性内链，形成“选择任务→输入示例→运行→查看输出→理解更新条件”的路径。让 crawler 能沿普通 HTML 链接到达内容。现有 `search-terms.json` 服务站内 Pagefind 检索，它不等同于 Google 关键词排名配置；不能以扩充隐藏词表代替可见内容。[Google AI features 的内链与文本建议](https://developers.google.com/search/docs/appearance/ai-features)

### 4.4 技术与真实性要求

新页沿用项目现有页面注册、canonical、sitemap、结构化数据、英文/中文对应关系和机器内容机制。新增内容必须进入现有内容审校流程；未审校页面不能通过伪造审核记录发布。多语言关系应是对应页面之间的双向关系，每一语言页声明自己和其它版本，中文正文实际可读。[Google 多语言页面指南](https://developers.google.com/search/docs/specialty/international/localized-versions)

JSON-LD 的用途是表达可见事实与实体关系。机器文档可以帮助使用它们的 agent，但 Google 明确说明 `llms.txt` 等特殊文件不提高其搜索可见性；它也不要求为每种问法拆出一个页面。应把可运行范例、准确能力与具体边界当成可引用证据。[Google 生成式搜索优化指南](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide)

### 4.5 保持仓库、包站点与文档的身份一致

本地核对发现，README 已明确开源 APKG 生成库及三语言接口，Rust / Python / Node
包描述也已经包含 Anki APKG、媒体、模板和更新能力；Rust 包关键词包含 `anki`、
`flashcards`、`apkg`、`spaced-repetition`。这些是已有基础，本轮无需为了加词重复改写。

网站发布后，可把真实 CSV、JSON 与 AI 辅助教程作为仓库和各包文档的任务入口，
让从 GitHub、PyPI、npm、crates.io 进入的读者也能找到可运行范例。对外发布教程或
回应社区问题时，应提供输入、命令、输出与适用边界，并公开维护者身份。真实的采用
案例与独立引用可作为后续需求信号；不能以自建重复链接或付费伪装推荐替代它们。
本轮没有修改远端仓库资料、发布包或向社区发送消息。

## 5. 补齐搜索量与衡量改动的方法

### 5.1 关键词量化取样

在可访问的 Google Keyword Planner 中，导入第 3 节候选词，选择**历史指标**。固定为 Google 搜索网络、不含搜索合作伙伴，并记录明确地区与时间窗口，例如 2025-10 至 2026-09 的 12 个完整月份；美国、英国和其它计划覆盖的市场分别导出，不混加。中文词单独一组，并按真实目标市场取样；海外中文查询不能代表中国大陆全部搜索需求。

最少保存这些字段：`query`、`locale`、`country`、`network`、`period_start`、`period_end`、`avg_monthly_searches` 或原样区间、`source`、`retrieved_at`、`intent`、`target_page`。Google 的平均月量可能包含近似变体，广告竞争度表示广告主竞争，不是 SEO 排名难度；不能直接相加所有同义词估算可得流量。[Keyword Planner 指标定义](https://support.google.com/google-ads/answer/3022575?hl=en)

如果使用 Semrush/Ahrefs，统一供应商、地区、更新时间和窗口；保留“供应商估算”标签，避免把某工具的 US 数据与另一个工具的 Global 数据排序。Trends 只用于同一请求设置下比较趋势和季节性，采用 search term 或 topic 的规则也要一致；低量长尾不要据 0 删除。[Trends 口径](https://support.google.com/trends/answer/4365533?hl=en)

### 5.2 发布后的验证顺序

1. **发布验收：** 新页 HTTP 200，正文直接存在，canonical 正确，没有误加 `noindex`，sitemap/内链/hreflang 可达，示例程序产生文档承诺的 APKG。构建通过只证明交付完整，不证明已经收录。
2. **索引验收：** 在 Search Console URL Inspection 核对新增页、Google 选择的 canonical 和索引情况。记录实际发布与首次收录日期，再开始比较流量。
3. **需求验收：** 从 Search Console 导出查询、页面、国家、设备、展示、点击、CTR 和平均位置。按“生成工具 / CSV / JSON / AI 辅助 / Python / Node / genanki / 图片遮挡”分组，品牌词 `anki forge` / `ankiforge` 单列；不要把整个 `anki` 词排除为本站品牌词。[Search Console 性能报告](https://support.google.com/webmasters/answer/7576553?hl=en)
4. **推荐验收：** 查看 Search Console 的生成式 AI 表现报告；截至本次调研，官方说明该报告覆盖 AI Overviews 与 AI Mode 的展示，数据不足时可能不可见。它不是跨平台“被推荐率”，也不能把展示数视为安装数。[生成式 AI 表现报告](https://support.google.com/webmasters/answer/16984139)
5. **采用验收：** 观察相关落地页到示例下载、安装命令复制、仓库访问的后续动作；这些仍是使用意向。实际成功构建/导入的案例与用户反馈另行记录，继续复用仓库已有 [采用记录](geo-adoption-ledger.csv) 和 [测量协议](geo-measurement-protocol.md)。

建议建立发布前最近 28 天基线，索引稳定后比较下一个完整 28 天，并保留更长窗口观察低量词；国家、设备、查询组使用相同过滤条件。有展示但点击弱时先检查标题与意图，有点击但无后续动作时检查示例和运行门槛，无展示时先区分索引、需求与覆盖不足。该窗口是项目的观测设计，不是搜索引擎承诺的生效时间，也不提供单次改动的严格因果证明。

## 6. 本轮研究边界

- 取得了三项美国市场的公开供应商搜索量估算；没有取得完整词库、中文月量、本站后台查询量或 SEO 难度数据。
- 搜索结果样本能证明相关页面与用户任务存在，不能证明这些词按量排序，也不能预测 Anki Forge 的排名或推荐概率。
- 本文给出关键词与内容职责建议；代码是否完成、审校是否通过、页面是否发布，以本轮实施记录和实际验证结果为准。
- 不建议为了覆盖大词开发虚假的在线生成器、批量复制页面、伪造评分或制造外部推荐。项目现有公开包、程序、产物和更新证据就是最有价值的内容来源。
