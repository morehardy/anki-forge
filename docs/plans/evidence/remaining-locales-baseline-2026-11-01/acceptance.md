# 三语历史基线维护验收

2026-11-01；独立审阅者 `/root/baseline_acceptance`。本次在隔离工作树、固定提交 `76f17ee0c964334c9fd11b426a0a7c73c9487eea` 完成 P0 本地验收，未修改 en／zh-CN／es 正文、源审核结论或工程门禁。仅新增本目录证据。未提交 Git、未部署网站。

两套配置均通过严格构建、同输入复建、八项工程检查、独立使用任务／机器读取报告，以及 `release-evidence.mjs finish` 和 release evidence 检查。共完成 **36 个真实浏览器使用任务和 24 个真实机器读取任务**；没有 critical／major 未解决问题。

| 配置 | site / base / customDomain | 原始 manifest SHA-256 | 完整交付文件 |
| --- | --- | --- | --- |
| prefix | `https://morehardy.github.io` / `/anki-forge` / 空 | `d8f7a8a924940199f76e992fabefe5a0d195eb422c3d6ac313be6c0f599eed76` | 381 |
| root | `https://ankiforge.dev` / `/` / `ankiforge.dev` | `c6973e2a178003ad701b444c4dc5d67188d9748ec1966f0e7a28e61289a206cb` | 382，含 CNAME |

两套配置的 `inputSha256` 均为 `560ffd17e00312fb4462c96a14e9e04dbd212a6b82075c3c9950f743f7822031`，releaseLocales 为 en、zh-CN、es。各有 102 个正常 HTML 页面、1 个 404、93 份 Markdown、3 份 llms。各自的同输入复建确认 199 个 HTML／Markdown／llms 文件逐字节一致。报告绑定全部当前资源哈希，并保留实际原始 manifest；资源绑定不意味着人工阅读全部页面，实际读取范围以任务记录为准。

## 证据与验收方式

- 工程：`engineering/{prefix,root}/release-evidence.json` 及其哈希绑定日志，涵盖 check:content、check:site、check:surface、check:machine、check:schema、check:gates、test:browser、check:search。独立复建日志和新公开面检查位于 `engineering/`；最终 finish／release 检查日志也保留在此。
- 使用任务：`usage-tasks-{prefix,root}.json`；每语实际执行 Rust／Python／Node 基础制卡入口导航、JSON／媒体、原始 APKG 更新、排错六任务。`usage-{prefix,root}/browser-tour.json` 保留页面正文、代码、真实剪贴板复制、下载与截图。三语各自的四个 recipe 下载链接均经实际 HTTP 获取并核对源文件与 manifest 哈希。
- 机器读取：`machine-read-{prefix,root}.json`；每语四任务分别从其实际 llms HTTP 响应开始，沿真实链接读取同语 Markdown，涵盖 Python 安装、Node API、JSON 更新和兼容性／构建限制。每配置共 42 次实际 HTTP GET，保留状态、MIME／UTF-8、无下载 disposition、URL、完整正文和哈希。实际读取涉及每语 9 篇不同 Markdown 及 llms；不使用英语回退。
- SDK 执行复用：`execution-reuse-audit.json` 明确记录当前 23 项源／依赖输入与既有真实执行相同、三份基础程序与执行副本字节相同；另用 ZIP、Zstandard、SQLite 独立解码检查 10 个保留 APKG。三 SDK 基础包各为 1 笔记／1 卡；三 SDK 的 JSON v1/v2 各为 3 笔记／3 卡／2 个精确媒体，GUID 不变且仅一处答案变化；比较／预备报告允许这次 RISK.NOTE_CHANGED。重复键恢复包为 2 笔记／2 卡，真实日志含 NOTE.KEY_DUPLICATE。**本次未重跑 SDK 消费程序**，报告显式引用原执行及当前输入绑定；没有把旧报告改哈希伪装新执行。
- 独立语义判断：`review-conclusions.json` 是读取新采集内容后撰写的结论，不是旧 passing answers。`assemble-reviewed-reports.py` 仅装配这些结论和真实观察，不自动判定审阅通过。观察辅助脚本不输出通过结论。
- root 审阅：root 浏览器与 HTTP 独立运行；`root-review-comparison.json` 核对全部任务正文／代码和 42 个响应完整正文，仅将精确的 prefix 部署 URL 改为 root URL 后逐字相同。因此沿用本轮刚完成的语义判断，使用实际 root URL 和哈希。未将 GitHub 源码链接或任意路径泛化替换。另直接查看 root 三语截图确认排版。
- 直接查看的截图：prefix 的 en/Rust、zh-CN/Python、es/Node，以及 root 的 en/JSON、zh-CN/更新、es/排错。1440×1000 下正文、代码、导航和目录可读，没有阻挡任务的重叠或裁切。其余截图作为实际浏览器状态留存，不宣称逐一人工查看。

浏览器实际 origin：prefix `http://127.0.0.1:61256`，root `http://127.0.0.1:64406`。机器读取实际 origin：prefix `http://127.0.0.1:61356`，root `http://127.0.0.1:64508`。服务均为对应冻结产物的本地预览，不代表生产部署。

## 历史公开面与生产状态

`trusted-surface.mjs ad98a6f5b9293e7657429e3ed423d11e55259a74` 选择不可变 base revision 的历史合同，再由两套 engineering 验证保留。两套配置也直接检查本次新捕获的完整三语 baseline：保留 102 页和 96 个机器资源；旧机器 baseline 的 60 个资源继续受保护。未放宽门禁或覆写旧机器历史。

本轮父任务先前独立核验的生产记录在 `../remaining-locales-2026-11-01/three-locale-capture.json`、`three-locale-verified-manifest.json` 与 `engineering/three-locale-live.log`。它记录 production `ad98a6f`、Website workflow 41314923919、部署 job 124874543206、独立下载 artifact／manifest 摘要，以及真实线上 102 URL、资源、feed、sitemap、404 校验。这里使用该已验证生产产物作为历史公开面来源，**不是宣称当前维护提交已部署**。维护提交没有新增公开内容，无需为提交号变化再次循环捕获基线。

## 保存与后续使用

原始 manifest 分别为 `prefix-origin-manifest.json`、`root-origin-manifest.json`。受审产物保存在此隔离工作树的 `target/remaining-locales-baseline/{prefix,root}-artifact/`；官方工程输出同时保留在 `target/website-localization/{prefix,root}/` 并复制到本目录。两套产物位于构建输入之外，没有以第二套构建覆盖第一套受审产物。

本目录只证明上述固定三语输入的维护验收。加入 pt-BR 或其他构建输入变化后，应运行对应语言数的实际验收；不得将此 inputSha 改写成新输入。仅提交号变化是否可重用，由现有验证器按真实输入和资源字节判断。需要重新核验某配置时应恢复其受审 dist，并设置匹配的 SITE_URL、SITE_BASE、SITE_CUSTOM_DOMAIN；不要在另一配置的 dist 上声称该配置通过。

开始 prefix engineering 时普通沙箱禁止监听 localhost（EPERM），该次中断日志保留为 `engineering/prefix-engineering-sandbox-interrupted.log`。随后以实际浏览器所需权限完整重跑，最终正式 receipt 的八项均通过；没有跳过浏览器检查。root 构建前也记录了当前 prefix 产物不能用于 root 的预期拒绝。完整 npm test 留给父任务在最终代码上统一运行，本子任务未重复全套单元测试。Anki GUI 导入、播放、学习者复习状态及新部署都不在本次新增验证声明中。
