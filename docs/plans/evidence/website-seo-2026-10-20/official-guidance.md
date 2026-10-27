# 官网 SEO 官方依据核对（2026-10-20）

日期按用户时区 America/Los_Angeles。本文复核 2026-09-22 旧研究中的关键规则，仅记录第一方资料。**未登录本站 Google Search Console、Bing Webmaster Tools 或 CDN 后台**；不能据此断言本站实际收录量、排名、AI 引用或账号设置。网站源码和线上响应由本次主审计另行核对。

标记：**证据**是官方公开规则或能力；**建议**是针对项目的实施推断，没有本站效果数据，也不构成排名或 AI 推荐承诺。

## 当前最相关的八项规则

1. **先满足索引与展示资格，再改进内容。** 证据：Google AI 搜索依赖现有搜索系统；页面须已索引、允许摘要，站点还须纳入 Search generative AI。满足条件仍不保证抓取、索引或展示。Google 不因 `llms.txt` 提升或降低排名，也不要求专用 AI schema。建议：优先查正文可获取性、索引指令、WAF 和站长平台，再完善实际用户任务的教程与可复现示例。[Google AI 优化指南](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide)

2. **标题要表达该页的具体用途，并匹配正文语言。** 证据：Google 建议独特、简洁、描述性的 `<title>`，避免重复模板和关键词堆积；标题与正文语言不同可能被重写。建议：中文教程使用中文标题，包含真实任务或 API 名称；不把长首页宣传语作为每页后缀。官方没有固定的“超过多少字符就处罚”规则。[Google 标题链接指南](https://developers.google.com/search/docs/appearance/title-link)

3. **sitemap 的 `lastmod` 必须可信。** 证据：Google 忽略 `priority`、`changefreq`；只有持续可核验的 `lastmod` 才会被使用，应反映正文、结构化数据或链接等实质更新。建议：从页面真实内容变更生成时间，不在每次构建时刷新全站日期；无法可靠计算时宁可省略可选时间字段。[Google sitemap 指南](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap)

4. **双语关系需要双向对应，也需要真实翻译。** 证据：每个版本列出自身和其他对应版本，使用完整 URL；缺少返回链接可能使标注被忽略。简体中文可用 `zh-Hans`；地区代码则表达对应地区的目标受众。Google 以正文识别语言，不用 `lang` 或 `hreflang` 判定正文语言。建议：只映射实际存在且内容对应的中英文页；若面向各地区的简体读者，优先评估 `zh-Hans`，不机械地使用 `zh-CN`。[Google 多语言指南](https://developers.google.com/search/docs/specialty/international/localized-versions)

5. **结构化数据表达可见事实，不补造评分。** 证据：标记应符合页面可见内容；Google `SoftwareApplication` 富结果要求名称、价格及评分或评论。没有真实评分时，不能为通过检测虚构评分；有效 schema 与获得富结果资格是不同问题。建议：选与页面实际内容匹配的 schema，把文档质量放在增加 schema 数量之前。[结构化数据通用规则](https://developers.google.com/search/docs/appearance/structured-data/sd-policies)、[SoftwareApplication](https://developers.google.com/search/docs/appearance/structured-data/software-app)

6. **搜索爬取与训练授权分开处理。** 证据：OpenAI 明确 `OAI-SearchBot` 用于搜索，`GPTBot` 用于可能参与训练的抓取，两者可独立设置；官方建议同时核查公布的 IP 能否通过基础设施。`ChatGPT-User` 是用户触发的访问，不用于控制自动搜索索引。建议：若希望在 ChatGPT 搜索展示，应避免误拦搜索爬虫；无需以开放训练作为交换条件。[OpenAI Crawlers](https://developers.openai.com/api/docs/bots)

7. **IndexNow 提交真实 URL 变更，保留独立的索引验证。** 证据：面向参与的搜索引擎，新增、实质修改、删除或跳转可提交；不要反复提交未变更的页面。HTTP 200 表示提交已接收处理，不能证明已索引；sitemap 仍用于完整发现。建议：在文档发布后提交变更 URL，记录响应，再从站长平台检查索引状态；不称其为 Google 索引 API。[IndexNow FAQ](https://www.indexnow.org/faq)

8. **分平台衡量展示、引用和到站，不能混成“AI 权重”。** 证据：Google 独立生成式 AI 报告主要报告展示，可按页面、国家、设备、日期分析；Bing 报告引用和 grounding queries，引用不等于点击、排名或权威度。建议：记录普通自然搜索点击、AI 展示/引用、有效到站及安装相关行为，按发布批次比较趋势，避免将一次波动归因给某个标签。[Google AI 效果帮助页](https://support.google.com/webmasters/answer/16984139)、[Bing AI Performance 公告](https://blogs.bing.com/webmaster/2026/2/Introducing-AI-Performance-in-Bing-Webmaster-Tools-Public-Preview/)

## 旧记录中 2026 年能力的复核与更新

| 旧记录事项 | 本次核对结果及来源 |
| --- | --- |
| Google 独立 AI 效果报告与 2026-08-31 全球推出 | **确认**。2026-06-03 公告及当前帮助页均保留全球推出说明；帮助页当前明确 AI Overviews / AI Mode 的展示数据，不能扩写为独立 AI 排名或转化报表。[发布公告](https://developers.google.com/search/blog/2026/06/gen-ai-performance-reports)、[报告帮助](https://support.google.com/webmasters/answer/16984139) |
| “帮助页仍保留灰度说明” | **此描述现已过时**。当前报告帮助页说明全球推出；看不到报表通常是展示量不足。旧发布公告正文仍保留最初分批发布叙述，应优先读其顶端的更新说明。[报告帮助](https://support.google.com/webmasters/answer/16984139) |
| Search Console 的 Search generative AI 控制 | **确认**。当前官方帮助页标明 2026-08-31 全球推出；默认纳入，子属性可继承父属性；这是展示纳入控制，不是其他搜索的排名信号，也不控制训练。本站实际设置尚未检查。[控制帮助](https://support.google.com/webmasters/answer/16908024) |
| FAQ 富结果在 2026-05-07 停止 | **确认**。Google 更新日志 5 月 8 日条目明确停止日期；6 月 15 日条目说明移除文档。保留有用的 FAQ 正文，但不能以新增 `FAQPage` 换取该展示作为收益承诺。[Google 更新日志](https://developers.google.com/search/updates) |
| llms.txt v2 与 Lighthouse 检查 | **确认**。原提案确实标注 2026-08-10 修改为 v2；Chrome 文档确实存在对应检查，2026-05-05 更新，并将不存在文件的 404 记为 N/A，文件仍可选。这些是文档/agent 可用性依据，不代表 Google 搜索加权。[llms.txt 提案](https://llmstxt.org/)、[Lighthouse 检查](https://developer.chrome.com/docs/lighthouse/agentic-browsing/llms-txt) |
| Bing AI Performance 2026-02-10 公开预览 | **确认**。旧路径抓取超时，新规范路径可读。原报告对引用、页面和采样 grounding queries 的描述仍有依据。[Bing 2 月公告](https://blogs.bing.com/webmaster/2026/2/Introducing-AI-Performance-in-Bing-Webmaster-Tools-Public-Preview/) |
| Bing 新增测量能力 | **旧记录不完整**。2026-06-16 已宣布 Intents、Topics、Citation Share、Compare 全球预览；2026-10-13 新增 Topics Portfolio beta。后者需要足够数据，小站可能不可见，数值仅适合方向性规划，不能证明优化因果关系。[6 月公告](https://blogs.bing.com/search/2026/6/New-AI-Visibility-Insights-in-Bing-Webmaster-Tools-Intents-Topics-Citation-Share-Compare/)、[10 月 beta 公告](https://blogs.bing.com/webmaster/2026/10/introducing-topics-portfolio-beta-in-ai-performance/) |

本次没有全面复核旧报告中的 Anthropic、Perplexity 爬虫部分，也没有验证旧审计的“29 个页面”等本站历史统计；这些不能当作本次线上事实沿用。本文没有修改旧报告或网站代码。
