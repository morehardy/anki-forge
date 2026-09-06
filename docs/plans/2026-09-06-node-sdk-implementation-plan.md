# Node 产品 SDK 与 npm 分发实施计划

- 日期：2026-09-06
- 状态：实施中；已完成本机可运行候选版，四平台、完整一致性及正式发布仍未完成。勾选项表示已落地的子任务，不表示整阶段验收通过。
- 目标：用户通过 npm 安装后，使用完整的 JavaScript / TypeScript 产品接口；构建与业务规则由现有 Rust 核心执行。
- 完整性基线：当前 Rust 的 Deck、Project 及其可达用户能力，不要求向 Node 暴露全部内部 IR、writer、contract 管理工具。
- 推荐主路线：TypeScript 产品接口 + napi-rs 原生适配层 + anki_forge Rust 核心 + 按平台发布的 npm 预编译包。
- 完成标准：第 11 节全部满足。最小 Basic 示例是中间里程碑，不代表 SDK 已完成。

## 1. 当前基础与待补工作

已经具备的核心包括产品对象、模板语义、媒体规范化、身份推导、APKG 构建、检查、比较、风险分析、身份锁文件和更新安全。Node 当前通过 JSON 和 contract_tools 调用其中一部分。

| 现状 | 实施影响 | 代码依据 |
| --- | --- | --- |
| 评估时 Node 主入口导出执行命令的函数，没有产品对象 | 新建产品层及其类型，不在现有函数上只加别名 | [旧 Node 入口（现移入 legacy）](../../bindings/node/legacy/src/index.js) |
| 评估时 npm 包为 private，默认从 cwd 寻找仓库并运行 Cargo | 建立真正的安装产物与加载器 | [package.json](../../bindings/node/package.json)、[旧运行时发现](../../bindings/node/legacy/src/runtime.js) |
| product-build 复用 Rust Project 构建流程，但参数有限 | 核心可复用；完整 SDK 改由原生适配层承接对象和构建选项 | [CLI 产品构建](../../contract_tools/src/product_build_cmd.rs) |
| Rust 已有添加时校验、媒体注册时的指纹、模板导入状态 | 原生层保留真实 Rust 对象，避免每次 build 都从 JSON 重建而丢失这些语义 | [Project](../../anki_forge/src/product/project.rs)、[媒体注册](../../anki_forge/src/product/media_registry.rs) |
| Rust 已内嵌默认契约，初始化可以复用 | .node 内直接链接核心，不需要用户提供 manifest 或安装 contract_tools | [内嵌契约](../../anki_forge/src/runtime/embedded.rs) |
| 现有 Node 测试 19 项在本次评估中全部通过 | 保留兼容验证，同时新增产品能力与真实安装验证 | [现有测试](../../bindings/node/test) |

本次评估还复现了四个现有问题：validate 实际执行构建并可能写 APKG / 锁文件；productDocument 临时落盘改变相对媒体路径基准；替换可执行文件仍保留 Cargo 参数；协议校验接受错误字段类型。这些需要在旧入口保留期间修复并回归，不能把旧实现直接作为新产品接口。

## 2. 技术路线与职责

### 2.1 为什么推荐原生适配层

先前的 CLI 建议适用于批量提交完整文档。用户现在明确要求完整的 Node 产品接口，需要进一步保留增量对象状态、添加时错误和媒体注册后的变化检查。

| 方案 | 对完整产品接口的影响 | 本计划选择 |
| --- | --- | --- |
| 每次调用启动 CLI，传递完整 JSON | 批量 build 简单；逐条添加容易重复传输、重建状态，添加时检查也需要额外设计 | 保留为仓库内旧工具入口 |
| 常驻 Rust 子进程和请求协议 | 能保留状态；需要维护进程生命周期、句柄、消息协议和崩溃后的状态处理 | 暂不引入 |
| napi-rs 原生适配层 | 可以直接保存 Rust 对象，并将文件与构建操作放入异步任务 | 推荐作为 npm SDK 的主实现 |

这是基于状态和接口复杂度的选择，不是已有性能基准证明 CLI 不够快。P0 必须先验证原生编译、状态持有、异步调用与打包可行性；若出现实质阻碍，更新本节方案后再继续，不并行维护两套产品实现。

Node-API 提供原生扩展接口及 ABI 稳定性；具体 Node-API 级别、工具版本与目标平台仍须自行测试。napi-rs 可生成原生加载器和 TypeScript 声明。[Node-API 官方说明](https://nodejs.org/api/n-api.html)、[napi-rs 入门](https://napi.rs/docs/introduction/getting-started)

### 2.2 三层职责

| 模块 | 负责 | 规则 |
| --- | --- | --- |
| TypeScript 产品模块 | JS 风格构造、参数形状、路径基准、Promise、报告与异常、Node Writable 适配 | 不计算 GUID、config ID、卡片数量、模板生成规则或更新风险 |
| Rust 原生 Adapter | 对象与类型转换、持有核心对象、异步任务、错误和报告投影 | 每个用户操作调用已有 Rust 方法，不复制另一套产品规则 |
| anki_forge 核心 | 产品校验、媒体与模板处理、身份、构建、比较、风险和发布行为 | 保持为语义来源；发现跨语言能力缺口时先补核心复用点 |

新增 bindings/node/native，为 publish = false 的 cdylib workspace member。初期使用 prelude；需要命名完整报告、策略等类型时，可由仓库内原生 Adapter 使用 internal-tools。记录该内部使用关系，不把它变成 Rust 下游的稳定接口。默认 Rust 消费者的可见模块和兼容承诺保持由现有边界测试约束。

N-API 的 FFI 代码放在新 crate。P0 实际检查宏展开对 unsafe lint 的要求：核心继续保持 unsafe_code = forbid；如生成的 FFI 代码需要例外，例外只限定在 Adapter 并写入 ADR，手写 unsafe 必须有明确必要性。不得为了绑定放宽整个 workspace 的核心规则。

P0 同时验证 cargo test --workspace 在没有 Node 宿主的 Rust 测试环境中可运行。纯 cdylib 的 lib test / doctest harness 按实际链接需求配置，FFI 行为在 Node 进程内测试；纯转换逻辑保持可独立测试。不得为了加入 native crate 跳过既有核心测试。

### 2.3 版本轴

- npm 主包版本与平台包版本精确一致，内部 binding 版本与该发行版对应。
- Rust crate 版本、内嵌 contract bundle 版本独立记录，不强行使用同一个版本号。
- 原生模块提供内部 binding metadata：binding 版本、核心版本、契约版本、目标 triple、选用的 Node-API 级别。
- 加载时检测主包与原生包不匹配，返回明确加载错误；构建报告可查询实际版本。
- 新版暂按 0.2.0 系列组织；包名沿用 anki-forge-node。实际 registry 归属与已发布版本在 P0 核实，不假定名称可用。

## 3. 产品接口与完整性范围

### 3.1 目标使用方式

以下为目标接口示例，P0 将其变成类型契约测试，后续阶段逐项实现。

```ts
import { Project, Note } from 'anki-forge-node';

const project = new Project('Spanish', {
  stableId: 'spanish-a1',
  defaultDeck: 'Spanish::A1',
  baseDir: process.cwd(),
});

project.addNote(Note.basic('hola', 'hello', { stableId: 'es:hola' }));

const validation = await project.validate();
validation.ensureSuccess();

const report = await project.writeApkg('spanish.apkg');
report.ensureSuccess();
```

自定义模板和媒体的目标用法：

```ts
import { Project, Note, NoteType, Field, Template, GenerationRule } from 'anki-forge-node';

const project = new Project('Vocabulary', { stableId: 'vocabulary' });
const vocabulary = NoteType.custom('vocabulary-card', {
  name: 'Vocabulary Card',
  fields: [
    new Field('Expression', { key: 'expr', identity: true, required: true }),
    new Field('Meaning', { key: 'meaning' }),
    new Field('Audio', { key: 'audio' }),
  ],
  templates: [new Template('Recognition', {
    key: 'recognition',
    front: '{{Expression}} {{Audio}}',
    back: '{{FrontSide}}<hr id="answer">{{Meaning}}',
    generateWhen: GenerationRule.all(['expr']),
  })],
});
project.addNoteType(vocabulary);

const sound = await project.media.addFile('./hola.wav', { exportAs: 'hola.wav' });
project.addNote(
  Note.custom('vocabulary-card', { stableId: 'es:hola' })
    .text('expr', 'hola')
    .text('meaning', 'hello')
    .sound('audio', sound),
);
const report = await project.build({
  output: './v2.apkg',
  compareTo: './v1.apkg',
  failOn: 'high',
});
report.ensureSuccess();
```

### 3.2 能力矩阵

标为首版必需的行都属于本计划完成条件。按用户可观察行为对齐 Rust，命名和异步形式按 Node 习惯设计。

| ID | 首版必需能力 | Node 目标入口 | Rust 对照 |
| --- | --- | --- | --- |
| C01 | 简单单牌组、名称与稳定身份 | Deck 构造与 basic / cloze / imageOcclusion 辅助入口 | Deck 及 lanes |
| C02 | 长期项目、默认牌组、稳定身份 | Project 构造选项 | Project::new / stable_id / default_deck |
| C03 | Basic 与 Cloze，额外字段、标签、指定牌组 | Note.basic / cloze，链式内容方法 | product::Note |
| C04 | 自定义 Normal 与 Cloze note type | NoteType.custom / customCloze | NoteType |
| C05 | 字段 key、身份字段、排序字段、必填/可选字段 | Field，IdentityRecipe | Field / FieldKey / IdentityRecipe |
| C06 | 模板 key、前后面、浏览器内容、目标牌组、CSS | Template / NoteType | Template / TemplateKey |
| C07 | Anki 默认、all、any、cloze 生成规则 | GenerationRule | GenerationRule |
| C08 | 显式身份、字段推导、Deck 的身份选择与覆盖 | stableId / identity 选项 | 既有 Project 与 Deck identity 行为 |
| C09 | 文本转义、原样 HTML、图片和声音内容 | Content，Note.text / html / image / sound | Content / Note |
| C10 | 文件、字节输入、文件名冲突、媒体引用 | media.addFile / addBytes / addBuffer，MediaRef | Rust 媒体注册与诊断 |
| C11 | 图片遮挡模式、矩形、标题与背面补充 | Note.imageOcclusion 与 Deck 遮挡入口 | 两条现有 image occlusion 路径 |
| C12 | 模板目录和配套资源导入 | project.importTemplateBundle | Project::import_template_bundle |
| C13 | 添加时校验与失败后对象不变 | addNote / addNoteType | ProjectAddError 及 Deck 添加行为 |
| C14 | 聚合项目校验、独立模板语义校验 | validate / validateTemplate | Project::validate、核心模板检查 |
| C15 | 文件、Buffer、Writable 输出 | build / writeApkg / toApkgBuffer / writeTo | Project / Deck 构建与 Deck 字节导出 |
| C16 | APKG 基线、锁文件、风险阈值与模式 | BuildOptions 与 update-safe 便利方法 | BuildOptions / update_safety |
| C17 | 当前项目与 APKG 比较，无发布副作用 | diffAgainstApkg | Project::diff_against_apkg_with_limits |
| C18 | 完整构建配置、报告与错误 | 类型化 options / reports / errors | build、diagnostics 与 InspectLimits |

C09 的内容组合能力只覆盖 Rust 当前可表达的组合，不新增富文本编辑器。C11 按 Deck 与 Project 各自的现有规则验收，不把两条路径的差异擅自统一。C17 与旧的两个 inspect JSON 文件 diff 是不同用户操作。

### 3.3 状态与异步约定

1. 值对象采用不可变构造方式，修改返回新值；添加到 Project 后持有 Rust 拥有的快照。后续修改调用方对象或数组不得改变已添加内容。
2. addNote / addNoteType 同步调用 Rust 添加逻辑，立即保留已有的错误优先级及不修改项目的行为。构造对象的 JS 类型检查不代替 Rust 业务校验。
3. 文件读取、媒体指纹计算、模板目录导入、全项目校验、构建和比较返回 Promise，通过原生异步任务执行。Task.compute 中仅操作 Rust 拥有的数据，不访问 JS Env 或悬空的 JS Buffer。[napi-rs AsyncTask](https://napi.rs/docs/concepts/async-task)
4. 同一 Project / Deck 在异步操作开始排队时即进入 Busy 状态。并发修改或第二次异步操作返回 ProjectBusyError；不同实例可以独立运行。首版不做隐式排队或并发修改合并。
5. Adapter 在提交异步任务时转移项目状态，避免在 JS 线程上复制整份大型项目；任务结束后归还状态。任务创建/提交失败、业务错误和返回值转换失败都需要释放 Busy 状态。业务失败后仍可继续修改项目；panic 在 FFI 边界受控处理，不可恢复状态标记为不可继续使用，不让 unwind 穿过 FFI。
6. 任务中的 Rust 引用由 Arc 等拥有关系保活。测试 GC、错误完成和环境退出，不依赖 JS finalizer 提供业务正确性。
7. 原生构建当前没有可靠的执行中取消点，首版不承诺 AbortSignal / timeout 可以终止已开始的构建或阻止发布。不能用 Promise.race 包装出虚假的取消保证；需要取消时另行设计核心协作取消和发布检查点。

这里的异步安排是绑定层设计。napi-rs 的任务模型可以把计算放到工作线程，但需要 Adapter 自己实现状态归还和资源所有权，不能把框架的 Promise 支持当作这些行为已经具备。[napi-rs 任务与清理方法](https://napi.rs/docs/concepts/async-task)

### 3.4 校验、错误和报告

| 操作 | 正常返回 | 业务失败 |
| --- | --- | --- |
| addNote / addNoteType | void | 同步抛 ProjectAddError，保留核心 code 与 diagnostic |
| media 注册、模板包导入 | MediaRef / 导入结果 | Promise 拒绝，错误携带核心诊断，已接收状态保持原子性 |
| validate / validateTemplate | ValidationReport | 校验问题放在报告中；ensureSuccess() 抛 ValidationError |
| build / writeApkg | BuildReport | 核心 BuildError 映射为 Node BuildError，保留完整 report 和 cause |
| diffAgainstApkg | ProjectDiffReport | 对应比较错误，保留 report |
| 加载原生包、类型转换等 | 相应结果 | NativeLoadError / BindingProtocolError / TypeError，和业务错误区分 |

BuildReport 必须完整覆盖 status、comparison、artifact、counts、media、diagnostics、metrics、policy、inspect、diff、risk、updateSafety，不通过 helper 的几个布尔字段替代完整状态。保留 warningCount、diagnosticCodes、prettyReport、ensureSuccess。底层 Adapter 可使用内部 outcome union 传递核心 Result，由 TS 一处转换为公开异常。

领域 code 沿用 Rust 原值；绑定错误采用独立命名空间和类型。完整保存 source、help 以及核心确实提供的位置信息，不编造缺失的行号或阶段。

所有可能超过 JavaScript 安全整数范围的 Anki ID / config ID 在公开报告中使用十进制字符串；普通计数和限制参数检查安全整数范围。不能先转成 JS number 再转字符串。P2 加入超过 2^53 的往返验收。

validate 不接收输出路径、报告写入路径或写锁文件选项，不构建 APKG；它只声明 Rust 同等校验范围。媒体存在性、规范化、最终卡片生成与基线可读性等属于 build 的检查，不宣称 validate 通过即保证可构建。validateTemplate 使用 Rust 模板语义检查，并说明仅覆盖模板范围。

### 3.5 构建选项与输出

以下构建设置都需要类型映射与行为验收，不能静默忽略未知选项：

| Node 选项 | 核心行为 |
| --- | --- |
| output / artifactsDir | 产物与 staging 位置 |
| inspect | 是否执行当前产物检查 |
| inspectLimits | 当前和基线共用有限的 APKG 检查预算 |
| mediaMode | path-backed 或 self-contained |
| mediaPolicy | 默认 strict，以及既有 unused / unknown MIME / declared MIME mismatch 配置 |
| mediaStoreDir | 显式媒体存储位置 |
| compareTo / failOn | APKG 基线与风险阈值 |
| identityLockfile / writeIdentityLockfile / updateSafety | 既有锁文件与更新安全模式 |
| reportJson | 按 Rust 现有报告写入与路径冲突规则执行 |

inspectLimits 完整映射 maxArchiveBytes、maxEntries、maxCentralDirectoryBytes、maxZipEntryBytes、maxZipTotalBytes、maxMetaBytes、maxMediaMapBytes、maxCollectionBytes、maxMediaBytes、maxDecodedTotalBytes、maxZstdWindowBytes。默认值从核心取得，避免 TS 再维护一份数值表。

提供 firstUpdateSafeBuild(lockfile) / updateSafe(lockfile) 等便利构造，严格映射现有 BuildOptions 行为。BuildOptions 的 output 与 writeApkg(path) 不允许冲突。

toApkgBuffer 首先对齐 Rust Deck 字节导出的默认首发语义，可在 Project 上提供同样便利操作。内部保留临时文件直到 Buffer 完成读取，再清理。该便利入口不接受写身份锁文件或声明已发布产物的选项。

writeTo(writable) 基于完整 Buffer 写入，尊重背压、传播流错误，默认不结束调用方持有的流。文档明确这不是增量生成 APKG；若日后需要真正流式构建，应先补 Rust writer 能力。

## 4. 路径、媒体和模板资源

### 4.1 路径基准

- Project / Deck 构造时捕获 baseDir，默认捕获当时的 process.cwd()。后续 chdir 不改变已有对象的路径含义。
- 媒体、模板目录、输出、基线、锁文件和报告等相对路径均按这个显式基准转换，再交由 Rust 处理。
- 不把基线的 symlink / hardlink 检查改成 JS 字符串比较；继续使用 Rust 的实际路径与发布检查。
- 私有临时目录只承载内部资源，不改变用户输入的相对路径基准。
- 主包、平台包和 node_modules 可为只读；所有临时文件放在系统或用户指定的可写位置。

### 4.2 文件和 Buffer

- addFile 调用 Rust 注册，保留注册时指纹；构建前文件被改动时继续产生 MEDIA.SOURCE_CHANGED。
- addBytes 对齐 Rust 当前内联输入限制（现有实现为 64 KiB），保持对应错误；阈值由核心提供。
- addBuffer 是 Node 便利入口：小数据使用已有字节路径，大数据写入 Adapter 拥有的临时媒体存储，再通过 Rust 文件注册。大数据分流行为和资源占用要写入文档。
- 异步接收 Buffer / Uint8Array 时先确定拥有关系，调用方后续改动原 Buffer 不影响已提交内容；不能把 JS 指针保存到 Rust 工作线程。
- 同名同内容、同名不同内容、非法文件名、空资源、MIME 不匹配与缺失引用全部复用核心诊断。
- MediaRef 的等价语义遵循 Rust 的导出文件名语义。不得额外用 JS 对象身份或 project token 拒绝核心允许的引用；跨项目缺失媒体仍由核心检测。
- 模板目录导入直接复用 Project::import_template_bundle 的验证和原子注册。失败时不留下半个 note type 或部分媒体绑定。

## 5. npm 产品交付

### 5.1 包结构

推荐根包携带产品 JS、声明、加载器、README 和许可证；各平台包携带一个 .node 产物。原生模块静态链接核心和其已有内嵌契约资源。

平台包使用与主包完全一致的版本，由 optionalDependencies 和 os / cpu / libc 元数据进行选择。用户机器不需要编译器或安装时下载脚本。[napi-rs 分发模型](https://napi.rs/docs/deep-dive/release)

| npm 平台包后缀 | Rust target | 首版支持 |
| --- | --- | --- |
| darwin-arm64 | aarch64-apple-darwin | 必需 |
| darwin-x64 | x86_64-apple-darwin | 必需 |
| win32-x64-msvc | x86_64-pc-windows-msvc | 必需 |
| linux-x64-gnu | x86_64-unknown-linux-gnu | 必需 |

该范围对齐现有 [Rust Tier 1 平台](../adr/0009-rust-tier-1-platforms.md)。Linux ARM64、musl / Alpine、Windows ARM64 作为明确的后续扩展，不用通用 Linux 支持措辞掩盖缺失目标。P0 在每个目标确认动态库依赖与最低系统要求；Linux 使用固定的兼容基线构建，并在声明的最老 glibc 环境运行安装测试。

Node 22 与 24 为首版必需测试线，Node 26 增加兼容 smoke；发布前重新核对支持状态与确切最低 patch。选择满足功能需要的最低 Node-API 级别，P0 编译验证后固定依赖与工具版本。截至本计划日期，官方页面列出 22 / 24 为 LTS、26 为 Current。[Node 发布状态](https://nodejs.org/en/about/previous-releases)

### 5.2 JS 模块与安装行为

- 支持 ESM import 与 CommonJS require。采用单一 CJS 实现和薄 ESM 具名导出层，避免两份类和原生加载状态。
- TypeScript 源码生成声明；分别验证 NodeNext、CommonJS 和现代 bundler 的类型解析。生成的原生声明仅供 Adapter 校验，公开产品声明不泄露 NativeProject 等内部名称。
- package.json 明确 files、exports、types、engines、license、repository 和发布元数据。构建脚本只在维护端执行，消费端安装不触发 Cargo。
- native loader 从已安装的匹配平台依赖加载，校验 binding 版本。开发构建使用明确的本地路径配置，发行模式不回退到 cwd / PATH 中碰巧存在的二进制。
- npm 可省略 optionalDependencies，SDK 必须检测缺失并提供可操作的安装错误；不能把缺失原生包解释为能力降级或静默执行 Cargo。[npm optionalDependencies 语义](https://docs.npmjs.com/cli/v11/configuring-npm/package-json/#optionaldependencies)
- 验证 npm install、npm ci、生产依赖安装、安装脚本禁用、干净 npm cache、从真实打包产物安装以及只读包目录。
- pnpm / Yarn 运行一次安装 smoke；首版支持承诺以 npm 为基准，遇到不同加载行为时明确处理，不依赖扁平 node_modules。
- 支持直接 Node 运行；浏览器、Edge Runtime 不在首版范围。服务器打包场景文档说明保留平台依赖和将原生模块 external 的方式。

### 5.3 发布顺序与恢复

构建并测试四个平台的不可变产物，先发布平台包，再发布引用其精确版本的主包，最后从 registry 安装主包验证完整链路。主包的正式 dist-tag 只在所需平台包全部可安装之后更新；先使用预发布版本及 next 验收。

多包发布可能部分成功。工作流必须核对已有同版本产物，禁止覆盖或将不完整主包提升为 latest；出现产物不一致时发布新版本。记录产物哈希、源提交、binding/core/bundle 版本和安装验证结果，恢复步骤写入 Node 发布手册。

持续发布优先使用 npm trusted publishing；包名归属、各包的发布者配置与首次发布准备是发布阶段的外部依赖，本地 tarball 验收不依赖这些设置。[npm trusted publishing](https://docs.npmjs.com/trusted-publishers/)

## 6. 文件与模块落点

以下路径为实施时新增或修改的目标，不表示现在已经存在。

| 路径 | 职责 |
| --- | --- |
| bindings/node/src/index.ts | 产品入口 |
| bindings/node/src/project.ts、deck.ts | 用户项目与单牌组接口 |
| bindings/node/src/note.ts、notetype.ts、content.ts | 值对象、构造与内容接口 |
| bindings/node/src/media.ts、template-bundle.ts | 媒体与模板目录操作 |
| bindings/node/src/options.ts、report.ts、errors.ts | 完整选项、报告和异常 |
| bindings/node/src/internal/native.ts、paths.ts | 原生加载、内部返回值转换与路径基准 |
| bindings/node/native/Cargo.toml、build.rs | cdylib 与 napi-rs 构建配置 |
| bindings/node/native/src/lib.rs、project.rs、deck.rs | 原生注册与 Rust 对象持有 |
| bindings/node/native/src/values.rs、media.rs、templates.rs | 值转换、媒体和模板 Adapter |
| bindings/node/native/src/tasks.rs、reports.rs、errors.rs | 异步状态与报告投影 |
| bindings/node/npm/* | 生成的平台包清单与构建产物，不手工维护重复版本 |
| bindings/node/test/product/、types/、installed/ | 产品、类型与真实安装测试 |
| bindings/node/test/capability-matrix.json | C01—C18 到场景和断言的映射 |
| bindings/node/scripts/* | 构建、平台打包、安装 smoke、版本核对 |
| bindings/node/legacy/* | 原 CLI wrapper 的私有兼容位置，仍运行既有测试 |
| docs/node/quick-start.md、api.md、migration.md、release-runbook.md | 用户和维护文档 |
| .github/workflows/node-sdk-ci.yml、node-sdk-release.yml | 新 SDK 验证与发行 |
| Cargo.toml、Cargo.lock、deny.toml、scripts/verify-ci.sh | workspace、依赖规则与验证集成 |

旧 wrapper 当前为私有包。先迁移到 legacy 并修正仓库引用；新 npm 主入口只呈现产品接口。既有低层自动化的迁移方式写入 migration 文档，legacy 不进入默认产品发布文件清单。Python 包及其 CLI 协议继续有原来的回归测试。

核心侧只补确实缺少的复用入口，例如取得媒体阈值、输出完整报告投影或独立模板验证。此类方法先限制为仓库内部能力，不为绕过 Rust 可见性而整体公开 writer / IR。

## 7. 分阶段实施与提交拆分

每阶段交付一个可验证结果。下列提交是建议粒度，同一阶段可拆为多个小 PR；后续阶段不重复实现前一阶段的规则。

### P0：固定范围并验证原生桥接

**依赖：** 无。

**任务与提交：**

- [x] P0.1 记录技术路线 ADR，冻结 C01—C18、示例签名、同步/异步和错误约定。
- [ ] P0.2 建立最小 native crate，验证 Rust 1.92、napi-rs、核心依赖和四目标的编译；固定 Node-API 级别与版本。
- [x] P0.3 验证持有真实 Rust Project、同步添加错误、异步构建、错误后状态恢复和嵌入式契约初始化。
- [ ] P0.4 审查新依赖、workspace feature 合并、生成代码 lint 及动态库要求；确认 npm 名称和版本现状。

**验收：** 本机可加载 .node 并生成有效 APKG；四目标构建 spike 可运行；添加时错误与 Rust 一致；长构建期间 Node 定时器仍能运行。将失败、限制和确定的依赖版本写回 ADR。

### P1：从安装包跑通最小产品流程

**依赖：** P0。

**任务与提交：**

- [x] P1.1 迁移旧 wrapper 到 legacy，修复四个已复现问题并保留回归。
- [x] P1.2 实现 Project、Note.basic、writeApkg 的第一条产品链路。
- [x] P1.3 实现主包、当前平台包、生成加载器、ESM / CJS 入口、类型声明与精确版本检查。
- [x] P1.4 用临时本地 registry 发布打包产物；在仓库外的新目录中只安装主包并构建 Basic APKG。

**验收：** 消费者环境找不到 Cargo / rustc / contract_tools，也没有仓库 contracts 目录；npm 安装后 ESM、CJS 和 TypeScript 示例成功。测试不能用 npm link 或预先手装平台包绕过自动依赖选择。

### P2：完整报告、错误与异步状态

**依赖：** P1。

**任务与提交：**

- [x] P2.1 实现核心报告到绑定 DTO 的完整字段映射、十进制 ID、版本信息和参数范围检查。
- [x] P2.2 实现公开错误层次、report 保留、ensureSuccess / prettyReport / diagnosticCodes。
- [x] P2.3 实现 Ready / Busy / Failed 状态、任务提交时状态转移、业务错误恢复和 GC 保活。
- [x] P2.4 加入异步事件循环、并发实例、同实例拒绝并发、超过 2^53 的 ID 和部分报告测试。

**验收：** blocked、invalid、error 不会被包装成成功；无报告字段丢失；没有 JS 主线程等待长期持锁；业务失败后项目仍可使用。

### P3：完成笔记、字段、模板与身份产品对象

**依赖：** P2。

**任务与提交：**

- [x] P3.1 实现 Field、Template、GenerationRule、IdentityRecipe 和 Normal / Cloze NoteType。
- [x] P3.2 实现 Note.custom / cloze、文本、HTML、标签、指定牌组和身份覆盖。
- [x] P3.3 接入 Rust add_note / add_notetype 的完整添加时校验，并保留错误优先级。
- [x] P3.4 完成不可变值对象、添加后快照语义和 TypeScript 正反例。

**验收：** Basic、Cloze、Custom Normal、Custom Cloze 均由产品对象生成正确卡片；字段 key / 名称、template key / 顺序、生成规则与身份行为对齐 Rust；失败添加不影响已有内容。

### P4：媒体注册与内容

**依赖：** P3。

**任务与提交：**

- [x] P4.1 实现稳定 baseDir 和文件路径转换，接入 addFile / MediaRef。
- [x] P4.2 实现 addBytes、addBuffer、临时媒体存储和异步 Buffer 所有权。
- [x] P4.3 接入 image / sound 与内容组合、MIME / 文件名 / 引用诊断。
- [x] P4.4 覆盖注册后源文件变化、重复注册、跨项目引用、chdir、Unicode 和只读包目录。

**验收：** 图片、音频、视频等核心已有媒体类型能进入 APKG；大 Buffer 的处理没有悬空内存或提前删除临时文件；保留注册时指纹检查；引用与核心一致。

### P5：Deck、图片遮挡与模板目录

**依赖：** P4。

**任务与提交：**

- [x] P5.1 实现 NativeDeck / Deck 便利接口，直接复用 Deck 核心默认值和身份策略。
- [ ] P5.2 实现 Project 与 Deck 图片遮挡接口，包含 modes、rects、header、backExtra、comments。
- [x] P5.3 接入模板目录导入，保留资源路径、模板错误位置与原子性。
- [ ] P5.4 完成 Deck / Project / 模板包 / 产品文档路径之间的语义对照。

**验收：** 简单牌组和图片遮挡真实可导入；空遮罩、重复矩形、越界和缺失图像等现有错误保持；模板导入失败不产生部分状态。

### P6：独立校验、完整构建配置与多种输出

**依赖：** P5。

**任务与提交：**

- [x] P6.1 实现独立 validate / validateTemplate 及其明确的检查范围。
- [x] P6.2 完成第 3.5 节全部 BuildOptions、媒体策略与 11 项 InspectLimits 映射。
- [x] P6.3 实现 toApkgBuffer / writeTo 的临时资源管理、背压与错误传播。
- [x] P6.4 加入校验无发布副作用、inspect=false、self-contained、预算触发和 Buffer / 流输出测试。

**验收：** validate 不创建 APKG、不修改基线与锁文件；每个公开构建选项有至少一个可观察断言；字节输出可由检查器读取；失败时临时产物按约定清理。

### P7：更新安全、差异与发布保护

**依赖：** P6。

**任务与提交：**

- [x] P7.1 完成 compareTo、failOn、锁文件、三种更新安全模式和便利构造。
- [x] P7.2 实现 diffAgainstApkg 与完整风险 / comparison / updateSafety 报告。
- [x] P7.3 覆盖 unchanged、answer-only、tags、content revert 的 revision 行为，以及字段重命名和模板重排。
- [x] P7.4 覆盖基线损坏、旧锁文件、GUID 保留、symlink / hardlink 路径冲突和被策略阻止的构建。本机 Unix 验证通过，Windows symlink/ACL 单独验收。

**验收：** 相同基线与变更在 Node / Rust 得到一致身份、revision、风险和状态；被阻止的构建不覆盖现有输出、基线或锁文件；只比较不推进发布状态。

### P8：四平台完整安装验证

**依赖：** P1 打包基础；P7 功能完整。

**任务与提交：**

- [ ] P8.1 固定四目标构建环境、平台包生成和 native metadata 验证。
- [ ] P8.2 建立主包及全部平台包进入临时 registry 的安装链路，使用 npm 真实选择平台依赖。
- [ ] P8.3 覆盖 Node 22 / 24、Node 26 smoke、ESM / CJS、npm ci、生产安装、禁用脚本与只读包目录。
- [ ] P8.4 验证缺失平台包、不支持的平台、版本不匹配、动态库缺失的错误与安装指引。

**验收：** 四个平台均从安装后的主包完成 Basic、媒体、自定义模板、遮挡和更新安全 smoke；所有支持平台的最低系统要求已记录并通过测试。

### P9：一致性矩阵、文档与 CI 集成

**依赖：** P3—P8；测试与文档随各阶段逐步加入，最终在此收口。

**任务与提交：**

- [ ] P9.1 完成 C01—C18 的场景映射，每个场景独立用 Rust 与 Node 产品接口构造数据。
- [x] P9.2 接入 APKG 观察器，比较语义结果、完整诊断和更新证据。14 个独立 Rust/Node 场景通过，覆盖索引记录剩余边界。
- [x] P9.3 集成快速检查、完整矩阵、发行安装检查；保持 Rust crate 与 Python 既有验证有效。脚本已接入；远程矩阵结果仍计入 P8。
- [x] P9.4 完成 quick start、API、媒体、模板、更新安全、错误处理、迁移和发布手册；README 三个完整 JavaScript 示例进入安装测试。
- [ ] P9.5 用 Node 产品入口生成 Anki Desktop 场景包与检查记录，复核模板、媒体、遮挡和升级导入行为。

**验收：** 能力矩阵无首版待办；支持声明与 CI 实测一致；文档示例从安装包执行，不引用仓库 fixtures 或内部导出。

### P10：预发布与正式 npm 交付

**依赖：** P9 通过，registry 归属和发布身份已配置。

**任务与提交：**

- [ ] P10.1 生成完整 release evidence，确认主包和平台包文件清单、版本、哈希与许可证。
- [ ] P10.2 发布预发布平台包和主包到 next，并从公开 registry 在四目标安装验证。
- [ ] P10.3 修复预发布发现的问题，重跑受影响矩阵；执行一次部分发布恢复演练。
- [ ] P10.4 发布正式平台包，再发布主包；安装验证后更新正式入口和 release notes。

**验收：** 用户只执行 npm install anki-forge-node，即可运行产品示例；registry 版本、平台包、文档和能力矩阵对应同一发行版。

## 8. 必须具备的测试证据

### 8.1 产品与核心一致性

| 组别 | 必需场景 | 核心断言 |
| --- | --- | --- |
| 创建 | Basic、Cloze、多牌组、Custom Normal / Cloze | notes / cards / notetypes / templates / deck 分配 |
| 身份 | 显式 stable ID、字段推导、空值、重复、覆盖 | 相同 GUID / identity evidence；错误发生阶段与 code |
| 添加 | 未知 note type / field、缺失身份、非法模板 | 错误优先级；失败前后项目内容一致 |
| 媒体 | 文件、字节、Buffer、音频/图片/视频、重复导出名 | 内容哈希、媒体映射、引用与 MIME 诊断 |
| 媒体状态 | 注册后替换文件、异步提交后修改 Buffer、GC | 指纹变化被发现；内存输入快照正确 |
| 模板 | 内联与目录、browser front/back、target deck、CSS | 正文和卡片生成一致；失败导入不提交部分状态 |
| 遮挡 | 两模式、多矩形、标题、附注、非法矩形 | 卡片与字段语义；现有错误诊断 |
| 校验 | 合法 / 非法项目、独立模板检查 | 不写 APKG / 锁文件；与 Rust validate 范围相同 |
| 输出 | 文件、Buffer、Writable | 有效 APKG；背压、写错误和临时资源寿命 |
| 更新 | 新笔记、内容/背面/标签修改、回退、旧锁文件 | GUID、mtime/revision、风险和锁文件证据 |
| 保护 | baseline 与 output/report/lockfile 路径冲突、策略阻止 | 基线、旧输出、锁文件字节保持；错误报告完整 |
| 资源 | 各项检查预算、同实例并发、不同实例并发 | 有限检查；没有冻结事件循环或污染对象状态 |
| 数据边界 | 64 位 ID、无效数字、unknown 字段、特殊字符 | 无精度损失、无静默选项丢弃、诊断位置可用 |

Rust 与 Node 必须各自经过用户产品接口构造项目；两边都喂同一份 ProductDocument JSON 不能证明绑定对象层正确。使用既有 APKG inspect 作为观察器，Rust 侧复用 [用户能力矩阵](../../anki_forge/tests/rust_user_capability_matrix.rs) 和 [模板入口一致性测试](../../anki_forge/tests/custom_template_entry_parity_tests.rs) 的观察方法。

比较语义投影，而非直接要求 ZIP 字节一致。只归一化明确非语义字段，如耗时和临时目录；不能忽略 GUID、config ID、revision 或媒体哈希来使测试通过。

APKG inspect 通过只证明所观察的结构与语义，不能独自证明 Anki Desktop 中的渲染和升级结果。预发布复用现有 [Desktop 场景](../manual-validation/anki-desktop-v1) 的检查方式，至少覆盖 Basic、Cloze、自定义 Normal / Cloze、媒体、图片遮挡和更新导入。生成包与检查记录进入发行证据；有条件时补充既有 roundtrip oracle，不新增 GUI 自动化作为本 SDK 的实现前提。

### 8.2 安装验收方法

1. 在构建环境生成主包与所有平台包的 tarball，检查 npm pack 文件清单。
2. 将这些不可变 tarball 放入临时 registry，复制同一批产物给各消费环境。
3. 消费环境使用新的 npm cache / cwd，移除开发覆盖变量和 Cargo 等工具路径，仅通过主包名称和版本安装。
4. 运行 JS、CJS、TS 示例；在仓库不存在、路径含空格及中文、node_modules 只读的条件下执行。
5. 做实际 npm ci 的 lockfile 重装，验证精确平台依赖；特意 omit optional 时确认明确的加载错误。
6. 正式发布前后用真实 registry 重跑同样的消费 smoke。不能把本地 workspace 依赖测试当成安装完成。

### 8.3 检查分层

- 每次相关提交：TypeScript 编译与类型测试、Adapter 检查、产品 smoke、既有 legacy 测试。
- 涉及核心行为的提交：相关 Rust 测试、Rust 默认接口边界、contract governance；不要无差别重跑无关昂贵矩阵。
- PR：完整 Node 产品矩阵与四目标构建，至少完整执行计划中的支持版本安装矩阵。
- Release：全部平台安装矩阵、依赖与许可证检查、文件清单、版本匹配、registry 安装与发布恢复证据。

脚本目标为 npm --prefix bindings/node run check、test:product、test:types、test:installed；这些名称在实施时建立，不代表现在已经可执行。完整检查接入 scripts/verify-ci.sh，重型平台发行检查放入专门工作流。

## 9. 依赖与实施顺序

主顺序为 P0 → P1 → P2 → P3 → P4 → P5 → P6 → P7 → P8 → P9 → P10。

平台构建在 P0 验证可行性，安装模型在 P1 跑通，P8 才用完整功能进行全平台验收。能力测试和示例跟随每一阶段提交，P9 负责补齐矩阵及文档收口。这个顺序能提前暴露发行问题，同时避免产品层在未固定的错误与状态模型上反复返工。

所有核心 contract、诊断 code 或兼容声明变化遵循现有 [Contract Change Policy](../process/contract-change-policy.md)：在对应实现 PR 中准备 ADR / RFC、必要的 registry / manifest / version change record 和验证证据。只增加 Node facade 不应无故提高 contract bundle 版本。

## 10. 主要风险与处理位置

| 风险 | 具体处理 | 阶段 |
| --- | --- | --- |
| 原生依赖与 Rust MSRV / lint 冲突 | 编译 spike、固定依赖、FFI lint 范围审查，保留核心 forbid | P0 |
| 产品对象再次经 JSON 重建而丢失状态 | Adapter 保存实际 Rust Project / Deck 及资源；验证添加和媒体指纹 | P0、P3、P4 |
| 异步构建期间对象被修改或 GC | 提交时 Busy、拥有关系保活、错误恢复测试 | P2 |
| JS 数值破坏身份或更新证据 | IDs 从 Rust 直接转十进制字符串；数字范围校验 | P2 |
| 大 Buffer 导致复制或资源占用 | 明确字节所有权；大输入文件分流；Buffer 导出记录内存行为 | P4、P6 |
| Node 风格重命名导致选项或报告字段遗漏 | 完整映射表、未知参数检测、逐项能力测试 | P2、P6 |
| npm 安装成功但原生包未装入 | 真实主包依赖选择测试、缺失包加载错误、四目标验收 | P1、P8 |
| Linux / Windows 隐式依赖开发机动态库 | 固定系统基线、依赖检查、最小消费环境运行 | P0、P8 |
| 多平台发布只成功一部分 | 平台包先发，版本精确，主包与 tag 最后；预发布恢复演练 | P10 |
| 把部分 SDK 当作完整交付 | C01—C18 和下面的完成清单全部作为发行条件 | P9、P10 |

## 11. 完成清单

- [ ] C01—C18 全部有产品实现、公开类型、示例和可观察测试。
- [ ] Node 使用者无需编写产品 JSON、寻找 manifest 或配置 Rust 工具链。
- [ ] npm 主包及四个平台包可通过标准依赖安装获得；消费端无编译或额外下载脚本。
- [ ] ESM、CJS、TypeScript 与支持的 Node / OS 组合全部通过安装测试。
- [ ] 添加时校验、独立 validate、媒体指纹、模板导入和身份行为保持 Rust 语义。
- [ ] 全部构建选项、完整报告、业务错误、Buffer 与 Writable 输出均完成。
- [ ] 比较和更新安全矩阵通过，被阻止的构建保留旧发布文件与基线。
- [ ] Node 生成的代表性场景完成 Anki Desktop 复核，记录实际检查版本和结果。
- [ ] 旧入口迁移有说明和回归；Rust 与 Python 的原有支持范围未被意外改变。
- [ ] 用户文档、发布手册、版本与产物清单齐全，文档示例从安装包运行。
- [ ] 预发布和正式 registry 安装均已验证；正式 npm 包可供用户使用。

需要外部设置的事项只有实际 npm 包名/命名空间归属、发布身份和各平台发布基础设施；这些不阻止 P0—P9 的本地实现与安装产物验收。本文完成表示实施计划已准备好，SDK 的实际完成仍以上述交付清单为准。


## 12. 首轮实施记录（2026-09-06）

已实现 0.2.0 候选 SDK，代码位于 `bindings/node/src` 和 `bindings/node/native`。
技术决策见 [ADR 0017](../adr/0017-native-node-product-sdk.md)，用法与限制见
[Node README](../../bindings/node/README.md)，发布步骤见
[RELEASING](../../bindings/node/RELEASING.md)。当前尚未公开发布 npm 包。

### 已验证证据

- 本机 macOS arm64：Rust 1.92，napi 3.12.2 / napi-derive 3.6.3 / napi-build 2.4.1，Node-API 8。
- 优化构建的原生模块约 11 MiB；运行时外部加载依赖为 macOS 系统库 libiconv / libSystem。其他目标依赖仍须由 CI 和目标机检查。
- Node 22.17.0 和 24.19.0：首轮 20 个产品集成测试通过；随后扩展到 22 个产品测试和 6 个一致性/更新测试，最终结果见后续记录。包含真实 APKG、失败后状态、事件循环、GC 保活、worker 退出、Deck、自定义 Normal/Cloze、模板导入、媒体指纹、跨项目引用、Buffer/流、全部 11 项预算、只比较和发布阻断。
- 旧 wrapper：23 个测试通过，包括四个已复现问题的回归。CLI 新增可选 `product-build --base-dir`，不改变省略该参数时的行为。
- Node 22 和 24 的隔离 registry 安装通过：主包真实选择宿主平台依赖、首次安装、空缓存生产 npm ci、禁用安装脚本、清空 Cargo/仓库配置、只读 node_modules、ESM/CJS/TypeScript、缺失原生包与版本不匹配。
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-features --locked`、Rust 格式检查通过。Rust 原有标为 ignored 的手工/性能场景不计作已运行。
- Python 兼容测试：107 passed、5 subtests passed；本轮未运行独立 wheel/import-isolation 发布测试。
- 新增 `node-sdk-ci`，声明四个目标 × Node 22/24/26 的 12 项构建及安装检查。写入 workflow 不等于这些远程检查已经通过。

### 实施中确认的细节

- `baseDir` 在 Node 入口转换用户路径；Rust 自身的媒体 staging 留在临时构建工作区。将同一个目录同时作为用户输入基准和 staging 目录会在重复构建时碰撞，已用大 Buffer 重复构建回归覆盖。
- Project 的 `addBuffer` 大数据分流读取核心新增的 `MediaRegistry::inline_limit_bytes()`，没有在 TypeScript 维护另一份阈值。
- MediaRef 保留真实 Rust 引用和导出文件名语义；不能靠 project token 拒绝核心允许的跨项目引用。
- Deck 媒体接口保留自身 Rust 语义：`addFile` 使用文件 basename，`addBytes(name, bytes)` 走 Deck 的字节路径。Project 媒体提供 exportAs 和大 Buffer 暂存；没有把两个 Rust API 的差异掩盖成同一套行为。
- 源码开发依赖独立放在私有 `toolchain` 包及其锁文件中，通过 `npm run setup` 安装；未公开发布的平台 optionalDependencies 不进入开发依赖锁文件，避免首次发布前 npm ci 校验失败。消费端仍按主包的精确平台依赖安装。
- `updateSafe(lockfile)` 不默认写入锁文件，严格对齐 Rust 的便利方法；`firstUpdateSafeBuild(lockfile)` 才默认写入。
- 流输出在生成 APKG 期间也监听目标流的关闭/错误，并等待原生任务结束后报告失败，不伪装成已取消 Rust 构建。

### 尚未关闭的工作

1. **当前 Rust 的图片遮挡缺陷：** `hide-one-guess-one` 由 stock renderer 生成 `c1,2`，现有 writer 会返回 `PRODUCT.CLOZE_MARKER_MALFORMED`。`hide-all-guess-one` 已成功导出。Node 保留原始错误；需要在核心层完成 grouped IO 语义设计、契约/Anki 验证后再关闭 P5.2。
2. C01—C18 更完整的独立 Rust/Node 错误场景矩阵；已完成 14 个独立产品构造场景及完整观察/报告/identity index 比较，映射见 [覆盖索引](../../bindings/node/COVERAGE.md)。
3. Windows 的 symlink/ACL 场景和跨平台资源寿命差异；本机 Node 专项 GUID/revision、字段改名/模板重排、损坏基线、旧锁文件、symlink/hardlink 保护已通过。
4. 完成四目标实际 CI、Node 26、系统最低版本与动态库核查、全部平台包一起进入 registry 的安装验收。宿主机安装测试目前对其他平台提供元数据来检验 npm 的平台过滤，不虚构其他平台二进制。
5. 用已生成的 Node 场景包完成 Anki Desktop 复核；自动化观察器通过不代表 Desktop 导入/渲染验证完成。README 三个完整示例现已从隔离安装包执行。
6. 确认 npm 包名控制权和发布凭据，完成全部可审查 tarball、预发布、恢复演练与正式发布。当前的仓库候选版不等于标准 registry 上已经可用的完整产品。

### 本轮收尾验证

- Node 22.17.0 / 24.19.0 均通过最终优化原生构建的 22 个产品测试及 6 个一致性/更新测试。
- 14 个独立 Rust/Node 场景比较完整构建报告、全部 APKG observations 与 identity index；GUID、模型/config ID、revision、媒体哈希均保留参与比较。只归一化耗时与测试目录。
- 特别验证 `9007199254740993` 模型 ID、旧 GUID 保留、旧锁文件迁移、内容修改/回退/标签/unchanged revision、模板 ord 改变与 config ID 稳定、损坏基线和文件别名保护。原生 JSON 边界另覆盖 i64/u64 极值与负安全整数边界。
- 11 个 InspectLimits 各有实际超限失败断言。MIME mismatch 严格/警告行为通过；Rust Product 对未知扩展名显式声明 application/octet-stream，这种输入不触发 unknown MIME 诊断，SDK 保留此语义。
- Node 22/24 的最终隔离 registry 安装、全新 npm ci、ESM/CJS/TypeScript、只读目录与 README 三个完整示例通过。主包包含 COVERAGE.md，本机主包与平台包 tarball 已生成在 `bindings/node/artifacts/0.2.0/darwin-arm64`。
- 新增转换模块的测试、native crate 全 target clippy、Rust 格式和 TypeScript 检查通过；前述 workspace/Python/legacy 结果仍适用于未再修改的对应代码。
- Desktop 待复核包与 SHA-256/自动化证据已生成在 `bindings/node/artifacts/desktop/scenarios-xTa1Y0`；其中 `DESKTOP-CHECK.md` 明确标为 pending。没有把观察器结果记作 Anki GUI 验收。
- CI 现为 4 个原生构建 job 汇集真实产物后，12 个 Node/OS 消费 job 使用包含全部平台 tarball 的临时 registry 安装。完整镜像已接入 `scripts/verify-ci.sh`；远程 jobs 尚未运行。
