# 日语完整候选验收

2026-11-01。用户要求继续完成日德语言；按每语完整验收推进，日语先形成五语候选。源码冻结于 `5a50666dbc15e9aa7342de1d3d6587d1a9f147bd`。**本地实现与双配置验收完成，未推送或部署。** 德语草稿此时只在排除构建输入的 evidence staging 中。

## 实现与独立源审

- 日语交付29篇文档、2篇博客、210个业务词条、11个框架词条、全部元数据／导航／提示／语言规范／检索数据。五语共170个正式主题，另有404页；机器资源为155份Markdown和5份llms。
- 实际失败驱动两处搜索修复：保留日语浊点／半浊点，避免片假名概念命中更长的无关词。公开排序接口留下红绿回归，Latin重音折叠保持正常。日语分享图明确使用JP优先字体及SVG语言属性。
- 技术和表达分别由非作者代理独立审阅，170条双轴记录绑定最终输入。完整审校、后来单句 `internal-tools` 边界增量及原始快照分别保留；旧四语按真实差分复核，未冒称重读所有旧文。新语无未决major／critical，保留3项非阻塞表达minor。报告为 `technical-source-review-final.json` 与 `linguistic-source-review-final.json`。
- 初次机器预览发现 `Optional` 协议标题被翻译，已修正词条，未放宽验证器。原错误控制台摘录及日志复用限制见 `preview/initial-machine-heading-failure.md`。

## 实际验证

完整单测149/149；严格类型检查123文件，0错误／警告／提示。文档、AI及Skill quickstart合同通过；实际执行14个完整文档程序，并独立检查23个APKG。后续补充的API边界句没有改变执行代码，正式内容门禁再次验证代码保真。

两配置的 `inputSha256` 均为 `85f5276a59a50143053000a91fe702d76521e13fb9232db86a2498016a029d1f`。

| 配置 | site / base / customDomain | 原始manifest SHA-256 | 交付文件 |
| --- | --- | --- | ---: |
| prefix | `https://morehardy.github.io` / `/anki-forge` / 空 | `afb99163be210ec5a8b27226faf0ce0cb91193662c73240d0757259cc8f95feb` | 597 |
| root | `https://ankiforge.dev` / `/` / `ankiforge.dev` | `23f7c509d6ddbe5dec65cf0274309926129ef3c910cffcb900913b608efd564c` | 598，含CNAME |

每配置实际复建确认331个HTML／Markdown／llms文件逐字节一致，八项工程门禁全部通过，包含43项浏览器检查及真实Pagefind。日语核心12/12、正例23/25、负例5/5；其余语言结果为en 25/25、zh-CN 24/25、es 25/25、pt-BR 23/25，核心和负例均全过。日语APKG内容查询及genanki替换表达两项非核心缺口如实保留；没有改题、目标页面或阈值。两个环境的五个日语负例另实际读取命中正文，结果整体未越过能力边界，见 `search-negative-boundary-{prefix,root}.json`。

## 100个独立产物任务

两配置共完成60个浏览器使用任务及40个机器读取任务。浏览器共60次真实完整程序复制、40次输入下载；机器共140个真实HTTP响应。四份报告 `usage-tasks-{prefix,root}.json`、`machine-read-{prefix,root}.json` 均已接入 `release-evidence.mjs finish`，两套 `check:release` 通过。

日语任务实际正文、代码、下载及截图经过独立审读。旧四语浏览器正文逐字相同，明确复用此前判断；旧机器正文相同，llms新加的日语入口另行检查。根路径任务仍实际操作／请求，只在逐字或精确部署URL替换比较成功后复用语义结论。不是把prefix报告的配置字段改成root。

公开SDK消费程序为显式复用：独立复核23项源码／依赖绑定、10个APKG的ZIP／Zstandard／SQLite／媒体、6份比较报告、14份日志／安装解析记录。它与本轮14个文档程序的实际执行范围不同。没有声明运行Anki UI或新跑全部公开SDK程序。

每份机器报告绑定160个必需机器资源；使用报告另绑定真实读取HTML及recipe，共204个资源。全部资源绑定不代表逐页人工阅读。工程收据独立核验全部597／598个交付文件。正文布局16组尺寸／文字缩放观测无横向溢出，实际截图及OG通过人工检查；打开的语言菜单是覆盖正文的正常浮层。本机字体结论不冒充Linux CI验证。

原始产物存于 `target/remaining-locales/ja/{prefix,root}-artifact`，原始manifest及工程收据在本目录。后续证据提交只改变排除构建输入的文件；CI仍须按其真实HEAD重建并运行门禁。

## 发布边界

远端主线核对时仍为三语 `ad98a6f`。本轮没有把五语候选当作生产历史基线，没有推送、部署或线上核验声明。P0历史保护、巴葡先行发布及verified基线推进仍是实际发布顺序；日语本地候选可独立保存，随后继续德语六语候选。
