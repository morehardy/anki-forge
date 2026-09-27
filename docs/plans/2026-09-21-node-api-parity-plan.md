# Node 接口缺口清单与 Rust 能力对齐实施方案

> 后续实施状态见 [2026-09-22 实施记录](2026-09-22-node-api-parity-progress.md)。下文保留原始审计基线和方案约定。

- 日期：2026-09-21。
- 状态：已完成源码审计和定向运行验证；本文是待实施方案，不代表下面的新接口已经存在。
- 审计基线：`2edb8cc60e928b0d9cc161a2fcdc01234b722983`；Node SDK 候选版 `0.2.0`，Rust crate `0.1.0`。
- 对照范围：Rust 的 Supported Consumer Interface，以 `prelude`、公开用户文档和可观察行为为准；Node 对照 npm 根入口。
- 与旧计划的关系：补充 [2026-09-06 Node SDK 实施计划](2026-09-06-node-sdk-implementation-plan.md)，保留原有 C01–C18 编号和发布流程。
- 本次交付：缺口、设计决定、文件落点、提交顺序和验收用例。生产实现、版本号和发布状态尚未改变。

## 1. 结论与完成口径

Node 已有原生 Rust `Deck` / `Project`、常规制卡、媒体、模板包、校验、构建、比较、更新安全和完整 JSON 报告。主要工作是补齐对象转换、产物所有权和部分可观察接口，不需要重写 SDK 或改回 CLI。

目前不能宣称完整对齐，原因分为四类：

1. **能力缺失**：无输出路径构建、临时产物句柄、Deck 转 Project、部分读取接口、可变对象克隆和完整 u64 检查预算。
2. **行为差异**：`writeTo()` 先分配完整 APKG Buffer；Rust `write_to()` 分块复制已生成的 APKG。
3. **实现路径与证据不足**：Node Deck 构建经过持久保存的 Project 副本，现有 Rust 对照场景同样先转 Project，不能证明所有 Deck 原生行为与资源特征。
4. **共享缺陷和发布待办**：hide-one 图片遮挡问题属于 Rust 核心；跨平台、最低运行时、Desktop 和 npm 发布证据需要独立验收。

本方案区分三个完成状态：

| 状态 | 必须满足 |
| --- | --- |
| 核心工作流补齐 | N01–N03 完成，真实 APKG、错误报告和资源清理通过验收 |
| 声明范围内的接口对等 | N01–N07 完成；N08 的行为对照完成；S01 范围记录明确；逐项能力映射无未解释缺口 |
| 可发布完整 SDK | 上述对等条件、原计划的共享核心缺陷与 Release Gate 均关闭，并有确切提交对应的证据 |

优先级 P0/P1/P2 表示实施顺序和依赖，不是线上事故严重程度。不能用测试数量、C01–C18 有记录或两个语言同样报错，替代完整性结论。

## 2. 已有能力：避免重复实现

| 既有能力 | Node 当前入口 | 审计结论 |
| --- | --- | --- |
| Deck / Project 构造、稳定 ID、默认牌组 | `new Deck()` / `new Project()` | 已有；对象转换另见 N02 |
| Basic / Cloze / 自定义 Normal 与 Cloze | `Note`、`NoteType` | 已有 |
| 字段、模板、CSS、浏览器模板、目标牌组 | `Field`、`Template`、`NoteType` | 输入能力已有；只读观察另见 N04 |
| 字段身份、单条身份覆盖、生成条件 | `IdentityRecipe`、`GenerationRule`、options | 已有 |
| 文本、HTML、图片和声音 | `Content`、Note 内容方法 | 已有，渲染交由 Rust |
| 文件、字节、大 Buffer、指纹与重名检查 | `media.addFile/addBytes/addBuffer` | 已有；Deck 只提供对应 Rust 的文件/字节语义 |
| 模板目录导入与原子失败 | `importTemplateBundle()` | 已有 |
| 项目校验、模板语义校验 | `validate()`、`validateTemplate()` | 已有；Deck 校验的精确对照见 N08 |
| 文件 / Buffer / Writable 导出 | `build/writeApkg/toApkgBuffer/writeTo` | 已有，产物模式和分块写入仍有缺口 |
| compareTo、failOn、锁文件、更新安全模式 | `BuildOptions` 与便利函数 | 已有 |
| 不发布的候选比较 | `diffAgainstApkg()` | 已有 |
| 11 项检查预算、媒体模式和策略 | `InspectLimits`、`mediaMode/mediaPolicy` | 已有；u64 输入范围见 N07 |
| 完整报告、诊断、风险、更新证据 | `BuildReport.raw` 与类型化属性 | 已有，包含 metrics / policy；不要套用 Python 的报告缺口 |
| Busy / Failed、工作线程、ESM / CJS | 原生状态机、异步任务、生成入口 | 已有，新增能力必须沿用 |

依据：[Node 公共导出](../../bindings/node/src/index.ts)、[构建接口](../../bindings/node/src/buildable.ts)、[类型](../../bindings/node/src/types.ts)、[现有覆盖索引](../../bindings/node/COVERAGE.md)。

以下区别属于语言适配，本身不构成缺口：Rust builder 对应 Node options；FieldKey / TemplateKey 对应经 Rust 检查的字符串；同步 Rust I/O 对应 Promise；大整数报告以精确十进制字符串表达；Node 路径采用构造时捕获的 `baseDir`。不要求机械复制 Rust 的方法数量。

## 3. 缺口总表

| ID | 分类 / 优先级 | Rust 已有能力 | Node 现状 | 实际影响 |
| --- | --- | --- | --- | --- |
| N01 | 功能与所有权 / P0 | 无 output 构建；仅 artifacts_dir 构建；ApkgArtifact clone / persist_to / 生命周期 | TS 与 native 都要求 output；报告只返回路径 JSON | 不能持有完整报告对应的临时 APKG，再决定保存到哪里 |
| N02 | 功能 / P1 | `Project::from(deck)` | 两类对象独立，缺少转换入口 | 已用 Deck 建立的身份、HTML、媒体状态不能整体转入可编辑 Project |
| N03 | 资源行为 / P1 | `Deck::write_to()` 用 64 KiB 缓冲复制成品文件 | `writeTo()` 先调用 `toApkgBuffer()`，一次写完整 Buffer | 大包的传输阶段额外内存随 APKG 大小增长 |
| N04 | 只读观察 / P2 | Field / Template / NoteType / Note / IdentityRecipe 的读取方法，Deck 的 metadata / notes | 多数值对象数据封装在 WeakMap；Deck 仅公开 name 等少量属性 | 通用生成器、调试工具和配置复用无法读取对应 Rust 信息 |
| N05 | 媒体引用查询 / P2 | `deck.media().get(name)` | Deck media 仅有 addFile / addBytes | 丢失引用变量后不能按已注册文件名取回引用 |
| N06 | 可变状态快照 / P2 | Project / Deck 的 Rust Clone | 无公开 clone；SharedProject 的内部 Clone 只是共享同一状态 | 无法从已有项目分出彼此独立的构建变体 |
| N07 | 参数范围 / P2 | InspectLimits 的 11 个 u64 字段 | 只接受非负安全整数 number | 不能精确表达 Rust 接受的全部有限预算值 |
| N08 | 路径差异与验证 / P2 | Rust Deck 自己的 build / validate 路径 | 操作前 clone Deck 为 Project，保存到 context，再执行 Project 操作 | 存在额外复制/驻留；精确 Deck 行为缺少独立证明 |

N04/N06/N07 对普通制卡影响较小，但如果目标是“全部声明范围内的能力”，不能永久省略后仍宣称完全对齐。N08 的额外复制由源码确认；耗时和峰值内存的实际增量尚未测量。

## 4. N01：产物句柄与无输出路径构建

### 4.1 证据与根因

- Rust [BuildOptions](../../anki_forge/src/build/options.rs) 的 output 是 Option；[用户文档](../../anki_forge/README.md#artifact-ownership) 明确支持临时产物与持久化。
- Node [buildable.ts](../../bindings/node/src/buildable.ts) 无条件检查 `config.output`；[native/lib.rs](../../bindings/node/native/src/lib.rs) 的 BuildInput.output 是必填 PathBuf。
- [native/state.rs](../../bindings/node/native/src/state.rs) 将 Rust BuildReport 转为字符串后释放报告；[reports.rs](../../bindings/node/native/src/reports.rs) 只序列化路径。
- 因此，只删 output 的必填检查会产生“Promise 返回时临时文件已经删除”的问题。成功与失败两条返回路径都必须持有原生 ApkgArtifact。

### 4.2 推荐的公开 Interface

以下为拟新增/调整的声明，不是当前可执行接口：

```ts
interface BuildOptions {
  output?: string;                    // 从必填改成可选，其余选项保留
  artifactsDir?: string;
  // 其余当前已有字段保持不变
}

declare class ApkgArtifact {
  readonly path: string;
  clone(): ApkgArtifact;
  persistTo(path: string): Promise<ApkgArtifact>;
  close(): Promise<void>;
}

declare class BuildReport {
  readonly artifact: Readonly<{ path: string }> | null;
  readonly artifactHandle: ApkgArtifact | null;
  // raw、counts、metrics、policy 等既有属性保持不变
}

// 同时适用于 Project 与 Deck：
// build(options?: BuildOptions): Promise<BuildReport>
```

推荐保留 `report.artifact` 和 `report.raw.artifact` 的原有 JSON 路径快照，新增 `artifactHandle`。理由是当前 BuildReport 构造器允许接收原始报告；一个手工构造或反序列化的报告不能凭路径重建原生所有权。

该选择的兼容约定：

- SDK 构建返回的 report / BuildError.report，在 Rust 报告有产物时具有真实 artifactHandle。
- 手工 `new BuildReport(raw, pretty)` 仍可解析报告，但 artifactHandle 为 null。
- 保留路径快照不会保活文件；保留 report 或未关闭的 artifactHandle 才会。
- artifactHandle 属性返回同一个句柄实例，不在每次读取时隐式 clone；显式 close 会释放该实例持有的所有权，需要独立保活的调用方先 clone。
- 日后若希望 `report.artifact` 直接变成句柄，需按公开 Interface 变更单独安排版本迁移，不在本轮静默替换。

目标用法：

```ts
const report = await project.build();
report.ensureSuccess();
const artifact = report.artifactHandle;
if (!artifact) throw new Error('Successful build has no owned artifact');

try {
  // 此时 report.raw、诊断和临时 APKG 都可供调用方检查。
  const saved = await artifact.persistTo('exports/reviewed.apkg');
  await saved.close();  // 释放句柄；持久文件继续存在
} finally {
  await artifact.close();
}
```

### 4.3 生命周期与错误语义

| 行为 | 必须遵守的规则 |
| --- | --- |
| 默认 build | 返回临时 APKG；最后一个原生所有者释放后删除 |
| artifactsDir-only build | 返回调用方管理的持久文件；关闭句柄不删除目录和文件 |
| 显式 output | 保持现有文件发布语义 |
| clone | 增加独立所有者；关闭一个句柄不影响另一个 |
| close | 幂等，释放当前所有者；在途操作已取得的克隆继续保活 |
| persistTo | 调用 Rust persist_to；原子复制，成功返回持久句柄，失败保留原文件 |
| 路径基准 | 句柄捕获来源 Project / Deck 的 baseDir；persistTo 的相对路径不受后来 chdir 影响 |
| 同路径/别名 | 保留 Rust 规则：临时产物不能持久化到自身/别名；持久产物的同文件操作按核心处理 |
| GC | 作为未显式 close 时的回收后备，不承诺发生时间 |
| 关闭后的操作 | clone / persistTo 拒绝明确的 ArtifactClosedError；path 可保留为诊断快照 |
| JSON | 无句柄、无所有权；不能通过 JSON 或任意路径恢复成可删除文件的对象 |
| 错误报告 | late failure 已生成产物时，BuildError.report 仍持有它，不能把错误结果一律清空产物 |

并发 persistTo 的每次任务分别持有 Rust 克隆；close 不取消已经开始的持久化。同一目标路径的并发写入没有新增排序承诺。不要把 persistTo 描述为 compareTo / failOn / 锁文件事务；它只是核心已经提供的产物复制能力。

close 的 Promise 完成表示该句柄的原生所有权已释放；实际删除沿用 Rust ApkgArtifact / TempPath 的清理语义，不新增比核心更强的清理失败报告保证。禁止通过保存路径后直接 unlink 的方式模拟 close。

### 4.4 Implementation 落点

1. 新增 `native/src/artifact.rs`，封装 Rust ApkgArtifact。native 对象持有可释放的 Rust 句柄，不维护第二套临时文件删除规则。
2. 将 build 的任务输出改为有类型的结果，例如内部 `NativeBuildResult { result: string, artifact?: NativeApkgArtifact }`。worker 传递拥有所有权的 Rust 值，在 JS resolve 阶段创建 N-API 包装。
3. success 和 error 使用同一个 build-outcome decoder。修改 `internal/native.ts`、`internal/outcome.ts`、`report.ts`、`errors.ts`；在创建 BuildError 之前附上真实句柄。
4. 项目恢复 Ready 与返回产物所有权相互独立。不要把上一次报告/产物存在 SharedProject 中，否则会无端延长临时文件寿命。
5. 用现有 JsDeferred / spawn_blocking 机制执行 persistTo / close；Promise 提交失败、JS 转换失败、Worker 退出时，任务持有的 Rust 值也必须被释放。
6. 新增 `src/artifact.ts`；构造只允许内部 token/factory，不提供 `new ApkgArtifact(path)`。
7. 完成句柄传递后，再将 TS output 改成 optional、native output 改成 Option，支持 `build()`、`build({})` 和 artifactsDir-only。
8. 对 reportJson 无持久路径的情况继续交给核心产生既有 `PROJECT.REPORT_JSON_WRITE_FAILED`；不要在 JS 新造一套判定。

核心无需为绑定重新发明产物机制。直接复用 [artifact.rs](../../anki_forge/src/build/artifact.rs) 及 [生命周期测试](../../anki_forge/tests/artifact_lifecycle_tests.rs)。

## 5. N02 / N05 / N06：对象转换、媒体查询与克隆

### 5.1 Deck → Project

推荐：

```ts
const project = await Project.fromDeck(deck);
// 原 deck 仍可使用；新 project 可继续添加自定义类型、笔记、模板包和媒体。
```

选择**快照转换**，语义对应 `Project::from(deck.clone())`。它保留原 Deck 可用性，避免给 JS 用户增加“已被移动”的第四种对象状态。暂不增加 `consume` 参数或同时提供第二个转换入口。

| 选择 | 取舍 |
| --- | --- |
| 推荐：异步快照转换 | 复制成本 O(Deck 状态大小)，但不用新增 Moved 状态；符合可复用 JS 对象习惯 |
| 暂不采用：消费式 intoProject | 可减少部分复制，但必须定义原 Deck、媒体 façade 和所有旧引用失效的规则 |

实现规则：

- 调用时立即 reserve Deck；克隆和核心转换在 worker 内执行。
- 返回新的 SharedProject，项目 mutation/index/Busy 状态不能与原 Deck 共用。
- 继承原 Deck 的 name、stable ID、默认牌组、身份快照、HTML、媒体来源及诊断位置，直接使用 Rust 转换。
- 继承 Deck 已捕获的 baseDir；不接受在转换时静默覆盖 stableId/defaultDeck 的参数。
- 转换不构建 APKG、不重新注册媒体、不重新哈希源文件；注册后的源文件变更仍在构建时被原指纹发现。
- 新增模块私有的 native-adoption factory，初始化真正的 Project 实例；不能用 Object.create 绕过带私有字段的构造，也不应先生成再丢弃一个无用的 Rust Project。
- 原 Deck 在成功或普通错误后恢复 Ready；panic 沿用 Failed；环境退出不得泄漏新副本。
- DeckMediaRef 与 Project MediaRef 继续区分类型。已导入笔记中的媒体照常生效，不通过 TS 重建成 Note.basic，也不靠重新注册媒体完成转换。

修改：`src/project.ts`、`src/deck.ts`、`src/internal/native.ts`、`native/src/deck.rs`、`native/src/state.rs`、`native/src/lib.rs`。Rust 对照为 [deck_import.rs](../../anki_forge/src/product/project/deck_import.rs)。

### 5.2 已注册 Deck 媒体查询

新增 `deck.media.get(filename): DeckMediaRef | undefined`，同步调用 Rust `deck.media().get()`。

源码对照：[Rust Deck media](../../anki_forge/src/deck/media.rs)、[Node Deck façade](../../bindings/node/src/deck.ts)。

- 查到时返回仍带 SDK 内部 brand 的引用；查不到返回 undefined。
- 只按注册的导出文件名查询，不读取文件，不重新计算指纹，不新增注册。
- Busy / Failed 行为与已有同步入口一致。
- Project.media.get 不属于现有 Rust Project 的直接对应能力；如果后续需要导入媒体的便捷取引用，可另加小型核心读取入口，不借此暴露整个媒体 IR。

### 5.3 Project / Deck 克隆

新增 `await project.clone()` 与 `await deck.clone()`。

关键区别：当前 `SharedProject.clone()` 克隆 Arc，得到的是**同一个**可变状态；公开 clone 必须深拷贝核心 Project / Deck 并建立新的 SharedProject。

Project 的大 Buffer 依赖 `ProjectContext.staged_media: Vec<TempDir>`，所以不能只调用 Rust Project.clone：

1. 将原生暂存目录的所有者改为可共享的资源持有者，例如 `Arc<TempDir>`。
2. 克隆核心状态、媒体引用索引；共享不可变暂存文件的所有权。
3. 原项目被回收后，克隆项目仍能重复构建；最后一个引用释放后清理暂存目录。
4. 调用方自己的文件仅保留来源与指纹，不改成 SDK 所有。
5. 两个克隆能独立添加内容、并行 build，各自 Busy/Failed 不互相影响。
6. 相同输入的 GUID/config ID/revision 保持一致；克隆本身不能重新推导身份。

Note / Field / Template 等不可变值对象可安全复用，不要求为每一种不可变对象增加没有行为收益的 clone 方法。

源码对照：[Rust Project 的 Clone](../../anki_forge/src/product/project.rs)、[Rust Deck 的 Clone](../../anki_forge/src/deck/model.rs)、[Node SharedProject 与暂存目录](../../bindings/node/native/src/state.rs)。

## 6. N03：让 writeTo 的传输内存有界

当前 Rust [copy_bounded](../../anki_forge/src/deck/export.rs) 使用 64 KiB 缓冲；Node [writeTo](../../bindings/node/src/buildable.ts) 调用 toApkgBuffer，然后 [writeBuffer](../../bindings/node/src/internal/stream.ts) 一次写入整个包。

本轮用 256 KiB 随机 WAV 输入、64 KiB highWaterMark 的 Writable 验证，观察到一次 `318430` 字节的 write 调用。该数据证明当前写入粒度，不是吞吐或峰值 RSS 基准。

推荐在 N01 后实现：

1. 先 `build()`，完成核心构建、检查和相应失败处理。
2. 保持 artifactHandle 存活，从产物路径创建分块可读流；建议固定 64 KiB highWaterMark。
3. 逐块 await 写入回调和必要的 drain，继续复用已有错误/背压语义。
4. 成功后不调用目标 Writable.end()；错误或目标提前关闭时，销毁源流并释放临时句柄。
5. 从构建开始前就监听目标流错误/关闭，保持现有“构建期间关闭”行为；不声称关闭流能够取消 Rust 构建。
6. 用户目标流的错误必须被传播，不能因释放产物又抛错误而掩盖原始失败。

`toApkgBuffer()` 继续明确分配完整 Buffer；不要把这个便利接口也包装成“低内存”。可以用新产物机制统一其构建入口，避免两套默认构建逻辑。

这里实现的是**先生成完整 APKG，再有界传输**。Rust 与 Node 均不因此具备边生成 ZIP 边向网络输出的能力。内存约束仅针对 SDK 的传输缓冲，不承诺约束 Rust 构建或用户 Writable 内部自行积累的数据。

## 7. N04：补齐读取能力，不复制 Rust 规则

推荐给值对象提供一个 `describe()`，返回 deeply frozen、类型明确的普通数据；Deck 的整份 notes 快照采用异步 `describe()`。不新增编辑器、公共 IR 或可直接修改核心状态的可变集合。

| 对象 | 应覆盖的 Rust 观察能力 | 拟返回的信息 |
| --- | --- | --- |
| Field | key_ref / name / is_identity / is_sort / is_required / is_optional / key_auto_derived | 有效 key、名称、全部标志、key 是否自动推导 |
| Template | key / name / front/back/browser source / target deck / generation rule | 有效 key、模板文本、浏览器文本、目标牌组与生成规则 |
| NoteType | id / kind / name / fields / templates / css / identity | 包括 custom Cloze field 的完整只读定义 |
| IdentityRecipe | field_keys | Rust 排序与去重后的字段 keys |
| GenerationRule | enum 可观察值 | kind 与对应字段参数 |
| Note | note_type_id / stable_id / deck_name / tags / identity / rendered_fields | 身份输入与 Rust 渲染后的字段；不伪造 Project 尚未推导的最终 GUID |
| Deck | name / stable_id / identity_policy / notes | 牌组信息和笔记快照，包含核心已经产生的身份信息 |

约束与落点：

- 自动 key、identity 字段排序、HTML escaping 必须通过已有 Rust 构造和读取方法获取，不能在 JS 复刻。
- 复用 `native/src/authoring.rs` 的类型转换，增加内部 projection Module；公开 Interface 不暴露通用 native JSON RPC。
- 不可变值的 describe 不执行 Project.add 校验，不改变原有报错时机。
- Deck describe 需要复制 O(n) 数据，使用已有异步 reserve；Busy 期间不读到部分状态。
- 新增 `src/snapshots.ts` 或同等独立类型文件；DTO 只包含稳定、明确列出的字段，不直接 serde 整个内部对象。
- Note.describe 必须保留 stock 字段的真实名称，例如 Basic 的 Front/Back；不能沿用 Python 文档里的字段拼写做重映射。
- Rust Project 当前没有与 Deck.notes 对应的公开查询接口，不在这一项凭空扩展 Project 查询能力。

字段与模板 key 的新 TypeScript 类型可以继续是 string。使用名义类型或新增 FieldKey class 不会自动补齐上述读取能力。

源码对照：[Rust Field/NoteType](../../anki_forge/src/product/notetype.rs)、[Template](../../anki_forge/src/product/template.rs)、[Note](../../anki_forge/src/product/note.rs)、[IdentityRecipe](../../anki_forge/src/product/identity.rs)；[Node 值对象](../../bindings/node/src/notetype.ts)、[Node Note](../../bindings/node/src/note.ts)。

## 8. N07：InspectLimits 的完整 u64 输入

当前 [InspectLimits](../../anki_forge/src/writer_core/inspect_limits.rs) 为 u64，而 Node [checkInspectLimits](../../bindings/node/src/options.ts) 限制在非负安全整数。输入 `9007199254740992` 会被拒绝；拒绝不安全 number 是正确行为，缺的是精确替代通道。

推荐：

```ts
type InspectBudget = number | bigint;
// InspectLimits 的 11 个可选字段均使用 InspectBudget。
await project.build({
  output: 'deck.apkg',
  inspectLimits: { maxArchiveBytes: 9007199254740993n },
});
```

- number 仍必须是非负安全整数；不能把已经丢失精度的 number 自动转 bigint。
- bigint 必须处于 `0..=18446744073709551615`。
- 内部序列化仅把已知预算字段转换为十进制字符串；不修改全局 BigInt JSON 行为。
- native 明确接受原安全整数形式和新十进制字符串形式，校验后转换 u64；不能先经过 f64。
- build 和 diffAgainstApkg 共用这一转换。
- 当前所有 Rust 默认值均在安全范围内，defaultInspectLimits 继续返回 number 默认值以避免无必要的返回类型破坏；增加默认值安全范围断言，未来默认值变更时显式审查。
- 本项修改需要主包、native 和平台包的版本一致，避免新 JS 参数协议连接到旧 native。

这不是放开无限预算。核心所有有限预算与超限错误仍照常工作。

## 9. N08、S01 与非绑定缺口

### 9.1 Deck 真实入口与资源路径

当前 ProjectTask 在 validate/build/diff 前执行 `context.project = Project::from(deck.clone())`，然后操作 Project。Rust 的 Deck.build 则调用自己的借用转换并使用 consuming build。现有 [Rust 对照 helper](../../bindings/node/native/examples/sdk_parity.rs) 的 deck 场景也返回 Project::from(deck)。

建议：

1. 增加直接调用 Rust Deck.build / Deck.validate_report 的独立对照，不能继续把双方都转 Project 当作 Deck 全部对等的证据。
2. build 对 Deck 分派到真实 `deck.build(options)`，Project 保持 `project.build(options)`，共用 N01 的结果与产物投影。
3. validate 先对照诊断代码、顺序、severity 和映射后的来源；若现有 Node 行为是有意增强的 Project 校验，明确记录范围与迁移，不未经回归直接改变诊断集合。
4. diff 的 Rust Deck 没有独立同名方法，Node 可以继续用短生命周期 Project 快照；不要在 context 长期保存快照。
5. 测量大型 Back 文本、多媒体、连续重复 build 后的活跃原生分配；不能凭进程 RSS 未下降就认定泄漏，也不能先宣称某个加速百分比。

此项可在产物任务输出改造稳定后独立完成；优化结论不能阻止 N01–N07 的功能交付。

### 9.2 S01：Project.lower() 的范围问题

源码中的 `Project.lower()` 是可调用的 public 方法，返回 authoring plan；不能声称它不存在或已被 feature 完全隔离。但 [prelude](../../anki_forge/src/prelude.rs)、[ADR 0012](../adr/0012-narrow-rust-0.1-interface.md) 和 CONTEXT 明确把 normalization IR / tooling 排除在 Supported Consumer Interface 的主要承诺之外。

本方案推荐保持 npm 根入口为产品 Interface，暂不引入 lower / raw writer / contract 加载：

- 在最终能力清单中明确记为“源码可达、产品对等范围外”，不能静默漏掉。
- 在 Rust 用户文档/ADR 中澄清 lower 的兼容承诺；本轮不删除方法、不新增 feature gate，不制造 Rust 破坏性变更。
- 如果后续目标扩大为“Rust 所有 public 方法”，需把 lower 单列为新范围，设计有版本的 advanced DTO；legacy normalize/build 不能直接充当现有 Project 状态的 lower。

### 9.3 共享核心问题和发布证据

| ID | 项目 | 当前判断 | 关闭方式 |
| --- | --- | --- | --- |
| R01 | hide-one-guess-one | Rust renderer 的 grouped c1,2 被 writer 拒绝；Node 保留核心错误 | 在 Rust 修复并验证契约/Anki 语义，再扩充两端成功场景；不在 TS 改写标记 |
| V01 | 行为覆盖 | 现有 C01–C18 与 14 个独立构造场景有价值，但不包含上述全部能力 | 建立新增能力和失败场景的明确映射 |
| V02 | 平台与最低版本 | 四目标 × Node 22/24/26 workflow 已存在；本次未核查远程运行结果 | 对候选提交收集全部 job；补明确的最低 Node 22.13.0 验证，而不只测试滚动的 22 |
| V03 | Desktop | 自动观察 APKG 不等于真实导入、渲染、更新历史验证 | 使用 SDK 产物完成现有 Desktop 清单并记录结果 |
| V04 | 分发与发布 | 仓库文档仍列出发布待办；本次未查询 registry 状态 | 按 RELEASING 验证包名控制权、全部 tarball、真实安装及发布恢复 |

R01 不属于“Node 比 Rust 少能力”，但未修复前不能承诺两个图片遮挡模式都可用。V02–V04 属于验证/分发门槛，不能写成 SDK 功能尚未实现，也不能因 workflow 存在就标记已验证。

## 10. 实施顺序与可合并提交

依赖主线：范围/契约 → 产物句柄 → 无路径构建 → 分块写入。转换/查询、克隆、只读观察和预算范围可作为独立变更，但各自合并前必须带对应行为测试。

| 阶段 | 建议提交 | 交付与文件范围 | 验收后才可关闭 |
| --- | --- | --- | --- |
| A0 | docs: record Node parity scope and cases | 本文；后续将能力矩阵落为 test/capability-matrix.json，关联 C01–C18 与新增条目 | N01–N08、S01、R01/V01–V04 状态可追踪；没有百分比式完整度 |
| A1 | feat(node): preserve native artifact ownership | native/artifact.rs、typed build result、report/outcome/error/native declarations、src/artifact.ts、index.ts | 显式输出路径成功/失败报告持有真实句柄；close/clone/persist 与旧报告接口兼容 |
| A2 | feat(node): allow temporary and artifacts-only builds | types.ts、buildable.ts、native BuildInput，扩充 A1 测试 | 临时文件正确存活/删除；artifactsDir-only；reportJson 和 late failure 规则 |
| A3 | fix(node): bound Writable transfer buffers | buildable.ts、internal/stream.ts；收敛 BytesTask 重复逻辑 | 分块、背压、目标流开放、错误/关闭、临时文件清理 |
| B1 | feat(node): import Deck snapshots into Project | project.ts/deck.ts、private adoption、native conversion task | 身份/HTML/媒体证据保留；继续编辑；原 Deck 可用 |
| B2 | feat(node): retrieve registered Deck media | Deck media façade、native Deck lookup | 同名取回、未知返回 undefined、没有注册副作用 |
| B3 | feat(node): clone mutable authoring state | state.rs 的资源所有权、Project/Deck clone、native task result | 独立状态；大 Buffer 暂存共享寿命；GC/失败不影响另一副本 |
| C1 | feat(node): expose immutable value snapshots | snapshots.ts、note/notetype/deck、native projection | Rust 自动 key/identity/rendered fields 对照；readonly 类型和运行期冻结 |
| C2 | feat(node): accept exact u64 inspection budgets | types/options、native/options、build/diff 编码、数字测试 | safe number / bigint 全边界与实际预算执行 |
| D1 | refactor(node): dispatch Deck builds to Rust Deck | state task 分派、Deck 原生对照和资源测量 | 诊断/产物对等；不再常驻多余 Project 副本；记录实测而非预期收益 |
| D2 | test(node): close parity and installed-consumer coverage | sdk_parity helper、product/parity tests、installed-smoke、COVERAGE、README、ADR 0017、CI | 新旧安装接口、四平台/最低版本、完整场景映射 |

R01 在核心修复路径独立实施。若调整输出语义或契约，应执行仓库的 contract-change-policy；不要为了完成绑定项绕过该流程。

预计规模仅用于排期：N01 最大且是主依赖；转换/克隆次之；流写入、快照和预算为中等；媒体查询较小。跨平台与 Desktop 需要独立验证时间，不以本机完成日期估算发布日期。

## 11. 验收矩阵

测试从公开 Node Interface 构造对象，Rust helper 独立构造对照数据。只归一化测试目录和耗时，不能去掉 GUID、config ID、revision、媒体内容或关键诊断来让结果一致。

| 用例 ID | 覆盖项 | 关键断言 |
| --- | --- | --- |
| A01 | N01 默认构建 | build() 与 build({}) 返回可检查 APKG 和完整报告；没有要求永久输出路径 |
| A02 | N01 clone 寿命 | 关闭原 handle 后 clone 的路径仍可读；最后一个 handle 关闭后临时文件删除 |
| A03 | N01 所有者解耦 | Project 被回收不影响 report；Project 仍存活时关闭产物不会被 project 的隐藏引用阻止清理 |
| A04 | N01 持久输出 | output、artifactsDir-only、persistTo 的文件在所有句柄关闭后仍存在 |
| A05 | N01 持久化失败 | 不可写目标/目标为目录等确定性错误不破坏原产物；原有目标保持完整 |
| A06 | N01 路径规则 | 相同临时路径、symlink/hardlink 别名、chdir、Unicode/空格路径遵循核心与 baseDir 约定 |
| A07 | N01 错误产物 | 临时 APKG 生成后锁文件发布失败，BuildError.report 保活 APKG；关闭后清理 |
| A08 | N01 JSON | 无持久路径的 reportJson 返回核心错误；JSON round-trip 不恢复句柄、不延长文件寿命 |
| A09 | N01 异步退出 | Promise/转换失败、GC、Worker 退出不泄漏句柄、不误删持久文件、不提前释放在途操作 |
| A10 | N01 原有安全行为 | compareTo/failOn、被阻止的输出、旧锁文件、路径碰撞仍保持现有语义 |
| T01 | N03 慢 Writable | 多块写入；等待回调/drain；总字节/哈希正确；目标未 end |
| T02 | N03 故障 | 生成期间关闭、写入中报错/关闭、回调错误均传播；源文件/流/监听器最终清理 |
| T03 | N03 资源特征 | 在专用子进程中分别观察构建与传输阶段；传输缓冲不随包大小线性扩张 |
| D01 | N02 转换 | 混合 Basic/Cloze/IO、HTML、tags、显式和推导身份，与 Rust from(deck.clone()) 全观察一致 |
| D02 | N02 继续编辑 | 转换后加入自定义类型、笔记、模板包、媒体；原 Deck 内容不变 |
| D03 | N02 证据保留 | 注册后源文件改变，再转换和构建，仍返回 MEDIA.SOURCE_CHANGED；不被重新注册掩盖 |
| D04 | N02 时序 | 转换期间原 Deck Busy；普通错误后可复用；新旧对象操作状态独立 |
| M01 | N05 查询 | 已知/未知文件名、丢弃原 JS 引用后重新查询、重复查询和 Busy/Failed |
| C01-new | N06 克隆 | 两副本独立添加并行构建；共享初始身份，新增内容不互相污染 |
| C02-new | N06 暂存寿命 | >64 KiB addBuffer 后克隆，释放原对象，副本反复构建仍成功；最后所有者清理暂存 |
| S01-new | N04 快照 | 自动 key、显式 key、optional/required、模板 browser/target、Cloze kind、identity 去重排序 |
| S02-new | N04 Note/Deck | 文本 escaping、HTML、媒体、tags、deck、已解析身份读取与 Rust 对照；快照修改不能改变对象 |
| U01 | N07 数值 | 0、MAX_SAFE_INTEGER、2^53、2^53+1、u64::MAX 的精确 bigint 路径；负数、小数、溢出拒绝 |
| U02 | N07 实际执行 | 大预算 build/diff 均被 Rust 接收；小预算仍真实触发 INSPECT.RESOURCE_LIMIT_EXCEEDED |
| P01 | N08 Deck 原生 | 独立 Rust Deck build/validate 对照；重复 build 后状态不变；记录副本驻留变化 |
| I01 | 安装与类型 | 真实 tarball 的 ESM/CJS 共用新类；NodeNext/CJS 声明；非法 options/伪造句柄/可变 DTO 的负向类型用例 |
| I02 | 平台 | Windows 句柄关闭后删除、alias/权限；macOS/Linux 同样验证；最低 Node patch 单独执行 |

T03 使用独立测量，不把整进程 RSS 的固定上限作为容易波动的普通单元测试。常规 CI 至少验证多块与背压行为；额外资源证据记录输入、包大小、Node/core 版本和采样阶段。

A05/A07 的故障注入应稳定、可跨平台复现，例如将目标位置预置为目录；不要仅靠 Unix mode bits，在提权用户和 Windows 下可能失效。

新增测试文件可以按行为拆分为 `artifact.test.mjs`、`conversion.test.mjs`、`stream.test.mjs`、`snapshot.test.mjs`。同步更新 `scripts/test.mjs` 的发现逻辑：它目前只运行固定 product/parity 文件，单独新增测试文件不会自动进入 npm test。

## 12. 兼容、文档与验证流程

### 12.1 保持的 Interface 与文档更新

- writeApkg、toApkgBuffer、writeTo 的现有调用继续有效；writeTo 保持目标流开放。
- build 的 output 变为可选；保留现有未知选项拒绝和 baseDir 规则。
- report.artifact/raw 的 JSON 结构保留，新增 handle 不进入核心 JSON。
- 新增类/错误/types 从根入口导出；更新生成入口和真实安装类型测试，确保 ESM/CJS class identity。
- 更新 COVERAGE 的 C15/C18，并增加转换、句柄、查询、克隆、观察与数值边界条目；旧测试失败预期不能假冒新成功路径验收。
- README 说明三种产物模式、路径快照与句柄区别、显式 close、流写入范围、转换快照成本。
- 更新 ADR 0017 中“Deck build 始终转 Project”的实现记录；新增 ADR 记录 artifactHandle 与 fromDeck 的设计取舍。
- 发布前按 RELEASING 确认实际 registry 与版本，不假定候选 0.2.0 已经公开可安装。

### 12.2 实施时执行的命令

从仓库根目录运行，使用满足要求的 Node/npm 和 Rust 工具链：

```sh
npm --prefix bindings/node run build
npm --prefix bindings/node run check
npm --prefix bindings/node test
npm --prefix bindings/node run test:parity
cargo test -p ankiforge --test artifact_lifecycle_tests --locked
cargo test -p anki_forge_node_native --test json_numbers --locked
npm --prefix bindings/node run test:legacy
npm --prefix bindings/node run test:installed
npm --prefix bindings/node run check:package
```

按变更运行新增测试和直接相关的 Rust 测试；实现分阶段通过后，按照 [开发指南](../development.md#verification) 执行 make verify-ci，并收集候选提交的远程检查。若只改文档，验证链接和 diff 即可，不需要重跑完整二进制矩阵。

发布阶段沿用现有四平台产物汇总和 `test:installed -- --all`，不另建平行分发系统。Desktop 和发布动作仍按已有流程逐项取得实际证据。

## 13. 本次审计证据与限制

本会话前一轮已重新构建 native SDK 与 Rust CLI，并在本机 Node 24.19.0 执行：

- Node 产品测试 22/22 通过。
- Rust/Node 一致性与更新测试 6/6 通过，内含 14 个独立产品构造场景。
- Python 的测试结果与本 Node 方案无直接验收关系，未用作 Node 覆盖证明。

本轮追加定向运行观察：

```text
Project.fromDeck: undefined
Project.clone: undefined
Deck.clone: undefined
Deck.media.get: undefined
Deck.stableId: undefined
Deck.notes: undefined
Field.key: undefined
IdentityRecipe.fieldKeys: undefined

build({}):
  TypeError: output must be a string
build({ artifactsDir: ... }):
  TypeError: output must be a string
build({ output: ..., inspectLimits: { maxArchiveBytes: 9007199254740992 } }):
  TypeError: Inspect limits must be non-negative safe integers

writeTo（256 KiB WAV，Writable highWaterMark = 65536）:
  chunk sizes: [318430]
  writableEnded: false
```

上述未暴露结论同时通过公共 exports、TypeScript 实现和 native 实现检查，不仅依赖“猜一个方法名并得到 undefined”。字段名称不同本身不算缺口。

本次没有验证远程 CI、全部目标机、最低 Node 22.13.0、Desktop GUI 或公开 registry；也没有测量 N08 的性能幅度。本文中的拟新增 Interface、用例和提交均待实施。

## 14. 关闭清单

更新于 2026-09-22；勾选项的实现、测试与限制见实施记录。外部发布门槛仍单独开放。

- [x] N01：native 产物所有权、临时/持久构建、错误报告寿命均通过。
- [x] N02：Deck 快照导入 Project，核心身份与媒体证据保留。
- [x] N03：Writable 分块传输、背压、失败清理和目标开放语义通过。
- [x] N04：声明范围内的只读信息可访问，数据由 Rust 产生。
- [x] N05：Deck 媒体按文件名查询可用。
- [x] N06：Project/Deck 独立克隆及大 Buffer 暂存寿命通过。
- [x] N07：全部检查预算接受精确 u64 输入。
- [x] N08：Deck 原生行为对照完成，资源路径的变更有实际证据。
- [x] S01：lower/IR 的范围与兼容承诺在文档中明确。
- [x] V01：能力清单逐项关联公开行为测试，新增文件进入实际测试入口。
- [ ] R01 / V02 / V03 / V04：在宣称“完整 SDK 可发布”前独立关闭。
