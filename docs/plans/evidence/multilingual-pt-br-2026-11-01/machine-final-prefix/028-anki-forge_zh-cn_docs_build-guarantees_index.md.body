# 构建与输出保证

Canonical page: https://morehardy.github.io/anki-forge/zh-cn/docs/build-guarantees/
Language: zh-CN
Description: 理解临时产物、路径保护和候选版本发布。

`Project::build` 只在操作成功后返回 `BuildOutput`，并保证其中有 APKG 产物。失败时返回 `BuildError`，包含结构化 kind/code、观察结果、底层原因和实际发布事实。

## 临时与持久产物

显式选择所有权模型：

| 请求 | 所有权 |
| --- | --- |
| `BuildOptions::to(path)` | 持久目标；句柄清理后仍保留 |
| `BuildOptions::temporary()` | 临时文件；随最后一个产物所有者删除 |
| `artifact.persist_to(path)` | 新的持久所有者；原所有者仍可使用 |

读取临时路径时，保持 `BuildOutput` 或克隆的产物存活。报告或 JSON 快照不会延长文件寿命。Rust 正常释放所有者；Node 提供 `artifact.close()` 以确定性释放。把路径复制到应用状态中不会创建所有者。

Rust 产物句柄保留绝对路径，即使构建或 `persist_to` 使用相对目标。之后改变进程工作目录不会重定向从已有句柄进行的读取或复制。每个新 `persist_to` 目标均相对于调用时的目录解析。

`BuildReport::snapshot()` 只有观察结果。`BuildOutput::snapshot()` 和 `BuildError::snapshot()` 还说明真实结果。成功快照中也可能有警告。不要从诊断严重程度、路径是否存在或报告是否包含比较发现项来推断结果。

## 基线与发布

更新时保留之前的原始分发 APKG 作为不可变基线，并选择独立输出目标。发布前检查基线证据和候选内容。策略拒绝不会替换输出。缺失或损坏的证据、检查预算耗尽均为硬失败。

发布通过原子替换 APKG 完成。这是单文件保证；写入应用自己的独立 JSON 报告属于另一次操作。普通构建不写额外身份文件：完整的带版本证据嵌入分发包。

发布后仍可能发生晚期错误，例如确认持久性时出错。因此错误快照记录 `PublicationSnapshot` 事实：`path`、`stage`（`not_published` 或 `published`）、`temporary`，以及持久性（`confirmed` 或 `unconfirmed`）。重试或声称没有写入文件前，应检查这些事实。`PersistError` 同样保留发布事实和实际 I/O 源。仅原子替换并不能证明崩溃后的持久性。

快照序列化保留原生路径，不访问文件系统。Unicode 路径仍为 JSON 字符串。非 UTF-8 Unix 路径使用 `{"encoding":"unix_bytes","bytes":[...]}`；含未配对 UTF-16 代理项的 Windows 路径使用 `{"encoding":"windows_wide","units":[...]}`。数组保存精确的原生字节或代码单元。成功产物路径、失败发布路径和绑定错误路径详情均使用此表示。它们描述路径，但不保留产物所有权。文件系统可能拒绝特定原生路径，但其发布失败事实仍可序列化。

## 检查限制

有限的归档大小、条目数量、展开大小、解码集合或媒体及 zstd 窗口预算，分别独立应用于基线与候选包。从 `InspectLimits::default()` 开始，确有需要时再有意提高相关限制。零不是无限制。这些计数器并非进程级内存或 CPU 沙箱；媒体导入另有更早执行的单资源预算。

## 并发工作

Rust 构建借用不可变项目状态。拥有所有权的模型和媒体可跨项目共享。Node 绑定在调用 build 或 compare 时捕获项目快照，之后的添加不会改变该请求。保持产物所有者存活，直到全部读取者结束；另一个消费者仍需路径时关闭最后一个临时所有者，是应用的生命周期错误。

包检查不能证明所有客户端行为。请使用目标 Anki 版本和设置测试渲染、播放与更新导入。调度与 schema 更改限制见[更新](https://morehardy.github.io/anki-forge/zh-cn/docs/updates/)。

## 源码中的预备发布

预备发布已包含在公开 `0.3.1` 包中。

`prepare_publication` 构建、检查并比较一个私有候选包；`publish` 使用同一文件。选项和相对目标在预备阶段绑定。项目编辑不会改变候选包。仅预备不会替换目标；释放或关闭未发布的所有者会移除候选包。

每次发布尝试都会消耗所有者，包括策略或 I/O 失败。重试或改变策略、目标需要重新预备。报告仍可读取，但不拥有文件。Node/Python 提供幂等 `close()`，以及原因是 `closed` 或 `consumed` 的不可用错误；发布开始后关闭不会取消它。耗时不包含等待审阅的时间。成功替换后仍可能发生晚期持久性失败，因此要检查实际发布事实。

完整操作见[示例](https://morehardy.github.io/anki-forge/zh-cn/docs/updates/#build-once-review-then-publish-from-source)，类型化错误和工作线程行为见各语言 API 参考。
