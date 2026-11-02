# P2 日德样稿与八语菜单探针

2026-11-01，独立代理 `/root/baseline_acceptance`。先固定 [intentions.md](intentions.md) 和 [queries.json](queries.json)，再运行当前 `website/src/lib/search.mjs` 与真实样稿的小语料探针，并在复制的固定 prefix 产物上运行八语菜单浏览器探针。未改变网站源码、正式 dist、releaseLocales、检索门槛或正式词表。

**发现了八语菜单可见性问题，以及日德搜索数据需要补齐的具体风险。此记录不是日德发布验收。**

## 检索观测

[search-observations.json](search-observations.json) 记录12条预先固定输入，每条分别运行纯词法与仅含 brief 中5个术语及 API 的词表，共24个观测。语料是每语首页UI及4篇真实样稿；通过排序函数的 `pagefindResults` 参数传入的正文只是明确标记的 fixture 输入，未运行 Pagefind。没有根据查询补入同义词、任务角色或停用词。

- 日语无空格 Rust 入门、JSON 图片/音频、原始 APKG 更新、Python 与 API 标识混排，目标分别排第一，说明这几个实际样稿任务没有暴露基本分词阻断。
- `でっきをこうしんしたい` 虽然目标 updates 排第一，但 matchedTerms 仅为 `を`、`した`、`い`，没有“でっき／こうしん”语义证据。对照的 `デッキを更新したい` 有明确任务词。**不能把名次碰巧正确当作假名变体已支持。** 完整日语生产词表应处理必要的同义写法和停用词，再用独立自然任务题验证。
- 德语 `Stapelaktualisierung mit dem ursprünglichen APKG` 的复合词是一个 Segmenter 词段，但未贡献匹配；Python API 排第一，updates 第二，主要依据是 mit／dem／ursprünglichen／APKG。Rust 自然任务的 quickstart 排第三；Notizschlüssel 自然句的 updates 排第二。需要完整本语 metadata、检索概念、停用词和任务意图之后再判断排序，而不能直接把这些情况归因为底层引擎缺陷。
- 每语无关字符串与数字负例均无结果。两个模式都记录了预先约定的前三名目标检查，但这只是诊断条件；本报告没有由该布尔值推断完整搜索通过。

没有执行完整31篇文档语料、正式30题、真实Pagefind索引、搜索交互或外部搜索效果验证，也没有调整源码或词表掩盖观察结果。

## 八语菜单：有阻挡可见性的失败

[fixture-provenance.json](fixture-provenance.json) 记录固定 `target/remaining-locales/pt-br/prefix-artifact` 的 manifest、输入摘要和隔离复制件。复制件位于 `target/remaining-locales/sample-probe-artifact`，沿用真实 compiled header/menu CSS 与 DOM classes，仅把菜单扩展为 configuredLocales 的八语（当前语+七个目标）、指向本地 fixture；日德页使用真实 quickstart/API/UI 样稿。正式发布集合没有变化。

[menu-observations.json](menu-observations.json) 记录 Chrome 的9个条件：320/390×360 CSS px、100%/200%字号、JS开/关8组合，另有德语当前项320×360/200%无JS。200%是逐元素将计算字号乘二，视口和其他间距不缩放；不等同每种浏览器原生文字缩放。

| 条件 | 真实观察 |
| --- | --- |
| 390px，100%，JS开或关 | 七项完整可见，Tab焦点可见，逐项Enter到达正确本地目标 |
| 320px，100%，JS开或关 | 菜单左缘 x=-74.8px，目标文字被视口左侧截断；无水平滚动可恢复 |
| 320px，200%，JS开或关 | 菜单 x=-109.8px，高389.3px；最后一项起点 y=514.3px，位于360px视口外 |
| 390px，200%，JS开或关 | 菜单 x=-109.8px，高389.3px；最后一项起点 y=463.3px，位于视口外 |
| 德语当前项320px，200%，无JS | 同样有左侧裁切与屏外末项；并非仅英语当前标签导致 |

各条件七个链接都可以被Tab依次聚焦、Enter导航，**但失败条件的焦点项不可见或不可完整辨认**。菜单 `overflow-y: visible`、无内部滚动；实际向下滚轮600px只移动正文，sticky header的菜单仍停在原位置，末项仍不可见。DOM存在与盲按Enter能跳转不等于可用的键盘访问。无JS也复现，修复应保持原生details/link可访问性。

已向父任务报告，建议定向限制菜单的水平位置和最大可用高度，使内部可滚动；没有自行更改共享样式。可直接查看 [320px/100%截图](en-320-text100-js0-open.png)、[390px/200%截图](en-390-text200-js0-open.png) 和 [德语200%滚动后截图](de-320-text200-js0-after-scroll.png)。本报告只针对记录的固定产物，父任务后续CSS修复需要重新跑同样输入确认。

## 边界和复现

日德390px/100%的 [日语截图](sample-ja-390.png) 和 [德语截图](sample-de-390.png) 可读，真实标题能折行。但这里只是 micromark 展示样稿的 fixture：没有完整本语导航、正式Starlight代码块增强、真实日德字体选择或OG生成，section注释被当作可见文本显示。因而不把主内容的全部几何表现当成正式产品结论，也不把德语fixture长代码产生的正文宽度当作生产缺陷。明确的菜单结论来自未修改的真实菜单CSS/DOM结构和实际坐标。

初次浏览器脚本因响应式页面同时存在两个语言菜单，未加 visible 过滤而触发严格selector歧义。失败记录保留为 `menu-observations-selector-error.json` 与 `menu-run-selector-error.log`；修正观察脚本后完整重跑，未修改测试条件或产品。最终 `menu-observations.json` 无 harness 错误。

使用 Node24：先执行 `search-probe.mjs`；`build-menu-fixture.mjs` 创建一次全新隔离目录（存在则拒绝覆写），然后以 SITE_URL=https://morehardy.github.io、SITE_BASE=/anki-forge、空SITE_CUSTOM_DOMAIN运行 `menu-probe.mjs`。浏览器固定使用本机 `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`，localhost监听需要对应执行权限。脚本和原始数据均保留在本目录。没有运行整套单元测试、部署、Anki客户端或日德完整发布流程。
