# Anki Forge GEO：官方依据复核与实施边界

核对日期：2026-10-20（America/Los_Angeles）。对象：[Anki Forge 官网](https://ankiforge.dev/)。

本文件为项目及官网审计提供第一方依据；所有用于结论的来源均已打开核查。它不替代实际站点、源码、Search Console、Bing Webmaster Tools 或 CDN 日志审计。未访问本站站长账号，因此不能判断本站是否被索引、AI inclusion 设置、实际引用量或推荐效果。来源页面的发布日期、更新日期与本次访问日期分别记录，不将搜索摘要中的“最近抓取”当作发布日期。

证据标记：**A** = 官方公开规则或功能；**B** = 官方建议，本站收益未经实测；**C** = 针对本项目的实施建议或待验证假设。

## 1. 首先区分五种结果

“爬虫访问成功 → 搜索索引收录 → AI 引用 → 在选型答案中推荐 → 用户实际采用”应分别衡量。页面可访问不证明已收录，被引用也不证明被推荐。这是本次审计的分析框架，不是平台公布的排名公式。**C**。

Google 的现行指南明确：生成式搜索以原有搜索系统为基础；资格要求包括已索引、可显示摘要，以及未在 Search Console 排除 Search generative AI。满足条件仍不保证抓取、索引或展示。没有必须添加的特殊 AI schema；llms.txt 不影响 Google 搜索可见性或排名。**A**。[Google AI 优化指南](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide)

对 Anki Forge，合理目标是让搜索系统及 coding agent 能稳定找到已发布版本的正确文档、准确描述适用场景，并完成最小使用任务。不能把“添加文件/标签”作为已经提升推荐的证据。**C**。

## 2. 搜索抓取与模型训练应分别决策

| 平台 | 已核实的用途与控制 | 审计含义 |
| --- | --- | --- |
| OpenAI | `OAI-SearchBot` 用于 ChatGPT 搜索；`GPTBot` 用于可能参与模型训练的抓取。两者设置独立。`ChatGPT-User` 是用户触发访问，不决定搜索资格，robots 可能不适用。 | 希望出现在 ChatGPT 搜索答案，应允许搜索爬虫并核对官方 IP 能通过 WAF；开放训练不是搜索准入条件。 |
| Anthropic | `Claude-SearchBot` 面向搜索，`Claude-User` 面向用户请求，`ClaudeBot` 面向训练；官方表示这些 bots 遵守 robots.txt。 | 不应将所有 Claude 标识统一当成训练流量；实际检索入口与用户获取都需可用。 |
| Perplexity | `PerplexityBot` 用于搜索展示，不用于基础模型训练；官方建议 robots 与公开 IP 同时放行。 | 有 Allow 规则仍可能被 WAF 阻断，需结合访问日志验证。 |

上述均为 **A**；来源：[OpenAI Docs：Crawlers](https://developers.openai.com/api/docs/bots)、[Anthropic 爬虫说明](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler)（2026-04-07）、[Perplexity Crawlers](https://docs.perplexity.ai/docs/resources/perplexity-crawlers)。

项目验收建议：核对关键页面及其依赖资源、robots、HTTP/X-Robots-Tag、验证码、403/429；用官方 IP 信息和日志验证真实 bot。仅修改本机请求的 User-Agent 不能证明真实搜索爬虫获得相同响应。**C**。

## 3. Google 已可单独检查 AI inclusion 与展示

Search Console 的 Settings → Search generative AI 控制 AI Overviews、AI Mode 及 Discover 相关生成式功能的纳入。顶层属性默认包括，子属性可能继承父属性；该开关不是其他搜索结果的排名信号，也不控制训练。帮助页确认 2026-08-31 全球推出。**A**。[Google Search generative AI control](https://support.google.com/webmasters/answer/16908024)

独立 Generative AI performance report 已全球推出，主要统计 AI 展示次数，支持页面、国家、日期、设备等维度；没有足够展示时可能看不到报表。不能把它写成已经提供独立 AI 点击、转化或完整提示词报告。**A**。[Google AI 效果报表帮助](https://support.google.com/webmasters/answer/16984139)、[2026-06-03 发布公告及 08-31 更新](https://developers.google.com/search/blog/2026/06/gen-ai-performance-reports)

建议先记录核心 URL 的索引情况、Google 所选 canonical、AI inclusion 和过去 28 天 AI 展示，之后按发布批次比较。这些账号状态无法从官网 HTML 推断。**C**。

## 4. Bing / Copilot 衡量能力较上次审计有新增

2026-02-10 的 AI Performance 公告提供总引用、被引用页面、grounding queries 样本和时间趋势，覆盖 Microsoft Copilot、Bing AI 摘要及部分合作入口。它衡量引用，不代表排名、权威度或某页在单次回答中的位置。**A**。[Bing AI Performance 发布公告](https://blogs.bing.com/webmaster/2026/2/Introducing-AI-Performance-in-Bing-Webmaster-Tools-Public-Preview/)

**本次新增：2026-10-13 发布 Topics Portfolio beta。** 它按主题整理本站的 grounding 活动，显示大致主题规模、参与站点数量及机会查询。官方要求把数字视为方向性信号；站点或主题数据不足时功能可能不出现，机会查询还可能包含其他品牌词。**A**。[Bing Topics Portfolio 公告](https://blogs.bing.com/webmaster/2026/10/introducing-topics-portfolio-beta-in-ai-performance/)

项目建议：若账号已有足够数据，用实际被引用页和机会查询判断应该补哪些文档；数据不足时先用手工问题集建立基线。不要将 beta 的主题规模当作精确市场份额，也不要把所有竞品品牌词都扩写成页面。**C**。

## 5. 发现新内容：sitemap、语言映射与 IndexNow

Google 要求 sitemap 使用绝对 URL，并建议包含希望展示的规范 URL；`lastmod` 应对应实质更新，不能用每次构建时间伪装全站更新；Google 忽略 `priority` 和 `changefreq`。**A**。[Google Sitemap 指南](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap)

多语言页面应提供一致、互相返回且包含自身的 hreflang 映射；缺少返回链接可能被忽略。语言版本应对应同一主题，不能只把界面换成中文便当作内容已翻译。后一句是项目质量建议。**A/C**。[Google 多语言版本指南](https://developers.google.com/search/docs/specialty/international/localized-versions)

IndexNow 通知参与的搜索引擎新增、修改或删除 URL，补充 sitemap；提交成功不保证被索引，也不是保证提高排名的机制。**A**。[IndexNow FAQ](https://www.indexnow.org/faq)

项目建议：在正式部署成功后按真实变更提交文档 URL，保留提交响应；删除、重定向也纳入变更集。验收区分“通知被接收”“已抓取”“已索引”。现有配置是否缺失由源码审计确认，不能仅因源码没看到某个脚本就断言 CDN 未启用服务。**C**。

## 6. llms.txt 与 Markdown 的价值：降低使用文档的成本

llms.txt 当前公开版本是 v2，页面标注 2026-08-10 更新，仍自称提案。它建议用精简背景与链接引导 agent 按需读取 Markdown；支持根目录或子路径，并建议 `alternate` / `describedby` 链接关系。提案本身不证明搜索平台会用它排序。**A**。[llms.txt 提案](https://llmstxt.org/)

Chrome Lighthouse 的 llms.txt 检查明确：文件是可选的，404 会记为 N/A。这是 agentic browsing 可用性检查，不能据此声称成为 Google 搜索准入门槛。**A**。[Lighthouse llms.txt 文档](https://developer.chrome.com/docs/lighthouse/agentic-browsing/llms-txt)

项目建议：优先维护同源自动生成的精简入口，指向安装、入门、API、限制、版本及完整示例；链接目标须真实存在且内容及时。不应为覆盖关键词制造大文件。以“agent 能用当前发布包运行示例、正确说明限制”为验收，优先级低于代码与文档一致性。**C**。

## 7. Schema 与 FAQ：不能用更多标签替代可信证据

结构化数据必须反映用户可见事实，即使通过 Rich Results Test 也不保证富结果。**A**。[Google 结构化数据通用规则](https://developers.google.com/search/docs/appearance/structured-data/sd-policies)

Google 的 `SoftwareApplication` 富结果要求名称、价格，以及 `aggregateRating` 或 `review`。项目没有真实评分时，不应编造评分补齐检测器。用 schema 描述实体与取得富结果资格是两件事。**A/C**。[Google SoftwareApplication 规则](https://developers.google.com/search/docs/appearance/structured-data/software-app)

FAQ 富结果已自 2026-05-07 停止，相关文档于 06-15 移除；因此不能把“添加 FAQPage 获得 Google FAQ 富结果”列为本轮收益。FAQ 正文可继续服务真实问题。**A/C**。[Google Search 更新记录](https://developers.google.com/search/updates)

## 8. 对开发者工具更值得做的内容工作

Bing 建议清晰标题、表格、FAQ、示例与来源，保持内容准确并统一实体描述；这些是质量建议，不能解释为某种排版保证获选。**B**。[Bing 内容改进建议](https://blogs.bing.com/webmaster/2026/2/Introducing-AI-Performance-in-Bing-Webmaster-Tools-Public-Preview/)

以下是针对 Anki Forge 的 **C 级实施建议**，最终排序应结合实际文档缺口和版本状态：

1. 用一个稳定介绍明确库的类别、输入输出、支持语言/运行环境、许可证、与 Anki 的关系，以及不支持的场景；官网、README、包注册页与仓库保持同一组事实。
2. 给高价值任务提供能运行的端到端教程，如从既有数据生成 `.apkg`、处理图片/音频、批量生成牌组；仅为真实已发布能力建页。
3. 提供版本化兼容性与边界表，写清格式、Anki 版本、平台、依赖、性能条件、错误行为，避免无条件“全面兼容”“零依赖”“最快”。
4. 写有证据的选型/迁移说明，列明适用和不适用条件；比较对象的事实链接到它们自己的文档，标注核对日期，避免自设指标宣布胜出。
5. 将关键代码示例纳入发布验证；教程指向可复现仓库或测试夹具。展示真实维护历史、issue 入口和已验证使用案例，减少 AI 引用过时 API 的机会。

Google 于 2026-10-01 更新生成式内容指南，继续强调人工核实准确性及实际增益；规模化生成缺少附加价值的页面可能违反垃圾内容政策。评级员指南不是“第一名公式”，评分也不直接影响排名。**A/B**。[Google 生成式内容指南](https://developers.google.com/search/docs/fundamentals/using-gen-ai-content)

## 9. 验收与下一步数据需求

| 层次 | 建议记录 | 不能推出的结论 |
| --- | --- | --- |
| 技术可读性 | 关键 URL 正文、HTTP、指令、canonical、语言映射；真实 bot 成功率 | 成功访问不证明已收录 |
| 收录与资格 | 站长平台索引检查、AI inclusion、提交记录 | 提交成功不证明排名提升 |
| AI 可见性 | Google AI 展示、Bing 引用页及趋势 | 展示/引用不等于选型推荐或点击 |
| 推荐准确性 | 固定中英文问题集，记录日期、平台、模型/模式、是否联网、是否引用官网、能力是否描述正确 | 一次抽样不能代表整体市场份额 |
| 实际采用 | AI 来源访问、入门页阅读、仓库/下载跳转、示例完成情况 | 安装量变化不能单独归因于 GEO |

本表为 **C 级衡量方案**；平台指标范围以前文官方文档为准。建议先基线、后按批次发布、再观察数周；引用率变化受需求、竞争内容和平台系统变化共同影响，不能仅凭前后两次截图作因果归因。

与 `2026-09-22-ai-search-official-research.md` 相比，本次确认原有“搜索/训练分离”“llms.txt 非排名因素”“FAQ 富结果已停止”仍有官方依据；Google 帮助页现在直接写明全球推出及低展示量的报告缺失原因，不再沿用旧文的灰度表述。主要新增是 Bing 10 月 Topics Portfolio beta；Google 指南的本次访问日期不代表其发布日期或更新日期。旧文的本站 URL 数量、抓取结果与账号判断不在本文件中当作当前事实复用。
