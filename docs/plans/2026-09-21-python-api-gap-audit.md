# Python 公共 API 能力差距审计

日期：2026-09-21。源码基准：`2edb8cc60e928b0d9cc161a2fcdc01234b722983`。以下是当时的缺口、影响和验收要求；源码证据已固定到该提交，不代表 Python 0.2 当前仍有这些缺口。

后续实施见 [执行记录](2026-09-22-python-api-parity-progress.md)、[PY-01–PY-20 当前能力与验证索引](../../bindings/python/COVERAGE.md) 和 [0.2 迁移说明](../../bindings/python/MIGRATION.md)。

配套：[Python API 能力对齐实施方案](2026-09-21-python-api-parity-implementation-plan.md)，包含架构取舍、分期任务、兼容性与发布门槛。

## 审计边界与结论

比较对象是公共包 `anki_forge` 与 Rust 0.1 的 Supported Consumer Interface：`prelude`、根级 `Deck` / `Project` / `Severity` 及版本查询。Rust README 明确排除 contract loading、normalization IR、writer、inspection、persistence 等内部模块；不能把 `internal-tools` 的所有 `pub` 项都算成 Python 必补接口。Python 的 `anki_forge_python` 也明确不进入公共 wheel，不能用其 raw / structured 接口宣称公共包已经补齐。[Rust 支持边界](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/README.md#L25)、[Rust prelude](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/prelude.rs#L1)、[Python wheel 包含规则](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/pyproject.toml#L15)。

Python 能完成常见的制卡、媒体导出和安全更新，但还不能覆盖全部受支持的 Rust 使用方式。更重要的差距是已有操作的语义：媒体可能在注册后被悄悄替换，跨项目引用会因局部序号碰撞而指向其他文件，多行模板/CSS 被 Python 拒绝，报告中的风险策略信息被丢弃。这些应优先于增加便捷方法。

以下使用四类主标签：

- **功能未暴露**：Rust 支持的操作无法经公共 Python API 直接完成。
- **语义差异**：双方都有相关操作，但身份、校验、状态、错误或输出行为不同。
- **报告信息丢失**：核心已经计算，Python 投影没有保留。
- **未验证 / 共有核心限制**：已有保护或低层共有限制，不直接视为 Python 功能缺失。

“优先级”是实施建议：P0 为可能产出错误内容或身份的差异；P1 为主要能力或可诊断性差距；P2 为便利性、回读与覆盖完善。它不是已经批准的发布范围。

## 已有能力，不应重复列入缺失清单

| 能力 | 当前证据及边界 |
|---|---|
| Basic、Cloze、安全文本与显式 HTML、标签、逐笔记 deck | [`Note` 的构造和内容方法](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/note.py#L24)，[`Project` 的 deck 解析](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L205)。没有 `Content` 同名类不等于不能表达这些内容。 |
| 自定义普通笔记类型和自定义 Cloze | [`NoteType.custom_cloze`](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/notetype.py#L215)，[`product-v3` 选择](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L118)，[现有序列化测试](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/tests/test_product_json.py#L52)。本次另实测自定义 Cloze 导出成功：1 个 note、2 张 card。 |
| 图片遮挡 builder | [两种模式、矩形、header/back_extra/comments/tags](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/note.py#L109)，[真实 runtime 构建测试](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/tests/test_product_e2e.py#L106)。已验证显式 stable ID 的 hide_all 构建；hide_one 有下文列出的共有核心限制。Deck 的自动图片遮挡身份是另一项差距，见 PY-17。 |
| 模板生成规则、浏览器模板、目标 deck | [`GenerationRule`](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/notetype.py#L69)、[`Template` 字段](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/notetype.py#L138)、[序列化](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/product_json.py#L199)。多行内容仍受 PY-05 限制。 |
| 自定义类型的字段身份和显式 note stable ID | [字段 identity 生成类型级 recipe](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/product_json.py#L234)，[stable ID 序列化](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/product_json.py#L260)。不能说 Python 完全没有 identity 能力。 |
| 文件/内存媒体、image/sound 引用 | [`MediaRegistry`](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/media.py#L36)、[内容序列化](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/product_json.py#L248)。注册验证、指纹及引用语义仍有 PY-06～08。 |
| baseline 比较、lockfile 读写、strict/report_only/disabled、风险阻断、report_json | [`write_apkg` 参数及调用](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L132)，[CLI 参数传递](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/runtime.py#L87)。缺的是独立 diff、部分配置和边缘组合，不是整个安全更新能力。 |
| hard link baseline 保护、阻断前保留既有 output/lockfile | 已有[硬链接别名 E2E](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/tests/test_product_e2e.py#L48)和[风险阻断 E2E](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/tests/test_product_e2e.py#L66)。不能只看 Python 的路径字符串比较就判定保护缺失。 |
| 自带 runtime 的平台 wheel 和安装隔离 | [bundled runtime 优先解析](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/runtime.py#L33)，[Linux/macOS/Windows、Python 3.11/3.12 矩阵](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/.github/workflows/contract-ci.yml#L31)。不能说 Python 尚未打包或必须靠源码仓库运行。 |

## 当前实现为什么会出现分叉

Python 在可变对象中维护 `Note` / `NoteType` / `Project`，序列化为 product-v2/v3 JSON，复制文件媒体到临时输入目录，再启动 `contract_tools product-build`。运行时最终走的是 `ProductDocument.build`。这条路径与真实 Rust `Project` 共用后续构建，但不是同一作者模型：[Python 构建入口](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L158)、[复制媒体](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L184)、[runtime 入口](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/runtime/product_build.rs#L5)。

`BuildInput::Document` 的 `validate()` 返回空的 facade validation report；它仍会经历 transport/lowering/normalization 校验，所以这不意味着“没有校验”。但它不执行 `Project.validate()`，也没有 Project 媒体注册表。区别见 [`BuildInput::validate`](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project/input.rs#L51)、[媒体注册表选择](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project/input.rs#L110)。

因此，仅把更多 CLI flags 接入 Python 可以补配置，不能自动补上注册阶段指纹、作者对象身份覆盖、模板来源、add 时校验和对象所有权。实施方案需要明确哪些操作调用真实 Rust facade，哪些继续走 document transport。

## 差距索引

| 编号 | 项目 | 主分类 | 建议优先级 |
|---|---|---|---|
| PY-01 | 独立 Project 校验及 ValidationReport | 功能未暴露 | P1 |
| PY-02 | add 时校验及结构化作者错误 | 语义差异 | P1 |
| PY-03 | 模板包导入 | 功能未暴露 | P1 |
| PY-04 | note 级身份覆盖、显式类型级 recipe | 功能未暴露 / 语义差异 | P1 |
| PY-05 | 模板/CSS 的多行与空白保真 | 语义差异 | P1 |
| PY-06 | 文件媒体注册校验及内容指纹 | 语义差异 | P0 |
| PY-07 | 内存媒体限制、空输入、内容去重 | 语义差异 | P1 |
| PY-08 | 跨项目 MediaRef 序号碰撞导致错绑 | 语义差异 | P0 |
| PY-09 | 构建选项、检查预算、工件目录 | 功能未暴露 | P1 |
| PY-10 | 临时 APKG 的所有权与持久化 | 功能未暴露 | P1 |
| PY-11 | bytes 和有界文件对象输出 | 功能未暴露 | P1 |
| PY-12 | 独立 diff 工作流 | 功能未暴露 | P1 |
| PY-13 | BuildReport 的 metrics / policy | 报告信息丢失 | P1 |
| PY-14 | fail_on 被错误限定为必须 compare_to | 语义差异 | P1 |
| PY-15 | 已加入对象的可变性与快照所有权 | 语义差异 / 迁移决策 | P1 |
| PY-16 | 字段名别名和默认 key 推导 | 语义差异 | P1 |
| PY-17 | Deck 使用方式与自动图片遮挡身份 | 功能未暴露，部分已有替代 | P1/P2 |
| PY-18 | Field.optional 的声明及回读 | 语义差异，已有等价构建能力 | P2 |
| PY-19 | 路径及失败后的发布语义覆盖 | 未验证 / 共有核心约束 | P1 验收门槛 |
| PY-20 | 公共能力对等、安装后验证与版本可观测性 | 未验证 / 部分接口未暴露 | P1/P2 |

## 逐项证据、影响和验收

### PY-01：独立 Project.validate / ValidationReport

**现状与 Rust 预期。** Python 无 `Project.validate()`；`to_product_document()` 只执行部分本地快速验证，构建才进入 Rust pipeline。Rust [`Project.validate()`](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L177)返回汇总报告，包含作者层错误和警告，例如隐式字段 key、模板 filter 警告；Python 的 [`NoteType.validate()`](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/notetype.py#L260)返回自身并抛首个本地异常，不是对应的汇总 API。

**影响。** 编辑器、批量导入或构建前检查无法取得与 Rust 一致的作者层报告；通过实际导出替代校验有副作用，也不能保证拿到同一组 facade diagnostics。

**验收。** 对同一逻辑项目，Python `validate()` 与 Rust `Project.validate()` 的 diagnostic code、severity、source、可用 span/help 一致；校验不创建 APKG、不写 lockfile、不污染 project。不得把 Rust `validate()` 本来没有执行的所有 normalization/媒体检查都宣传为本接口保证。

### PY-02：add 时校验和结构化作者错误

**现状与 Rust 预期。** Python `add_note()` 只立即检查类型和部分字段名，重复 stable ID 到序列化才报错；`add_notetype()` 不执行 Rust 模板引擎校验。[Python 添加](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L79)、[延迟 stable ID 检查](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L208)。Rust 添加会在变更状态前验证重复身份、模板语法/字段引用、重复模板名、Cloze 一致性，错误带稳定 code 和诊断位置。[Rust add](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L119)、[模板检查](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L547)、[重复身份检查](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L698)。Python `ValidationError` 目前只是普通 `ValueError` 子类。[错误定义](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/diagnostics.py#L11)。

**实测。** Python 第二次添加同一 stable ID 成功、notes 数量为 2；Rust 第二次添加返回 `AFID.STABLE_ID_DUPLICATE`。Python 注册 `{{Missing}}` 模板成功；Rust 返回 `TEMPLATE.RENDER_FIELD_UNKNOWN`。

**影响。** 错误暴露时机和失败后的对象状态不同；调用方只能匹配 Python 错误文本，不能统一使用 Rust diagnostic codes。

**验收。** 建立 add-time case 表，对重复 ID、未知类型/字段、重复模板名、错误 Cloze filter 和模板语法逐一比较；拒绝后项目保持原状。Python 异常保留 `ValueError` 兼容性，同时提供 `code`、`diagnostic` 等稳定数据。必填字段缺失等 Rust 本来留到构建阶段的检查，应保留其实际边界。

### PY-03：template bundle 导入

**现状与 Rust 预期。** Python 无模板包入口。Rust [`Project.import_template_bundle()`](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L135)读取 manifest、模板 HTML、浏览器模板、CSS 与资产，保留文件来源；全部成功后才提交类型和 staged media。manifest 支持普通/自定义 Cloze、字段与模板 key、生成规则、目标 deck。[bundle 模型](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/template_bundle.rs#L74)、[文件源映射](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/template_bundle.rs#L307)。

**影响。** Python 用户要手动读取、转换和注册全部内容；这不是原子导入，也不保证路径安全、限制和原始文件诊断位置与 Rust 相同。

**验收。** Normal/Cloze bundle 在 Python 与 Rust 产生一致的模型、卡片、媒体和身份；覆盖相对路径、路径穿越/越界 symlink、无效 UTF-8、大小上限、坏 manifest、重复类型与资产冲突；任一步失败均不部分修改项目；错误保留 code、源文件和可用 byte offset。直接复用 Rust importer，避免维护第二套解析器。

### PY-04：note 级 identity 覆盖与类型级独立 recipe

**现状与 Rust 预期。** Python 可标记 `Field(identity=True)`，序列化后变成类型级字段 recipe，也支持显式 stable ID；但没有 note 级 recipe，类型级 recipe 也不能与字段 identity 元数据独立表达。[Python 类型身份投影](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/product_json.py#L234)、[Note 数据结构](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/note.py#L24)。Rust 有 [`IdentityRecipe::fields`](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/identity.rs#L9)、[`NoteType.identity`](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/notetype.rs#L157)、[`Note.identity`](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/note.rs#L98)。每笔记覆盖有独立 recipe ID、provenance、`used_override`。[Rust 身份解析](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L1889)。当前 product transport 的 custom note 没有对应 identity 字段。[传输模型](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/model.rs#L234)。

**影响。** 同一类型不同笔记无法选择不同字段身份规则；手工写 stable ID 能固定身份，但不能保留同一 recipe / provenance / override 证据。不能简单把 hash 结果塞进 stable ID 就宣称完全对等。

**验收。** 显式 stable ID、note recipe、类型 recipe、已有 `Field(identity=True)` 兼容路径分别有用例；选择字段的顺序与重复项遵循 Rust 排序去重；比较最终 GUID、lockfile recipe/provenance、内容修改后的身份稳定性和错误字段引用。若保留 JSON transport，应增加作者语义桥接，不能只在 Python 重新实现身份 hash。

### PY-05：模板/CSS 多行内容与空白保真

**现状与 Rust 预期。** Python 将模板 front/back/browser source 当作普通非空名称验证，拒绝所有 ASCII 控制字符，包含合法 `\n`、`\r`、`\t`，同时裁剪首尾 ASCII 空白；CSS 也拒绝换行。[验证函数](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/notetype.py#L19)、[模板构造](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/notetype.py#L149)、[CSS setter](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/notetype.py#L231)。Rust 保存模板字符串和 CSS，再由模板引擎判断语义。[TemplateSource](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/template.rs#L30)、[front/back](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/template.rs#L122)、[CSS](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/notetype.rs#L152)。

**实测。** `Template(..., front="{{Front}}\n<div>extra</div>", ...)` 和 `css(".card {\n color: red;\n}")` 均在 Python 抛错；Rust 相同内容注册及导出成功。Python `"  {{Front}}  "` 被改为 `"{{Front}}"`。

**影响。** 正常的手写 HTML/CSS 无法直接使用；裁剪会改变模板源码，影响源码位置和要求保真的内容。

**验收。** front/back/browser templates/CSS 保留输入的全部字节对应文本；覆盖 LF、CRLF、tab、首尾空白、Unicode；对非法内容由相应语义校验决定，而非复用名称校验。对现有单行模板不改变生成结果。此项可独立修复，不必等待绑定架构重写。

### PY-06：文件媒体注册、指纹与源变化

**现状与 Rust 预期。** Python `add_file()` 解析路径但不读文件，保存 `length=None, sha256=None`，连不存在路径也接受；构建时复制当时的文件内容。[注册](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/media.py#L71)、[复制](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L190)。Rust 注册时验证普通文件、可读、非空并保存内容指纹；构建检查是否改变，返回 `MEDIA.SOURCE_CHANGED`。[Rust 注册](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/media_registry.rs#L135)、[复核](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/media_registry.rs#L199)、[注册读取](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/media_registry.rs#L333)。

**实测。** Python 注册后修改 WAV 源文件，重新构建的 runtime 返回 `success` 且无 diagnostics；缺失文件注册也成功。Rust 缺失文件注册直接失败，且已有[源变化回归](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/tests/project_media_api_tests.rs#L1159)。

**影响。** 一次构建可能悄悄使用注册后变化的媒体，削弱复现与更新安全；错误推迟到复制/normalization，位置及错误类型也不同。

**验收。** 覆盖 missing/directory/unreadable/empty、同长度内容替换、mtime 不变内容变化、注册后删除/清空、多个导出名引用同一源文件；拒绝时保留既有 output/lockfile，报告指向原媒体声明。指纹校验需覆盖实际消费的内容，不能留下“先 hash、随后不受约束地重新 copy”的窗口。注册时与构建时证据的生命周期需由实现方案明确。

### PY-07：内存媒体界限、空输入及内容去重

**现状与 Rust 预期。** Rust `add_bytes` 拒绝空输入和超过 65,536 bytes 的 inline payload；同名同内容允许复用，无论来自另一路径还是 bytes。[限制](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/media_registry.rs#L129)、[内容去重](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/media_registry.rs#L234)。Python bytes 注册只计算 hash，不执行空/上限检查；同名 bytes 可去重，但同名不同文件路径直接报冲突，file/bytes 也无法按内容统一。[Python bytes](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/media.py#L47)、[Python file](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/media.py#L71)。

**实测。** Python 接受 0 和 65,537 bytes 的注册；前者构建成功并仅有 `MEDIA.UNKNOWN_MIME`，后者到构建才 `MEDIA.INLINE_TOO_LARGE`。两条不同路径包含相同 bytes 且导出同名，Rust 返回同一文件名引用，Python 报错。

**影响。** 校验时机与允许的输入不同；重复源文件的批量导入会被不必要拒绝，过大 bytes 的编码成本发生在报错之前。

**验收。** 测试 0/1/65,536/65,537 边界；大文件经 `add_file` 仍可构建；file/file、file/bytes、bytes/bytes 同名同内容均按 Rust 规则去重，同名不同内容报稳定冲突码。明确公开 inline limit 的读取方式，不把缺少常量 accessor 单独夸大成构建能力缺失。

### PY-08：MediaRef 局部序号碰撞错绑

**现状与 Rust 预期。** Python 每个 registry 从 `media:000001` 起计数，`Note.sound/image` 最终只保留 media ID，Project 只检查该 ID 是否存在；原引用的 `export_as` 被丢失。[序号](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/media.py#L96)、[字段内容](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/note.py#L17)、[存在性检查](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L240)、[序列化](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/product_json.py#L253)。Rust Product `MediaRef` 明确按导出文件名相等，渲染仍引用原文件名。[Rust 合约](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/media_registry.rs#L13)。

**实测。** A 项目第一个媒体 `a.wav` 的 ref 用于 B；B 第一个媒体是 `b.wav`，二者 ID 均为 `media:000001`。Python 接受引用，JSON 指向 B 的 ID，构建成功且无 diagnostics，因而把 a 的引用绑定为 b。

**影响。** 合法外形的数据会产出错误媒体，普通成功/失败检查无法发现，应优先修复。

**验收。** 相同局部序号、不同 filename 的跨项目引用不得成功错绑。完整对齐按 filename 解析：目标项目缺少该 filename 时明确失败；目标已注册该 filename 时按 Rust 语义使用目标注册内容。同项目引用、手动构造 MediaRef 的现有行为也需定义。直接拒绝所有外部 registry ref 可作临时止损，但不算完整对等关闭；若新增 owner token，这是比当前 Rust Product 更强的限制，不可写成“Rust 本来就拒绝一切跨项目 ref”。Rust 同 filename 的跨 registry 引用也不携带所有权或内容身份，这部分是共有边界。

### PY-09：构建选项、InspectLimits 和保留工件

**现状与 Rust 预期。** Python 唯一公共构建方法的参数集不含 `artifacts_dir`、`inspect`、`inspect_limits`、`self_contained` 或 normalization 配置。[Python 签名](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L132)。Rust `BuildOptions` 支持这些入口，[具体方法](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/build/options.rs#L96)，`InspectLimits` 在 prelude 明确导出，包含 archive/entry/expansion/zstd 等 11 项有限预算。[预算字段](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/writer_core/inspect_limits.rs#L8)。

**影响。** 用户无法控制受信任大型牌组的检查预算、保留 staging/CAS 证据、选择媒体模式或报告 inspect 信息。

**边界修正。** Python 当前仍使用 Rust 默认检查和媒体策略，不是没有资源保护。`inspect(false)` 控制报告中的 inspect 摘要，不应宣传为关闭最终安全检查：[pipeline 始终执行比较/检查，随后条件保存摘要](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project/pipeline.rs#L485)。`ProjectNormalizeOptions`、高级 media policy 枚举和 `RiskLevel` 没有在当前 prelude 命名导出；补齐其完整配置应同时澄清 Rust 支持面，不能简单将隐藏类型全部当作已有、易用的 0.1 公共接口。[prelude](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/prelude.rs#L7)、[配置类型](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/build/options.rs#L5)。

**验收。** 明确 Python options 类型/参数；逐字段映射已承诺选项；current/baseline 使用同一 limits；低限额触发 `INSPECT.RESOURCE_LIMIT_EXCEEDED`，适当提高后同一 fixture 成功；`artifacts_dir` 持久保留且重复构建可用；`self_contained` 与 path-backed 的解码内容和身份一致，不要求 ZIP byte/hash 一致。高级策略的枚举、默认值与哪些值仍被禁止需独立列入合同表。

### PY-10：临时 ApkgArtifact 与 persist_to

**现状与 Rust 预期。** Python `write_apkg(path)` 强制明确输出，report 仅含 path 字典。Rust `build(BuildOptions::new())` 返回拥有临时 APKG 的 handle，最后一个持有者释放后清理，可用 `persist_to` 原子复制为永久工件。[Rust 生命周期说明](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/README.md#L73)、[artifact 实现](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/build/artifact.rs#L33)。

**影响。** 临时预览、上传前构建和服务端消费需要调用方自行管理文件和错误清理，无法复用 Rust 的生命周期合同。

**验收。** 支持无 output 构建；artifact 存活时 path 可读；复制 handle/报告或异常持有时不提前删除；最后一个 owner 清理；显式 output/artifacts_dir 和 persist 后目标不会被清理；持久化到临时源自身路径拒绝。Python 可提供 context manager/close 等习惯接口，但不能只暴露一个退出 subprocess 后已失效的临时路径。report JSON 的临时产物限制与晚期失败取证也须保持。

### PY-11：bytes 与有界文件对象输出

**现状与 Rust 预期。** Python 仅直接输出文件；用户自行读取该文件可取得 bytes，因此不是无法取得包内容。Rust `Deck.to_apkg_bytes()` 和 `Deck.write_to(Write)` 将这类工作流封装，并在 `write_to` 复制阶段使用固定 64 KiB 缓冲。[Rust 导出](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/deck/export.rs#L94)、[有界复制](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/deck/export.rs#L139)。

**影响。** 上传、响应体、BytesIO 等用途缺少一致的清理/错误传播；草率使用全文件 read 实现 `write_to` 会失去已有 Rust 的有界复制性质。

**验收。** `to_apkg_bytes` 返回可检查 APKG bytes；`write_to` 支持二进制 file-like、非 seekable sink、短写/写失败，并清理临时工件；大包复制过程不额外创建整包 bytes。准确描述为“先构建工件，再有界复制”，不要称为构建与网络发送同时进行的真正增量产包。

### PY-12：独立 diff

**现状与 Rust 预期。** Python 只能 `write_apkg(compare_to=...)` 获取 diff；没有对应 [`Project.diff_against_apkg`](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L875) 的公共操作。Rust 该方法内部仍临时构建 current APKG，并以 report-only 模式比较；它不是完全不生成包。[临时构建细节](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L888)。

**影响。** 用户只想查看变化也要选择发布输出路径，或者自己模拟临时目录和生命周期。

**验收。** 不要求用户提供输出路径、不发布 output、不推进 lockfile，返回完整 current/previous inspect、diff、risk、comparison status、diagnostics、metrics；missing/unreadable/invalid baseline 行为与 Rust 方法一致；支持 limits；成功与失败均清理内部临时包。

### PY-13：报告 metrics / policy 被验证后丢弃

**现状与 Rust 预期。** Rust `BuildReport` 有 metrics 和 policy。[Rust report](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/build/report.rs#L110)。Python `from_json` 调用 `_metrics/_policy` 验证后不保存，public dataclass 也没有这些字段。[Python report](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/report.py#L28)。内部 `_report_to_json` 固定生成 duration 0 和 `not_evaluated` policy；它目前只被测试使用，不能因此断言实际 `report_json` 文件被损坏。[辅助函数](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L313)、[测试调用](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/tests/test_runtime_product_build.py#L158)。实际 report 文件直接由 Rust 写入，保留原信息。

**实测。** runtime 文件中 policy 为 `blocked/high/RISK.BASELINE_UNAVAILABLE`、duration 为 10 ms；Python 对象没有 `metrics` / `policy`；内部再编码给出 `not_evaluated`。

**影响。** 调用方无法在内存中判断精确阻断原因和阈值、观测耗时；自行再编码不能保持证据。

**验收。** public report 保留完整 metrics/policy 和相关协议 provenance（至少 tool/schema version）；success/blocked/invalid/error 四类报告往返不丢受支持字段；`DiagnosticsError.report` 同样完整；保留现有 Mapping 访问兼容。pretty report、warning count、diagnostic code helpers 可作为便利层，不是独立核心能力缺失。

### PY-14：fail_on 错误要求 compare_to

**现状与 Rust 预期。** Python 预检规定 `fail_on is not None and compare_to is None` 必须失败，[代码](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L262)，并有[测试固定该限制](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/tests/test_product_validation_parity.py#L50)。Rust 风险策略也能针对 identity lockfile 的基线问题执行，并非只针对 APKG diff。[策略执行](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project/pipeline.rs#L486)。

**实测。** `identity_lockfile=missing, update_safety="report_only", fail_on="high"` 无 compare_to 时，Python 抛本地 ValidationError；同一 product document 直接走已有 runtime 返回 `blocked`、policy threshold/highest_risk 为 high、finding 为 `RISK.BASELINE_UNAVAILABLE`。

**影响。** 用户无法使用已有 lockfile-only 风险门槛，即使底层已支持。

**验收。** 删除过度预检并按 Rust 语义处理：无 baseline、仅 lockfile、仅 compare_to、两者同时、各种 update mode 组合；该 probe 应返回带完整 policy 的 blocked report，既有 output/lockfile 不变。保持未知 risk level 等真正无效参数的验证。

### PY-15：作者对象可变性是迁移合同，不可悄悄改变

**现状与 Rust 预期。** Python Project 保存传入对象本身；`notes` 返回 tuple、`notetypes` 返回只读 mapping，但内部 Note / NoteType 仍可变，后续序列化读取最新状态。[保存对象](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L67)、[再验证可变类型](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L208)。现有测试特意覆盖添加后修改 NoteType 的情况。[测试](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/tests/test_product_api_model.py#L221)。Rust Project 接收 owned Note/NoteType，外部 clone 不会反向改变 Project。[Rust 添加](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L119)。

**实测。** `p.add_note(note); note.text("front", "after")` 后，p 的 product document 已变为 `after`。

**影响。** 这是既有 Python 产品选择，不自动等于缺陷。若未来 native binding 在 add 时复制 Rust 对象，原脚本可能继续运行却悄悄导出旧值；换底层实现不等于无损兼容。

**验收。** 实施前明确 retain-live-reference、immutable snapshot 或受控 replace/update 的合同。为 add 后改字段/标签/deck/stable_id、增删字段/模板、同一对象加入多个项目、失败后状态各写 migration case；旧版本行为保留或通过明确版本迁移/报错处理。建议优先让存储和桥接方案服从选定的公共合同，而不是由绑定技术默认决定。

### PY-16：字段名别名、默认 key 与跨语言身份

**现状与 Rust 预期。** Rust 自定义 note 支持字段 key 或显示 name，[验证](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L754)；Python 只允许 key，[验证](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L245)。Rust Field 逐字符替换非 ASCII 字母数字，Template 主要小写并替换空格；Python 两者统一使用会折叠连续分隔符的 slug。[Rust Field](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/notetype.rs#L191)、[Rust Template](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/template.rs#L103)、[Python slug](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/notetype.py#L49)。Python 也未保留 key 是否自动推导，失去 Rust `NOTETYPE.FIELD_KEY_AUTO_DERIVED` 的对应作者层警告。[Rust 警告](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L358)。

**实测。** 字段名 `Back  Extra`：Rust `back__extra`，Python `back_extra`；模板名 `C++ Card`：Rust `c++_card`，Python `c_card`。Rust `.text("Front", ...)` 可填 key 为 `front` 的自定义字段。

**影响。** 同一作者输入跨语言可能形成不同稳定配置 key，影响模型/模板身份和迁移；用显示名填字段的 Rust 示例不能直接转换。

**验收。** 明确 key 推导版本与兼容策略，不直接改旧 Python 默认导致历史身份漂移；推荐新跨语言项目显式 key。验证多个空格、标点、非 ASCII、显示名和 key 同时出现等歧义；如果承诺自动推导对等，需比较最终 Anki 模型/模板 ID 及更新报告，而不只比较字符串。

### PY-17：Deck facade、Project 迁移和自动 IO 身份

**现状与 Rust 预期。** Python 只有 Project。普通单 deck 的 Basic/Cloze 可以用 Project + Note 表达，不应列为“无法制卡”。Rust [`Deck` lanes](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/deck/builders.rs#L51)还提供快速添加、验证、媒体入口、bytes/writer 导出，以及 [`Project::from(deck)`](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/README.md#L75) 的可编辑升级路径。

**额外的实质能力。** Rust Deck 图片遮挡可以不提供 stable ID，根据图片 hash、图片尺寸、模式和排序矩形推导身份，并在这个推导路径检查矩形边界。[推导](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/deck/identity.rs#L231)、[没有 stable ID 的添加路径](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/deck/builders.rs#L259)。Python 图片遮挡 builder 与 Rust Project builder 都要求显式 stable ID；不能拿 Project 行为覆盖 Deck 的能力。[Python 限制](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/note.py#L164)。

**验收。** 若新增 Deck，Basic/Cloze/两种 IO 模式、媒体与导出复用共享实现；不写 IO stable ID 时的结果与 Rust Deck 相同；图片尺寸缺失、越界矩形有同等错误；升级 Project 后可继续加 custom type/bundle/media，原 HTML、身份、媒体和诊断来源不变。Deck identity policy 的部分参数类型也尚未在 prelude 重导出，额外配置应先确定支持合同，不能无差别搬出内部 API。

### PY-18：Field.optional 不是必填行为缺失

**现状与 Rust 预期。** Rust Field 区分默认 `required=false, optional=false` 与显式 optional，并能 `is_optional()` 回读；Python 只有 `required`。[Rust Field](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/notetype.rs#L17)、[Python Field](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/notetype.py#L123)。但是当前 Rust 下放到 ProductFieldV2 时只传 required、不传 optional，[lowering](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project.rs#L1147)。因此 Python `required=False` 已能表达相同的非必填构建行为。

**影响。** 缺少显式作者意图和同构回读；不能把它写成“Python 不支持可选字段”。模板包的 required/optional 冲突仍需按 importer 规则处理。[bundle 冲突规则](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/template_bundle.rs#L222)。

**验收。** 若提供 optional，required/optional 互斥和最后一次设置规则明示；验证对象回读、模板包导入；相同 `required=False` 项目在新增前后输出行为不变。优先级低于媒体与身份差异。

### PY-19：路径、原子发布及失败后的证据

**当前已有。** Python 解析路径并做本地冲突检查，Rust pipeline 继续执行 hard link/symlink 和 staging 保护，不是只依赖字符串相等。[Python 路径预检](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/project.py#L144)、[Rust 路径计划](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/product/project/paths.rs#L34)。基线只读、风险阻断保留已有输出和 lockfile、每文件原子发布已有正式合同。[Rust README](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/README.md#L116)。

**尚需验收的部分。** 新增临时 artifact、artifacts_dir、report/diff、原始媒体路径桥接后，路径集合会扩大。Python 的 `Path.resolve()` 还可能影响输出 symlink 的表现；本次没有证明其相对 Rust 是缺陷，不应无依据承诺一致。媒体输入与发布目标任意 alias 的保护也不能从上述基线防护类推得到。

**验收。** 对 output/baseline/report/lockfile/artifacts/staging 的同路径、相对路径、symlink、hard link、目录别名矩阵做对等验证；包括晚期 report/lockfile 写失败、路径创建后变化以及已发布 artifact 出现在失败 report 中。公开语义为“每文件原子”，不得宣传多文件事务。安装后从含空格和中文路径运行同样的关键用例。

### PY-20：能力对等和发布覆盖

**当前已有。** 测试覆盖对象模型、JSON fixtures、runtime 参数/协议、真实 product 构建、源/runtime 打包和安装隔离；wheel 矩阵已经存在。默认 CI 本地脚本执行 Python 测试时忽略安装隔离，后者在独立 wheel job 执行。[本地 CI 命令](../../scripts/verify-ci.sh#L89)、[安装隔离](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/tests/test_import_isolation.py#L31)、[wheel smoke](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/.github/workflows/contract-ci.yml#L235)。`pyproject.toml` 要求 Python >=3.11，包含内联类型检查配置。[Python 支持](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/pyproject.toml#L1)。

**缺口。** 现有名为 parity 的测试不少是 Python 快速预检断言，并不执行 Rust facade 对照，有的固定了 PY-14 的旧差异。当前没有足够覆盖来证明所有新能力与 Rust 对等。Python 根包也没有 Rust `facade_api_version()` / `embedded_contract_version()` 对应查询；package version 可用 Python 标准包元数据取得，但 runtime/contract version 需要可用入口。[Python 导出](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/python/src/anki_forge/__init__.py#L18)、[Rust 版本轴](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/anki_forge/src/lib.rs#L120)。本次源码文件列表未见 `py.typed`，发布侧应检查 wheel 类型标记，而非仅看源码 annotations。

**验收。** 建立以能力为行的公共 API 对照 fixture；比较解码后 notes/cards/fields/templates/media、稳定 IDs、diagnostic codes、diff/risk/policy/update evidence，排除耗时、临时绝对路径和允许变化的 ZIP 顺序。新增接口必须在清洁 wheel 环境运行关键路径；保留 legacy 包隔离、bundled runtime/contract 资产校验、平台矩阵、typing consumer smoke，并增加 SDK/runtime/contract 版本可观测性。不能以源码环境测试全绿代替打包可用，也不能要求不同存储模式 APKG 字节完全相同。

## 本次验证记录与置信边界

本次执行了 `cargo build -p contract_tools --release --locked`，命中缓存并成功；所有 probe 使用临时目录，没有改实现。除静态审查外，运行了公共 Python API probes、经现有 runtime 的真实导出，以及只使用 `anki_forge::prelude::*` 的 Rust 对照 probe。

| Probe | 观察结果 |
|---|---|
| 自定义 Cloze | Python success，1 note / 2 cards |
| 多行模板/CSS | Python 构造时拒绝；Rust 注册并导出 success |
| 模板首尾空格 | Python 被裁剪 |
| 注册后改变文件 | Python success，无 diagnostics |
| A a.wav ref 用于 B b.wav 且序号相同 | Python success，指向 B 的 media ID |
| 缺失文件注册 | Python 接受；Rust `MEDIA.SOURCE_MISSING` |
| 空 bytes | Python 接受且导出 success；Rust注册拒绝 `MEDIA.EMPTY_SOURCE` |
| 65,537 bytes | Python 注册接受、构建报 `MEDIA.INLINE_TOO_LARGE`；Rust 注册即拒绝 |
| 同名同内容不同文件路径 | Python拒绝；Rust去重 |
| 重复 stable ID 添加 | Python接受后序列化拒绝；Rust add即拒绝 |
| 不存在字段的模板 | Python add接受；Rust `TEMPLATE.RENDER_FIELD_UNKNOWN` |
| lockfile-only + fail_on high | Python预检拒绝；现有 runtime 正确 blocked |
| 报告 policy/metrics | Rust写出的文件完整，Python对象无对应字段 |
| add后修改原Note | Python Project随之改变 |
| 默认key | 已确认 Field/Template 两类推导差异 |

会话此前报告 Python 107 项与 5 个子测试通过，本次没有重复全套或重建 wheel；它们不是上述新验收项已经通过的证据。本次也未验证真实 Anki Desktop 导入、全部操作系统、Python 3.13+、全量故障注入或新 API（尚未实现）。

## 不应纳入“Python 必须补齐”的项目

- `internal-tools` 的 IR、底层 writer、inspect、persistence、helpers/font bindings 等，仅因源码 `pub` 不成为 0.1 公共承诺。直接 `TemplateEngine` 校验若希望与 Node SDK 统一，可作为扩展提案；本审计要求的是现有 `Project.validate` 和 add/import 内部执行的模板校验。
- 缺少与 Rust 同名的 getter、`Content`、`FieldKey`、`TemplateKey` 类，不自动等于缺能力；Python 字符串、dataclass 属性、现有方法可提供同等表达。仅在信息或语义丢失时计入差距。
- `first_update_safe_build()` / `update_safe()` 没有同名快捷方法，但现有 keyword 参数已能组合出 lockfile 的主流程；PY-14 才是确认的组合限制。
- Rust Product MediaRef 本身按文件名而非 registry ownership；跨 registry 同名内容隔离是共有边界。PY-08 的不同 filename 被序号错绑则是 Python 独有问题。
- `hide_one_guess_one` 的 builder 入口已存在，但目前生成的分组 `c1,2` 标记会被核心构建管线以 `PRODUCT.CLOZE_MARKER_MALFORMED` 拒绝；这是共有核心限制，不能宣称两种模式都已端到端可用，也不应由 Python 单独重写 renderer 绕开。[现有核心对接回归](https://github.com/morehardy/anki-forge/blob/2edb8cc60e928b0d9cc161a2fcdc01234b722983/bindings/node/test/product.test.mjs#L360)。
- Rust `Field.optional` 当前不影响与 `required=False` 不同的 lowering 结果；修声明/回读不应被宣传为新增“可选字段制卡”。
- Rust 部分配置参数类型未在 prelude 重导出，是 Rust 公共面的易用性问题。若实施时修订导出合同，应记录为配套 Rust 工作，不把 Node 使用 `internal-tools` 得到的所有能力自动算进 Python 对等承诺。
