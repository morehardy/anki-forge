# 三语历史基线维护验收：bootstrap fixture 回补后

2026-11-01；独立审阅者 `/root/baseline_acceptance`。固定验收源码提交为 `1a448e963ee7d28125d0c637e22e398409e0fae8`，分支 `codex/remaining-locales-baseline`。它在 P0 证据提交 `bec6922` 上仅回补 `4302587` 的 trusted-surface 测试及两份不可变 bootstrap fixture，没有带入 pt-BR 内容、搜索或 Header 改动。本轮未部署网站。

**完整 npm test：147/147 通过，0失败、0跳过。两套配置均重新通过严格构建、同输入复建、八项工程检查、真实使用／机器任务、finish 和 release evidence 检查。** 本轮实际重新完成36个浏览器使用任务和24个机器读取任务，不是只刷新旧报告哈希。没有 critical／major 未解决问题。

| 配置 | site / base / customDomain | 本轮原始 manifest SHA-256 | 完整交付文件 |
| --- | --- | --- | --- |
| prefix | `https://morehardy.github.io` / `/anki-forge` / 空 | `02ad75fc0af832034eeb33549e79a486d0218eb8898389fefbb3ca964b3a4d20` | 381 |
| root | `https://ankiforge.dev` / `/` / `ankiforge.dev` | `f8a47ac8708def70173b1ef8a82daf6d39bceab999ab5fdfbce960e38705935a` | 382，含 CNAME |

当前 `inputSha256`：`3e2061dc5be3e1df804a54a1ba3818abcbf903d9a51e73259424f5c3c62e9ec6`。发布集合仍为 en、zh-CN、es；每套有102个正常HTML、1个404、93份Markdown、3份llms。同输入复建各确认199个HTML／Markdown／llms文件逐字节一致。

## 真实执行与报告

- `engineering/maintenance-full-test.log`：本轮完整147项单元测试，包含独立冻结 bootstrap fixtures 的回归。源码未在此后改变。
- `engineering/{prefix,root}/release-evidence.json`：两套新工程收据及哈希绑定附件。分别执行 check:content、check:site、check:surface、check:machine、check:schema、check:gates、test:browser、check:search，随后 attach 独立报告并检查完整产物。所有检查均通过，没有跳过浏览器门禁。
- `usage-tasks-{prefix,root}.json`：每语Rust／Python／Node基础制卡、JSON与媒体、原始APKG更新、排错六任务。新 `usage-*/browser-tour.json` 记录真实页面导航、搜索、剪贴板复制、页面正文及截图；每语四份recipe下载均为实际HTTP获取，并核对源文件与当前manifest摘要。
- `machine-read-{prefix,root}.json`：每语从本语llms实际HTTP读取开始，完成Python安装、Node API、JSON更新、兼容性／构建限制四任务。每配置42次真实HTTP GET，涵盖每语9篇不同Markdown及llms，保留URL、状态、MIME／UTF-8、无下载disposition、完整正文及哈希。没有英语回退。
- 两类独立报告各绑定124个资源：全部96个必需Markdown／llms，加实际访问的24个HTML和4个recipe资源。该绑定范围不等于声称人工阅读全部31篇文档。**完整交付文件另外由工程收据校验**；没有将未在这些使用任务中读取、会随重建生成的showcase APKG／provenance纳入可复用的任务字节范围。

本轮浏览器实际origin及机器读取origin保存在各自原始观察文件中。均为受审冻结产物的真实本地HTTP服务；它们不是生产上线声明。

## 原始独立判断与复用边界

旧 `76f17ee` 的整套证据逐字保存到 `history/76f17ee/`，并通过旧inventory核对其中205个已编目文件完全不变。历史规范路径也保留在Git提交 `bec6922` 中；归档文件中的原始路径／inputSha没有改写成新候选。

本轮先重新运行全部浏览器与HTTP场景，再比较新旧原始观察。`maintenance-{prefix,root}-comparison.json` 记录原始与当前不同的input／manifest身份：各配置所有HTML和机器文本、所有实际捕获任务正文与代码、recipe下载完全相同。因此显式沿用本轮先前由独立审阅者写出的语义判断（`review-conclusions.json`），而不是复制旧passed报告并更换哈希。新报告以维护重跑的reviewerRun、实际读取URL及新manifest绑定。

`root-review-comparison.json` 另比较本轮新root与新prefix；只归一化精确的部署URL，完整任务正文／代码与42个HTTP正文一致。直接查看本轮prefix的en/Rust、zh-CN/Python、es/Node，以及root的en/JSON、zh-CN/更新、es/排错截图；1440×1000下内容、导航与代码可读，没有阻挡任务的重叠或裁切。其余截图保留为实际状态，不声称全部人工查看。

SDK消费者本轮没有重跑。`execution-reuse-recheck.json` 重新核对23个当前源／依赖输入与先前真实执行相同；保留未改写的 `execution-reuse-audit.json` 及旧执行日志/APKG。原独立审计解码10个真实APKG：三SDK基础包各1笔记/1卡；JSON v1/v2各3笔记/3卡/2媒体、GUID不变且只改一个答案；策略允许RISK.NOTE_CHANGED；重复键恢复包有2笔记/2卡且真实日志包含NOTE.KEY_DUPLICATE。该复用不声称SDK重跑、Anki GUI导入、媒体播放或学习者调度验证。

## 重新生成资源的差异

不声称所有交付资源与旧候选逐字相同。`generated-resource-inspection-{prefix,root}.json` 独立检查每套9个变动资源：

- 7个重新生成的showcase APKG二进制不同。新旧包分别解压Zstandard、进行SQLite integrity check，比较笔记GUID／模型／字段内容、卡片序号、解码媒体及稳定身份，全部相同。比较明确排除了修订时间以及随集合载荷变化的摘要，不伪称包字节一致。
- `showcase.json` 只改变真实sourceCommit与上述包的大小／摘要；显示内容、数量、更新比较和源码摘要相同。当前工程check:site还逐个核对实际下载字节与showcase记录一致。
- `pagefind-entry.json` 仅对象键序不同，解析后的JSON与语言索引摘要完全相同；本轮实际搜索检查仍重新执行。

最初的维护比较辅助脚本曾过度假定全部manifest资源都逐字相同，被新生成APKG正确触发。随后依据实际差分完成上述额外独立检查，将结论限定为真实相同的页面／机器／任务内容，同时记录真实不同的生成资源。未修改任何产品验证器或工程门槛。

## 历史公开面与生产

两配置使用 `trusted-surface.mjs ad98a6f5b9293e7657429e3ed423d11e55259a74` 选择的不可变base合同执行工程验收，并直接验证当前新捕获的完整三语baseline：102页、96个机器资源保留；旧机器baseline的60个资源继续受保护。

既有生产核验见 `../remaining-locales-2026-11-01/three-locale-capture.json`、`three-locale-verified-manifest.json` 与 `engineering/three-locale-live.log`：production `ad98a6f`、Website workflow41314923919、deploy job124874543206、独立artifact／manifest摘要，以及102个真实线上URL、资源、feed、sitemap和404核验。它是本次历史公开面的可信来源，不代表维护提交1a448e9已部署。维护提交未新增公开内容，不因提交号变化循环推进基线。

## 保存与后续

本轮受审产物：隔离工作树 `target/remaining-locales-baseline-maintenance/{prefix,root}-artifact/`。原76f17ee产物仍在 `target/remaining-locales-baseline/{prefix,root}-artifact/`。原始manifest保存在本目录；官方工程输出复制到 `engineering/{prefix,root}/`。当前隔离工作树dist恢复为本轮root产物。

证据提交本身只修改排除构建输入的 `docs/plans/evidence/`。工程收据严格属于源码提交1a448e9的本轮产物；后续证据提交、PR或合并HEAD应由CI重新构建并执行工程检查，既有post-build验证器再按真实input和受审资源判断能否复用独立报告。不得改写报告input来批准pt-BR等新输入。新增语言、搜索／Header源码变更须执行其对应验收。
