# 德语检索词表修正

初次真实Pagefind结果完整保存在 `preview/initial-prefix-failure/`：核心12/12、正例21/25、负例4/5。失败项为概念关系、Notizfelder、APKG内容、稳定键术语和直接重置学习历史。未修改冻结题目、目标页面、意图、阈值、排序实现或旧五语数据。

修正只涉及 `website/search-concepts.json` 的德语部分及一条德语主题术语：

- `fundamentals` 补充 `hängen`、`zusammenhängen`、`unterscheidet`，保留必须出现笔记／卡片等上下文的条件；将真实文档领域复合词 `Notizfeld`、`Notizfelder`、`Notizschlüssel` 纳入上下文。它们表达关系或区别，不是把整条问题当关键词。来源是 `docs/de/concepts.md` 的Project、Notiz及模型／字段／模板说明。
- `identity` 补充稳定键的自然德语形式，并关联既有 `concepts`、`updates` 内容角色。没有将其提升为任务意图；明确的SDK或迁移意图仍按原规则处理。两篇来源正文已经解释稳定命名空间、笔记键及更新身份。
- `package-structure` 补充德语容器内容谓语 `steckt`，仍要求APKG／Paket等上下文。与原有 `enthält`、`Bestandteile` 表达同一结构含义。
- `compatibility` 将 `Lernverlauf`、`Lernfortschritt`、`Wiederholungsverlauf` 关联到现有兼容性边界，并补主题词 `Lernverlauf und Importeinstellungen`。`docs/de/compatibility.md` 的Anki导入段和 `docs/de/updates.md` 的合并／时间戳段已经说明客户端条件、已有学习计划和无法保证历史恢复；没有增加“直接重置历史”的能力、代码或承诺。

真实搜索复跑仍使用相同30题。是否达标以新的搜索输出为准；最终负例还必须独立读取命中正文，不能仅凭排名通过宣称语义边界通过。
