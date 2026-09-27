# Python API 能力对齐实施方案

- 日期：2026-09-21
- 状态：已采纳并实施。正文保留原提案与验收标准；当前完成状态和分发门禁见 [执行记录](2026-09-22-python-api-parity-progress.md)，架构决策见 [ADR 0021](../adr/0021-native-python-product-sdk.md)。未发布包。
- 源码基准：`2edb8cc60e928b0d9cc161a2fcdc01234b722983`
- 配套事实清单：[Python API 差距审计](2026-09-21-python-api-gap-audit.md)。本文的 PY-xx 编号与该清单一致。
- 目标：让 Python 覆盖 Rust **Supported Consumer Interface** 的用户能力，并保持媒体、身份、校验、诊断和发布行为一致。

## 1. 结论与执行建议

建议分成两条交付线：

1. **先修正当前 Python 包能独立解决的问题**：跨项目媒体引用错绑、多行模板/CSS 被拒绝、报告字段丢失，以及风险阈值参数被过度限制。每项单独做回归和可审查的变更。
2. **把完整对齐作为新的 Python 0.2 目标**：推荐薄 Python 接口配合持有真实 Rust 对象的原生适配层；先证明打包、对象所有权和一条 Basic 导出链路，再逐步覆盖其他能力。

原生适配是本方案的**建议决策**，不是仓库已经采用的架构。旧 Phase 5 设计明确选择 bundled CLI，并排除 PyO3；实施原生路线时需要先写新的 ADR 和兼容性说明，明确新目标取代哪些旧约束。不能把已有 0.1 CLI 方案描述成实现错误，也不能在修补版本里悄悄替换它。[旧设计](../superpowers/specs/2026-05-26-phase-5-python-adoption-design.md#confirmed-decisions)

如果必须维持 CLI 分发，完整对齐仍然可以实现，但需要增加有状态 Rust 会话或可靠的状态重建与证据传递协议；这不是在现有 `write_apkg()` 上增加几个参数的工作量。备选路线见第 4 节。

## 2. 对齐边界

### 2.1 已具备的能力要保留

Python 已有 Project、Basic/Cloze、custom Normal/Cloze、图片遮挡 builder、文本/HTML/图片/声音、模板浏览器内容与目标牌组、生成规则、文件/字节媒体、比较构建、身份锁文件和更新安全模式。它们需要补行为与测试，不应全部重写或列成“尚未实现”。[公开导出](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/__init__.py)、[笔记](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/note.py)、[笔记类型](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/notetype.py)、[构建参数](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L132)

### 2.2 必需与扩展分开验收

| 范围 | 本方案处理 |
| --- | --- |
| Rust 文档支持的 Deck、Project、Note、NoteType、Content、媒体、BuildOptions、报告、Artifact、InspectLimits | 完整对齐的必需范围；涉及未重导出的参数类型时单独核查可调用性 |
| Python 命名、dataclass、上下文管理器、文件对象写入 | 按 Python 习惯设计，以可观察行为对齐 |
| 独立 `validate_template()` | 有价值的 SDK 扩展；对应核心 TemplateEngine，不计作当前 Rust prelude 的独立必需入口 |
| `Field.optional` | 声明和回读对齐；当前 Rust 构建仍只透传 required，不能声称 Python 缺少“可选字段制卡能力” |
| `first_update_safe_build()`、`update_safe()` | 便利封装，已有参数组合可实现；优先级低于语义缺口 |
| `lower()`、normalize、Writer、独立 APKG inspector、底层 lockfile 修改 | 不因源码可达就纳入公开兼容面；按 ADR 0012 排除内部工具接口 |
| pandas/CSV、AnkiConnect、浏览器/WASM、自动发布 | 不属于本次 API 对齐目标 |

接口边界来自 [prelude](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/prelude.rs#L1)、[ADR 0012](../adr/0012-narrow-rust-0.1-interface.md)、[Rust 用户文档](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/README.md#supported-01-interface)，不是 Node 的所有导出或所有 Rust `pub` 符号的并集。

Rust 的部分高级配置存在自己的导出边界问题：RiskLevel、ProjectNormalizeOptions 和媒体策略枚举未重导出到 prelude。BuildOptions 的 inspect、inspect_limits、artifacts_dir、self_contained 等明确属于可用对照；Python 的额外命名配置若需这些类型，应列为 SDK 扩展，或另行修正并承诺 Rust facade 的导出。不能假定打开 internal-tools 后能命名的每个枚举都已经是受支持的 Rust 用户能力。

### 2.3 已有核心限制

`hide_one_guess_one` 目前生成的分组 cloze 标记会被核心构建管线以 `PRODUCT.CLOZE_MARKER_MALFORMED` 拒绝；Python 不应另写 renderer 绕过。绑定应保留同样的结构化失败，核心修复后再使三语言共同转为成功场景。它是跨语言共有的核心问题，不是 Python 缺一个方法。[核心诊断](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/writer_core/staging.rs#L777)、[现有回归](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/node/test/product.test.mjs#L331)

## 3. 为什么只扩充 CLI 参数不够

当前执行链路是：

```text
可变 Python Note / NoteType / Project
  → ProductDocument JSON
  → contract_tools product-build
  → Rust ProductDocument 构建入口
  → 共用 normalize / writer / compare / risk / report
```

共用后半段管线，不等于调用同一套 Project 操作：

- `BuildInput::Document` 的前置 validate 返回空报告；真实 `Project` 调用其聚合校验。
- Document 输入没有 Project 的媒体注册表，因而不能天然保留注册时指纹。
- 当前 ProductDocument 字段不能表达所有内存状态，例如每笔记 identity override、Field 的独立 optional 标志，以及 Artifact 的所有权。
- 旧设计文档里的 `Project::from_product_document(...)` 流程不能当作当前源码事实。

依据：[CLI 构建入口](../../contract_tools/src/product_build_cmd.rs#L27)、[Document/Project 分叉](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project/input.rs#L5)、[ProductDocument 数据模型](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/model.rs#L136)。

因此以下做法不算完成：

- 让 `validate()` 在临时目录偷偷做完整 APKG build，却宣称与 Rust Project.validate 一致。
- 在序列化 JSON 时添加 Rust 不读取的字段。
- 只在 Python 检查一次文件 hash，然后让 Rust 重新读取可能已经变化的源文件。
- 让子进程返回临时 APKG 路径，退出时又把它删除。
- 复用同一个 ProductDocument 同时生成“Python”和“Rust”预期，再把相等结果当成独立对等证据。

## 4. 架构路线比较

| 路线 | 可以直接复用的内容 | 需要新增的机制 | 适用判断 |
| --- | --- | --- | --- |
| A：继续一次性 CLI | 现有 wheel、进程隔离、JSON/report、发布管线 | 实体重建、原始媒体证据、add 校验调用、临时文件所有权、完整协议扩展 | 适合有限补齐；完整对齐可行，但重建成本和协议复杂度需单独证明 |
| B：bundled CLI 的有状态会话 | 原分发方式、真正的 Rust Project/Deck | 请求协议、句柄表、启动/退出、stderr 排空、崩溃恢复、fork 后行为、Artifact 会话寿命 | 对“必须保持子进程隔离”的需求更合适 |
| C：Python 薄封装 + Rust 原生扩展 | Project/Deck/MediaRef/Artifact 的真实对象与核心错误 | 原生 wheel、解释器边界、并发租约、兼容性迁移 | **本方案推荐的完整对齐路线** |

这是基于本项目状态和维护成本的设计判断，不是性能测试结论。PyO3 可以将 Rust 结构暴露为 Python 类；拥有型句柄比带 Rust 借用生命周期的对象适合跨语言边界。[PyO3 类与所有权](https://pyo3.rs/main/class.html)

### 4.1 推荐模块边界

```text
bindings/python/src/anki_forge/
  project.py / deck.py       Python 公共入口、参数兼容
  note.py / notetype.py      作者输入描述；加入项目时形成快照
  content.py / media.py      typed content 与注册句柄
  options.py                BuildOptions、InspectLimits、枚举
  report.py / diagnostics.py 无损报告与结构化异常
  artifact.py               持有 Rust Artifact 的 Python 包装
  _native.*                 私有原生扩展，不承诺其符号稳定

bindings/python/native/     新增，名称和布局在技术试验后固定
  authoring / media / build / reports / artifacts / state
  → 同仓库 ankiforge 核心
```

Maturin 支持 Python 源码与 Rust 扩展混合布局，适合保留 Python 友好入口；具体 PyO3/Maturin 版本在技术试验中验证并锁定，不在本计划凭空指定版本。[Maturin 混合项目](https://www.maturin.rs/project_layout.html)

优先通过现有 Rust facade 调用。适配器确需使用内部的报告转换或模板辅助接口时，按同仓库私有适配器管理；不得把这些 Rust 类型直接扩大为 Python 稳定 API。不要为了 Python 绑定随意扩大 Rust prelude。现有 Node adapter 可作为设计参考，不能直接复制其所有接口限制。[Node adapter](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/node/native/Cargo.toml)、[Node 对象状态](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/node/native/src/state.rs#L13)

### 4.2 如果保持 CLI，必须另补的工作

选择 B 时，新增的私有 `sdk-session` 需要 Rust 持有 Project、Deck、MediaRef、Artifact；协议携带版本、请求 ID、操作、句柄和完整结果，不让用户管理句柄。

- EOF/显式 close 释放对象；失联后的旧句柄不能静默重用。
- stdin/stdout 专门承载协议；stderr 有界收集并持续读取，避免管道阻塞。Python 官方文档明确提醒 PIPE 与等待方式的死锁风险。[subprocess](https://docs.python.org/3.11/library/subprocess.html#subprocess.Popen.communicate)
- 不能在一次外部写入结果不明时自动重试；返回附带可恢复报告的运行时错误。
- Artifact 保留引用必须延长所属进程或采用显式所有权交接；不能以“报告里有路径”代替生命周期。
- 不改变公开 ProductDocument 契约来偷运会话句柄；该协议应有独立版本与兼容测试。

选择 A 时，还要证明每次重建仍保留原始注册证据、Deck 身份快照、模板来源位置及原子 add 语义，并测量逐条 add 导致的重复序列化/重建开销。这些要求达成前，范围只能标为“CLI 功能扩充”，不能称为完整对齐。

## 5. 目标 Python API 与语义

以下均为目标接口草案，不是当前可运行示例。

### 5.1 作者模型与身份

- 保留 `Note.basic/cloze/image_occlusion`、`Note(type_id)`、`NoteType.custom/custom_cloze` 及现有 snake_case 写法。
- 新增 `Content.text/html`、`note.field(key, content)`、`MediaRef.image/sound`；现有 text/html/image/sound 方法继续可用。不把相同输出的快捷方法当作缺失的制卡类型。
- 新增 `IdentityRecipe.fields(keys)`、NoteType 显式 identity 和每笔记 `note.identity(keys)`；推导算法、规范化、字段顺序、空值和冲突处理都调用 Rust。
- 保留现有 `Field(identity=True)` 的类型级默认规则：没有显式 NoteType recipe 时，按字段声明顺序收集已标记的 key，并在适配时设置 Rust `NoteType.identity(IdentityRecipe::fields(...))`。显式类型 recipe 优先于这套回退；Field 的标记仍独立保留。每笔记 override 与显式 stable_id 的优先级沿用核心。只复制 Rust `Field.identity()` 标记不够，真实 Project.add_note 不会自动从这些标记生成 recipe。[现有 Python 映射](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/product_json.py#L234)、[核心身份前置条件](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L792)、[核心规则解析](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L1889)
- 保留 Python 现有 stock 字段 key 写法，适配时只对 stock 类型映射到 Rust 字段名称；自定义字段不得套用这张映射表。自动 key、非 ASCII 名称、前后空白由专门对等用例决定，不复制第二套推导算法。
- `Field(optional=True)` 作为声明补齐，与 required 的冲突规则须文档化；不引入 Rust 当前不存在的生成语义。

对应 PY-04、PY-05、PY-16、PY-18；依据：[Rust Note](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/note.rs#L54)、[Rust Field/NoteType](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/notetype.rs#L17)。

### 5.2 Project 与校验

```python
project = Project("Vocabulary", stable_id="vocabulary", base_dir=source_root)
project.add_notetype(note_type)        # 同步、Rust 校验、失败不添加
project.add_note(note)                 # 同步、加入时保存输入快照
project.import_template_bundle("templates/vocabulary")
validation = project.validate()        # 聚合诊断，不构建 APKG
validation.ensure_success()
comparison = project.diff_against_apkg("previous.apkg", inspect_limits=limits)
```

- validate 不保证媒体后续仍存在，也不完成 normalization、writer、update safety；这些仍在 build 执行。
- 模板包交给 Rust loader 解析，保留模板文件路径、字节位置和原子导入；导入失败不留下部分 note type 或媒体。
- diff 可以产生内部临时候选，但不发布用户 APKG，不写/推进身份锁文件，失败仍保留完整比较报告。
- 独立模板校验可在后续提供 `validate_template(source, field_names)`，只封装现有 Rust 语义。

对应 PY-01、PY-02、PY-03、PY-12；依据：[Project 操作](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L119)、[独立比较](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L875)。

### 5.3 构建配置

新增只读 BuildOptions；默认值来自核心，不手工维护另一套默认预算。

| Python 配置 | 对应行为 | 当前缺口 |
| --- | --- | --- |
| output、artifacts_dir | 显式输出、保留 staging，或默认临时 Artifact | output 已有，其余缺 |
| inspect、inspect_limits | inspect 控制报告摘要；limits 控制当前与基线包的 11 项有限预算 | 未暴露 |
| self_contained / media_mode | 核心自包含或路径媒体模式 | 未暴露 |
| media_store_dir、media_policy | 媒体存储和现有 MIME/未使用绑定诊断策略 | 未暴露 |
| compare_to、fail_on | 比较与风险阻断 | 已有，需修正 Python 额外限制 |
| identity_lockfile、write_identity_lockfile、update_safety | 身份证据、锁文件发布与模式 | 已有，需补完整对等证据 |
| report_json | 完整报告持久化及路径保护 | 已有，需验证与新 Artifact 的组合 |

表内 media_mode、media_store_dir、media_policy 的完整命名接口按第 2.2 节作为扩展或 Rust facade 后续补齐处理；优先验收默认特性下已经能正常调用的 BuildOptions 能力。fail_on 在 Python 和 Node 已提供字符串入口，其对照核心行为应保留，但不能据此断言 Rust RiskLevel 已在 prelude 导出。

inspect=False 不等于关闭核心最终检查；Python 文档和测试必须保留这个区分，不能为了加速跳过既有发布前检查。

`Project.build(BuildOptions(...))` 支持不指定输出；`write_apkg(path, ...)` 保留现有关键字调用作为便利入口。高级配置集中在 BuildOptions；出现重复且冲突的配置必须在进入核心前明确报错，不能静默覆盖。

检查预算使用 Python int 并按核心整数范围检查，拒绝 bool、负数和溢出；不要经过浮点转换。现有 update_safety 的 report_only/report-only 拼写兼容可在入口归一化，不随底层替换无声消失。

base_dir 是 Python 路径便利选项，构造时固定；相对媒体、模板、输出、基线、报告与锁文件按同一基准解析。它不是目前 Rust 所有路径调用都具有的额外保证；新默认行为的迁移见第 6 节。

对应 PY-09、PY-14、PY-19；依据：[Rust BuildOptions](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/build/options.rs#L60)、[InspectLimits](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/writer_core/inspect_limits.rs)。

### 5.4 媒体：先保证正确引用，再补便利方法

- MediaRef 按 Rust 的 export filename 语义解析；跨项目引用允许存在，但目标项目必须注册对应 filename。不能因为两个 registry 都有 `media:000001` 就认为它们指向同一文件。
- add_file 在注册时调用 Rust 校验并保存证据，build 再验证源文件变化；不得在 Python 预检之后把未经同一证据验证的重新读取结果交给 writer。
- add_bytes 沿用核心的空输入、64 KiB 限制和冲突语义；大内容使用文件，或以后提供明确命名的 spooling 便利方法。不能偷偷放宽同名 API 的核心限制。
- 不把 MediaRef 解释成“来源项目所有权”或“内容永远相同”的保证；同 filename 在目标 registry 中的解析按核心规则执行。
- 不在 Python 重新实现 MIME、图片尺寸、矩形边界或 identity 算法。

对应 PY-06、PY-07、PY-08；依据：[Rust 媒体注册与指纹](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/media_registry.rs#L13)。

### 5.5 Artifact、bytes 与文件对象

```python
report = project.build()
report.ensure_success()
artifact = report.artifact
saved = artifact.persist_to("final.apkg")
data = project.to_apkg_bytes()
project.write_to(binary_file)
```

- ApkgArtifact 持有真实 Rust handle；Python 的报告、异常和独立 artifact 引用都能延长临时文件寿命。
- 提供显式 close/context manager；最后一个拥有型句柄释放临时文件，显式输出与持久化副本不删除。单纯复制 path 不持有文件。
- persist_to 调用 Rust 原子复制；失败保留原文件，临时 Artifact 不能持久化到自身或其别名。
- report_json 在没有 output/artifacts_dir 时遵守 Rust 的拒绝规则，不能让 JSON 声称拥有临时文件。
- to_apkg_bytes 是完整字节结果；write_to 可以从已完成的临时 APKG 分块写入文件对象，不能称为“边构建边流式输出”。
- 文件对象写入必须处理 short write、零进展和异常，保持调用者文件打开；调用 Python write 时不持有项目状态锁。

ApkgArtifact 可兼容 `artifact["path"]` 读取，但 path-only JSON 的生命周期不升级为拥有型句柄。独立 close、共享引用及报告 close 的规则必须先由生命周期测试固定。

对应 PY-10、PY-11；依据：[Rust Artifact](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/build/artifact.rs#L14)、[生命周期测试](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/tests/artifact_lifecycle_tests.rs)。

### 5.6 报告与错误

- BuildReport 增加 metrics、policy、tool_version/schema_version 和 raw；保留所有已有字段、完整诊断、未知扩展字段和 Python 整数精度。
- 不再在回序列化时把 duration 重置成 0、policy 重置为 not_evaluated；原始报告必须能无损往返。
- 当前 report_json 文件由 Rust 写出，原始 metrics/policy 仍在；缺口位于 Python BuildReport 投影。当前 _report_to_json 只见测试调用，不能把它伪造默认值的问题扩大为“现有磁盘报告已经损坏”。
- 保留现有 Python 约定：核心返回有效失败报告时，build/write_apkg 返回报告；`ensure_success()` 再抛携带报告的异常。不要机械照搬 Node 的 Promise rejection 模式。
- 新的 ProjectAddError、MediaError、TemplateBundleError 可以继承现有 ValidationError；BuildError 继承 DiagnosticsError。稳定 code、failure_cause、path、span 与 report 来自核心。
- 初始化/加载失败、协议错误与领域诊断分开；不能将完整核心错误压缩成没有 code 的 ValueError 文本。

对应 PY-02、PY-13、PY-20；依据：[Python 报告现状](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/report.py#L28)、[当前回序列化](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L313)、[Rust 错误](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/build/report.rs#L109)。

### 5.7 Deck 与迁移 Project

新增 Deck 便捷作者接口，调用真实 Rust Deck；不能仅把 Project 改名，因为身份推导、身份覆盖、图片尺寸和矩形边界有独立行为。

特别是 Rust Deck 的图片遮挡可在没有显式 stable_id 时推导身份，并使用图片尺寸检查遮挡边界；现有 Python Project 图片遮挡 builder 必须提供 stable_id。这个差异属于实际能力缺口，不能仅归类为快捷语法。

`Project.from_deck(deck)` 建议通过核心 Deck 快照再执行 Rust 转换，保留原 Python Deck 可用；测试必须证明媒体、原笔记 GUID/身份来源与追加自定义笔记均与 Rust 对照一致。迁移应保留诊断来源，不把导入后的项目降为不可编辑快照。

对应 PY-17；依据：[核心转换](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project/deck_import.rs#L8)、[现有转换回归](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/tests/deck_project_facade_tests.rs)。

## 6. 兼容性与并发必须先定下来

### 6.1 不能悄悄改变的现有行为

当前 Python 会保存 Note/NoteType 的可变引用；加入项目后再修改输入，会影响以后序列化，现有测试依赖这一点。[测试证据](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/tests/test_product_validation_parity.py#L33)

这一项对应 PY-15，属于有意的版本迁移，不能作为普通 bug fix 无声改变。PY-16 的自动 key 推导变化也有身份风险：旧 Python 与 Rust 对连续空格和标点的处理不同，升级说明应要求已有牌组固定原 field/template key，而不是重新生成一套 key。

新原生路线建议采用“加入时形成快照”：

| 行为 | 0.1 现状 | 0.2 建议 |
| --- | --- | --- |
| 修改已加入的 Note/NoteType 原对象 | 可能改变项目，直到序列化重新检查 | 只改变输入对象，不反向改变已加入项目 |
| 重复 stable_id / 身份不足等错误 | 部分延迟到序列化 | 按 Rust add 边界同步拒绝，项目不变 |
| 添加不存在或空的媒体文件 | 可延迟到 build | 按 Rust 注册边界拒绝 |
| 项目相对路径 | 受不同调用时 cwd 影响 | 由构造时 base_dir 固定 |
| artifact | path Mapping | 拥有型对象，并提供 path 兼容读取 |
| RuntimeOverride / to_product_document | CLI 高级入口 | 归入 0.1 兼容范围；不伪装成原生运行时配置 |

这些变化需要新的 Python minor 版本与迁移示例。推荐保留 0.1 维护线供依赖旧行为的用户固定版本；首个原生 wheel 不默认携带两套运行时，也不把原有 dev-only `anki_forge_python` 塞回公共 wheel。若确有需求再单独设计 legacy 子入口。

迁移测试必须由 **0.1 实际生成 APKG/lockfile，再让 0.2 读取和更新**：验证 unchanged、答案修改、标签修改、含标点的隐式 key、自定义多模板和 legacy lockfile。能够保留身份的场景检查 GUID、模型/模板 ID 与 revision；不能安全恢复的场景必须明确阻断并给迁移说明，不能在测试里无基线重建后宣称升级成功。

公开 notes/notetypes 等只读观察接口需要明确返回快照。避免为了兼容 getter 永久保存两套可变笔记正文；优先按需读取核心只读视图，必要的核心内部观察接口单独评审，不作为新公共 mutation seam。

### 6.2 原生边界并发规则

- 同一 Project/Deck 同一时刻只允许一个操作；状态至少区分 Ready、Busy、Failed。领域错误恢复 Ready；不可恢复 panic 退休该对象。
- 昂贵 Rust 工作释放解释器锁后执行，但不携带借用的 Python 对象；输入 bytes/bytearray 先快照，输出在安全边界重新转为 Python 对象。
- 独立项目可以并行；拒绝并发修改同一项目，不承诺可随意共享可变 builder。
- 没有实现取消协议前，不提供“超时即取消构建”的承诺；进程终止与键盘中断不能被描述成文件写入已回滚。
- 普通 CPython、free-threaded Python、子解释器、fork 后复用分别列验证范围，不能因为一个 wheel 能 import 就一并宣称支持。

这些是适配层设计与试验项；PyO3 官方建议在不访问 Python 对象的长耗时工作中脱离解释器，并对 free-threaded 环境单独处理共享状态。[PyO3 并行](https://pyo3.rs/main/parallelism.html)、[free-threading](https://pyo3.rs/main/free-threading.html)

## 7. 实施顺序与可独立审查的任务

S/M/L 表示相对规模；不是未经试验的工期承诺。P0 表示先解决结果正确性或高频使用问题；不等同于已经证明安全漏洞。

| 任务 | 内容与关联缺口 | 主要文件/交付物 | 前置 | 规模 |
| --- | --- | --- | --- | --- |
| T0 | 固定对齐范围、复现清单、版本迁移；决定新 ADR | 本文、审计、拟新增 Python ADR、能力索引 | 无 | S |
| T1a | 修复多行模板/CSS；区分 ID 校验与源码文本保真，PY-05 | notetype.py、模型测试、模板 fixture | T0 | S |
| T1b | 按 filename 解析媒体引用，消除序号碰撞，PY-08 | note.py、media.py、product_json.py、跨项目 E2E | T0 | M |
| T1c | 无损报告与风险参数限制，PY-13/14 | report.py、project.py、runtime 测试、锁文件场景 | T0 | S/M |
| T2 | 原生技术试验：Basic→真实 Rust Project→APKG→wheel 外部安装 | 拟新增 native crate、试验包配置、试验记录 | T0 | M |
| T3 | 作者输入、快照、结构化 add、validate、identity、字段映射，PY-01/02/04/15/16/18 | authoring/state、Python模型、diagnostics、迁移文档 | T2 | L |
| T4 | 注册时证据、64 KiB、重复/空媒体、跨项目引用，PY-06/07/08 | native media、Python media、来源变化用例 | T3 | M/L |
| T5 | BuildOptions/InspectLimits、完整报告、路径/风险/锁文件，PY-09/13/14/19 | options/build/reports、buildable facade、路径与失败测试 | T3/T4 | L |
| T6 | Artifact 所有权、持久化、bytes/write_to，PY-10/11 | artifacts、Python artifact、生命周期与文件对象测试 | T5 | M |
| T7a | 原子模板包导入，PY-03 | native authoring/media、templates fixtures | T3/T4 | M |
| T7b | 无发布副作用的 diff，PY-12 | comparison、ProjectDiffReport、异常与基线测试 | T5 | M |
| T8 | 真实 Deck 与 Project.from_deck，PY-17 | deck adapter、Python deck、Deck身份/遮挡/转换测试 | T3/T4/T5 | M/L |
| T9 | 独立对等矩阵、wheel平台门禁、类型和文档，PY-20 | parity runner、CI、py.typed/stubs、README、迁移指南 | T3–T8 | L |

T1a/T1b/T1c 可分别合并，不等待原生扩展。T2 的跨平台构建必须前置验证，避免最后才发现打包路线不成立。T9 的测试基础设施从 T2 开始搭建，最后阶段只补齐矩阵，不把测试推迟到全部实现后。

原生路线选定后，T3、T4、T5 应逐个交付能真实生成 APKG 的纵向切片；不能先实现全部空接口再等待一次“大集成”。

T3 必须单独覆盖 `Field(identity=True)` 的兼容映射及其与显式类型 recipe、每笔记 override、stable_id 的组合。用 0.1 实际生成的 APKG/lockfile 做更新基线，核对 GUID、身份来源和 revision evidence；不能只比较 recipe hash，或仅证明新版本能从零构建。

### T2 的退出条件

技术试验通过需要交付可重复执行的证据，而不只是一个能 import 的空扩展：

1. Python 构造一条 Basic，调用实际 Rust Project.add_note/build；独立 Rust 对照的 GUID、字段和卡片数相同。
2. 单个媒体注册保留 Rust 证据；修改源文件后报告 SOURCE_CHANGED，领域失败后对象仍可复用。
3. 原生返回的临时 Artifact 在报告释放后可由独立引用继续持有，最后释放才清理；扩展不持有短寿命 Python 借用。
4. 以仓库 Rust 1.92 基线构建，验证所选依赖与扩展 FFI 的 lint 边界；核心继续禁止 unsafe，不新增手写 unsafe 绕过所有权问题。
5. 四个平台的真实 wheel 至少完成外部 venv 的 Basic smoke；sdist 若提供，必须从解包目录在仓库外完成构建，确认 path dependency、Cargo.lock、核心 build.rs 和内嵌资源完整。
6. 记录 CLI 基线和原生试验的 import、首次/重复构建、1,000/10,000 笔记输入成本、媒体注册及重复运行临时文件数量。区分冷启动与常驻，不预设“换原生一定更快”。

若遇到解释器/ABI/平台依赖不满足，先解决或重新选择第 4 节路线；不要在已经积累完整新 SDK 之后才暴露该限制。

### T1 的实现注意点

- **T1a**：允许正常换行/缩进并保持模板与 CSS 原文；不要简单删除所有验证，也不要继续用 ID 的 trim 规则处理模板正文。分别测试 LF、CRLF、Tab、前后空白和空模板的核心行为。
- **T1b**：FieldContent 必须保留足以解析 export filename 的信息；仅在构建前检查 media_id 是否存在无法修复错绑。目标 registry 缺 filename 时应明确失败，同 filename 按 Rust 语义处理。
- **T1c**：使用非默认 duration、blocked policy 和未知字段验证真实往返；仅锁文件+fail_on 的用例需要让 Rust 决定结果，不能为了通过 Python 预检而强行附加 APKG 基线。
- **PY-06**：注册指纹最终在 T4 关闭。若需在 CLI 维护线上提前修补，必须另做“注册取证→验证并复制同一字节快照→构建该快照”的切片，测试复制期间源文件变化；单纯 stat/hash 预检不算关闭。

## 8. 验收矩阵：定义什么叫完成

独立 Rust producer 使用公开 Deck/Project 构造相同场景；Python 单独创建输入。观察器可以复用仓库 inspector 和 baseline reader，但不能把 Python 生成的 JSON 反喂 Rust 当独立 producer。[已有独立对等模式](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/node/COVERAGE.md)、[Node Rust producer](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/node/native/examples/sdk_parity.rs)

| 组 | 至少覆盖的用例 | 必须断言 |
| --- | --- | --- |
| A 作者与身份 | Basic/Cloze、custom Normal/Cloze、多模板、标签/牌组、字段改名/重排、默认与每笔记 identity、Deck IO 无显式 stable_id | 字段/模板内容、卡片数、GUID、model/config ID、身份来源与 revision evidence |
| B 校验 | 重复 ID、未知字段、无 identity、非法模板、失败后重试 | 稳定 code、severity、path/span；add 前后项目状态；validate 不生成 APKG |
| C 媒体 | 文件/bytes、空/超限、同名同内容/不同内容、注册后修改/删除、同序号不同 filename 的跨项目引用 | 注册与构建时机、正确文件内容、SOURCE_CHANGED、无静默错绑；失败不污染 registry |
| D 模板与遮挡 | LF/CRLF/Unicode、多行 CSS、bundle CSS/字体/图片、导入中途失败、两类 IO 路径 | 模板保真、来源位置、原子导入；共有核心限制使用明确预期失败 |
| E 构建与更新 | 11 项预算、媒体模式/策略、APKG/锁文件基线、strict/report-only/disabled、答案/标签/回退更新、0.1 产物升级到 0.2 | 完整报告、GUID/mtime/revision、风险阻断、基线与旧输出不被改写 |
| F 临时产物 | report/exception/artifact 各自持有、复制句柄、close/GC、persist 失败、自路径别名 | 最后所有者释放才清理；显式输出保留；失败后临时原件仍可用 |
| G 输出与 diff | 文件/bytes/文件对象、short write/零写入/异常、独立比较 | 解码 APKG 内容一致、调用者流仍开着、比较不推进锁文件、不留下候选 |
| H 路径与错误 | Unicode/空格、cwd 改变、symlink/hardlink、staging重叠、报告写失败、只读目录 | 核心别名保护、原子发布、晚期错误中可恢复 artifact/report |
| I 分发与资源 | 干净 venv、仓库外运行、无 cargo/CLI 环境、版本错配、独立项目并行、反复构建 | wheel 自足、清晰加载错误、无共享状态污染、无临时文件累积 |

完整报告比较只允许明确列举的差异，例如耗时、临时根目录和适配器自身版本信息。不能统一删除 diagnostics、policy、身份字段或数值尾数来让比较通过；最终 ZIP 顺序/压缩造成的字节差异不等于语义失败，但须解释比较边界。

已有可复用测试：[Artifact](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/tests/artifact_lifecycle_tests.rs)、[Deck→Project](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/tests/deck_project_facade_tests.rs)、[模板入口对等](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/tests/custom_template_entry_parity_tests.rs)、[Python E2E](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/tests/test_product_e2e.py)。测试数量只是执行记录，能力矩阵每一行的独立证据才是覆盖依据。

## 9. 打包、类型与发布

现有 Python wheel CI 已包含 Linux x86_64、Windows x86_64、macOS x86_64/ARM64，以及 Python 3.11/3.12 组合；这是已有配置，不能描述为“还没有跨平台 wheel 支持”，也不能仅凭配置宣称远端验证已经通过。[当前矩阵](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/.github/workflows/contract-ci.yml#L28)

原生路线的发布要求：

1. T2 验证四平台可构建、加载，并在仓库外导出 Basic；原生 wheel 使用真实二进制兼容标签，不能复用 CLI wheel 的 py3-none 手动 retag 流程。
2. 优先试验以 Python 3.11 为最低版本的普通 CPython abi3 wheel；若所选 API/平台不支持，使用相应 CPython wheel。free-threaded 与其他解释器独立设门禁，不以 abi3 推断覆盖。[PyO3 分发](https://pyo3.rs/main/building-and-distribution.html)
3. 维持现有 Python 3.11/3.12 回归；新增 3.13/3.14 运行测试后再写对应支持声明。最低 OS/glibc、动态库依赖与打包修复结果保留证据。[Maturin 分发](https://www.maturin.rs/distribution.html)
4. wheel 内附扩展、Python facade、类型信息、许可证/必要 notices；默认使用核心内嵌契约，不依赖仓库相对路径、用户安装编译器或 package 目录可写。
5. 添加 py.typed 和私有扩展的必要 stub；验证公开 API 的正负类型用例。已有 mypy 配置不等于完整类型分发或已通过检查。
6. 提供 binding/core/embedded-contract 版本查询，拒绝不兼容的绑定与核心组合；不把 Python 版本、Rust crate 版本和 Bundle Version 混成同一轴。
7. 文档包含作者、媒体、bundle、更新安全、临时 Artifact、迁移和故障排查的可运行示例；安装测试必须脱离源码、开发工具和开发环境覆盖变量。

源包构建、PyPI 发布、凭据与发布审批是后续交付任务。本方案不创建 release tag、不上传包，也不以本机测试替代发布平台结果。

## 10. 完成标准与建议首批工作

完整对齐完成时，应同时具备：

- PY-01–PY-17 中的功能/语义项逐条关闭；其中便利封装和行为迁移按各自定义验收，不能通过删减已有制卡、媒体和更新能力规避对等。高级 CLI 入口的版本归属按第 6 节明确迁移。
- PY-18 明确实现或记录为无产物差异的声明取舍；PY-19 的既有核心保护有 Python 消费侧证据；PY-20 的对等/分发矩阵执行通过。
- 0.1→0.2 的快照、错误时机、路径、Artifact 与高级 CLI 入口迁移已有明确说明。
- 共有核心限制单独列出，不作为 Python 漏实现，也不宣称其成功能力已经可用。
- 文档中的命令和示例在打包后的消费者环境运行；版本/平台证据可追溯到同一提交。

建议首批按 **T1b 媒体错绑 → T1a 多行模板/CSS → T1c 报告/风险参数 → T2 原生纵向试验** 排序。前面三项修复已有用户行为，T2 用来尽早验证完整对齐路线的可行性；其后按依赖推进，不以新增方法数量衡量完成度。

## 11. 本次整理的验证范围

本会话前一轮已重新构建 Rust CLI 和 Node addon，并运行 Node 产品/对等测试 28 项、Python 107 项与 5 个子测试；Python 安装隔离测试未执行。这些结果只证明当前已实现功能的回归基线。

本次具体缺口的临时 probe、源码证据和限制详见配套审计。新原生架构、上述目标接口、wheel 与迁移方案尚未实现或验证；不能把计划中的验收条件写成当前测试结果。
