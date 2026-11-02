# 修改笔记内容时，保持 ID 不变

Canonical page: https://ankiforge.dev/zh-cn/blog/stable-ids-and-deck-updates/
Language: zh-CN
Description: 更新生成的 Anki 牌组时，稳定 ID 与构建基线如何共同发挥作用。
Published: 2026-09-22T00:00:00.000Z

生成的牌组很少在首次导出后就不再改变。定义会完善，例子会更清楚，错误会被修正。下一次构建需要知道哪些笔记仍是原来的笔记，只是内容有所改善。

考虑这个小改动：

| 字段 | 第一版 | 下一版 |
| --- | --- | --- |
| 正面 | hola | hola |
| 背面 | hello | hello; hi |
| 稳定 ID | es:hola | es:hola |

含义扩展了，逻辑笔记仍是同一条。

## 将身份与内容分开

显式稳定 ID 为笔记提供持久键。编辑笔记时保留该 ID，创建另一条逻辑笔记时选择新 ID。

身份还具有项目上下文。同一牌组的各次发布应保持项目命名空间一致。仅笔记 ID 本身只是可重复更新流程的一部分。

## 为下一次构建提供基线

之前的包告诉 Anki Forge 上次导出了什么。构建修订后的项目时使用它：

```rust
let comparison = project.compare(CompareOptions::against(&previous))?;
ensure!(comparison.policy().allows_publication());
let built = project.build(BuildOptions::to(&next).update_from(&previous))?;
```

此片段中，`project` 包含修订后的笔记，`previous` 指向版本 1，`next` 是版本 2 的独立路径。[完整示例](https://ankiforge.dev/generated/website_showcase.rs)包含两个项目的定义，并验证保留了一条笔记的身份。

输入包与输出包应使用不同路径。覆盖基线会丢失理解和验证发布所需的证据。

## 保留原始分发包

每个生成包都携带完整的身份和修订证据。下一次构建会将证据与实际数据库和媒体核对后，才复用身份。保留实际分发的原始包；Anki 重新导出的包不是受支持的基线。

[编写指南](https://ankiforge.dev/zh-cn/docs/updates/)说明严格更新、风险策略和客户端导入限制。

## 检查构建与导入

分发前检查构建报告及其更新安全诊断。如果构建时无法确认某条笔记与旧版本的对应关系，请检查源数据和基准包；不能仅凭输出文件存在就认定更新正确。

之后 Anki 会应用自己的导入行为。导入设置、已有笔记内容、本地编辑和 Anki 版本都可能影响结果。稳定身份帮助匹配笔记，但不会覆盖这些选择，也不承诺每次导入都保留本地状态的每一个方面。

[双版本示例](https://ankiforge.dev/zh-cn/examples/#updates)提供两个包，可以先尝试一次小而可检查的更新，再设计自己的发布流程。
