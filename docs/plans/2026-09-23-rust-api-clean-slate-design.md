# Rust API 目标设计：不保留历史兼容层

日期：2026-09-23。状态：2026-09-24 实施中，核心 API 与全部调用者已迁移，分发和最终门禁尚未完成；尚未发布。

本方案以最佳设计和使用体验为方向，不考虑历史兼容。旧源码、旧调用方式、旧 JSON 形状和历史成功输入均不是保留条件。当前源码和测试用于识别设计差距，实施以本文的目标模型、工作顺序和验收要求为准。

本次用 design-an-interface 流程让三个 subagent 独立提出方案，再综合取舍。实施遵循本文完整目标；进展与验证记录见文末，不据局部检查宣称整体完成。

## 1. 判断标准

优先减少用户必须记住的规则、操作顺序和对象间的对应关系。不要只减少方法数量，却把资源管理、类型注册和更新证据管理交给调用者。

主要成功路径是：创建项目 → 添加有稳定标识的笔记 → 输出 APKG。自定义模型、图片、音频、图片遮挡和后续更新沿用同一条路径。

不保留旧接口，不意味着可以忽略新版本内的正确性：新设计生成的包必须能正确导入；从它继续更新时，仍要保护笔记和卡片身份。这是产品能力，不是历史兼容包袱。

## 2. 三个独立方案与选择

### A：尽量少的任务入口

核心形状是 `Project::new / add / build`。Note 持有模型，Content 持有媒体；Project 自动收集依赖。

```rust
let image = Media::file("cell.png")?;
project.add("cell", Note::basic(image.image(), "细胞"))?;
let output = project.build(BuildOptions::to("biology.apkg"))?;
```

它隐藏模型和媒体注册顺序，普通调用最短。单纯追求三个方法会使独立资产、模板导入、验证和比较难以表达，因此不把“三个方法”作为硬目标。

### B：模型和资源作为完整的值

自定义模型先完成校验，随后不可变；笔记从模型创建。媒体是可共享的资源值，不是某个 Project 内的临时句柄。

```rust
let model = NoteType::builder("vocab") /* 字段和模板 */ .build()?;
let note = model.note().field("front", "cell");
project.add("term:cell", note)?;
```

它隐藏字符串模型 ID 的注册协议与跨项目引用问题，同时支持在普通函数中返回和复用模型。代价是自定义模型多一个自然的完成步骤；这一步用于真正的 schema 校验，不只是更换名称。

### C：从首次生成一直到更新的使用流程

首次添加就明确稳定 key；媒体导入取得快照；构建请求区分首次生成和更新，更新从上一分发包读取完整证据。

```rust
project.add("cell", Note::basic("什么是细胞？", "生命的基本单位"))?;
project.build(BuildOptions::to("v1.apkg"))?;
next.build(BuildOptions::to("v2.apkg").update_from("v1.apkg"))?;
```

它把长期维护的规则提前到第一次使用，减少后续补 stable ID、锁文件和更新开关的迁移。代价是第一条笔记也需要 key，以及构建系统必须提供完整的包内更新证据。

**选择：以 B 的模型结构为基础，采用 A 的自动收集，采用 C 的显式身份、媒体快照和更新流程。** 容器命名选择 Project：它管理一次发布的内容，可以包含多个卡组；Deck 不再作为另一套创作容器。不采用带 Project lifetime 的字段/媒体句柄，也不采用复杂类型状态泛型。

## 3. 应当改变的接口与行为

### 3.1 删除现有 prelude 和双创作体系

- 常用类型直接从根路径导入：Project、Note、NoteType、Field、Template、Content、Media、BuildOptions、BuildOutput。
- 较少使用的类型按 `note / schema / media / build / update / diagnostics` 等领域组织，保证所有公开签名都可命名。
- 不再保留旧 `prelude::*`、旧名称别名、两套 MediaRef/ValidationReport，以及独立的 Deck/Lane/Draft 创作流程。
- 不设置通用 `builders` 模块；NoteTypeBuilder 和图片遮挡构造器归各自领域。
- 卡组由 `Project::default_deck` 和 `Note::deck` 表达。以后确实需要卡组选项对象时，它只是配置值，不拥有另一套 notes/media/build。
- 内部 lowering、writer、运行时协议只通过仓库工具实际需要的入口访问；不为了清除 lint 公开整个内部目录。

默认公开模块的允许列表固定为 `note / schema / media / build / update / diagnostics`，以及这些领域下明确声明的报告 DTO 子模块。所有公开参数、返回值、泛型约束和关联类型都有可用路径；不重新公开 `product / authoring / authoring_core / writer / writer_core / runtime / update_safety / deck` 等实现目录。

默认消费者不能调用 `Project::lower/normalize`，也不能通过其他公开函数获得 ProductDocument、LoweringPlan 或 authoring IR。旧 Deck 删除后，其三个 lowering 出口一起消失。内部实现需要的逻辑保留为私有 helper；仓库工具确实需要跨 crate 的操作时，在 `internal-tools` 下经精选 `tools` 模块提供。开启该 feature 不意味着整棵内部实现树公开。每个 tools 导出必须对应一个真实工具调用者和一个测试，没有调用者的出口删除。

这里删除 prelude 是本项目的设计判断，并不是 Rust 禁止库定义 prelude。当前常用操作不需要统一导入一组扩展 trait，根路径已能承担发现和导入入口。参考 [Rust 标准库 prelude 的用途](https://doc.rust-lang.org/std/prelude/index.html)。

### 3.2 自定义模型先验证，笔记携带模型

目标形状：

```rust
let model = NoteType::builder("vocab")
    .name("词汇")
    .field(Field::new("front").name("正面"))
    .field(Field::new("back").name("背面"))
    .template(Template::new("recognition")
        .name("识别")
        .front("{{front}}")
        .back("{{FrontSide}}<hr>{{back}}"))
    .build()?;
```

`build()` 返回内部共享、不可变的 NoteType。`model.note()` 创建持有模型的 Note；Basic/Cloze 也持有内置模型，所有笔记进入同一种存储。

`Project::add(key, note)` 原子地加入笔记并收集模型、媒体：任一校验失败都不留下部分登记。同一模型 key 的相同定义复用，不同定义明确报错。用户不再先 add_notetype，再用 `Note::new("vocab")` 建立可能悬空的关联。

模板 bundle 的加载返回同一种 NoteType，并携带资产；不再以“修改 Project 一部分状态”的方式导入。模型独立合法性、项目内冲突、最终构建约束分别在模型完成、添加笔记、构建时检查。

**未完成值的提醒仍然保留。** 删除 PendingMedia 不等于删除所有构造器的误用检查：

| 值类型 | `#[must_use = "..."]` 应说明的动作 |
| --- | --- |
| `schema::NoteTypeBuilder` | 调用 `.build()` 完成模型校验 |
| `note::ImageOcclusionBuilder` | 调用 `.build()` 得到经过局部校验的 Note |
| `Note` | 调用 `Project::add(key, note)` 加入项目 |
| `NoteType` | 用 `.note()` 创建笔记，或保存/返回模型供后续使用 |
| Field、Template、Content、Media | 传给相应声明、内容或资产入口；单独构造不会加入项目 |
| BuildOptions、CompareOptions、UpdatePolicy | 传给相应操作或配置入口；配置值本身不执行任务 |

IO 构造器的 `.build()` 检查图片和 mask 的局部合法性；Project::add 再检查项目内关系。消费 `self` 的配置方法依靠返回类型的 must_use 发现丢弃结果。显式 `let _ = ...` 或 `drop(...)` 可以放弃值，drop 不添加笔记、不注册模型、不发布包；媒体暂存资源正常释放。不强制读取持久构建的 BuildOutput，因为只要输出文件是合法用法；临时输出必须保留产物句柄才能延长文件寿命。

### 3.3 key、显示名称和身份彻底分离

- `Field::new`、`Template::new` 接收稳定 key，`.name(...)` 设置显示名称；未指定 name 时使用 key 作为显示名称。
- 删除从显示名称 slug 派生 key 的规则，拒绝空 key、空白 key 和语法冲突，不再保留“中文派生为空但有时成功”的行为。
- FieldKey/TemplateKey 各自是不同的符号类型。接受 `Into<FieldKey>` 的方法不接受 TemplateKey；字符串字面量仍可直接使用。
- `From<&str>` 保留原值并且不失败，不声称 key 对象本身已经合法。统一在 schema 完成时验证声明，添加笔记时验证其字段引用；不让每个 setter 都返回 Result。
- 字段赋值只按 key 查找，不同时接受 key 和显示名称，不做静默 trim、大小写折叠或 Unicode 改写。

FieldKey 和 TemplateKey 各自提供以下完整转换矩阵；这里的 `K` 仅指当前这一种 key，不是跨所有字符串类型的 blanket 实现：

| 输入/访问方式 | 约定 |
| --- | --- |
| `From<String> for K` | 接管原字符串，不改值 |
| `From<&str> for K`、`From<&String> for K` | 复制原字符串 |
| `From<Cow<'a, str>> for K` | owned 时接管，borrowed 时复制 |
| owned `K` | 通过标准同类型转换直接传入 |
| `From<&K> for K` | 克隆符号值，原 key 可继续使用 |
| `as_str()`、`AsRef<str>`、`Display` | 只读原值 |

不提供 `From<TemplateKey> for FieldKey` 或反向转换，也不提供 `From<T: AsRef<str>>` 这种会让两种 key 隐式互通的实现。无需 Deref 或先退回 String 才能组合接口。

以下入口统一消费对应 key，避免只修 Note::field：

```rust
Field::new(key: impl Into<FieldKey>) -> Field
Template::new(key: impl Into<TemplateKey>) -> Template
Note::field(self, key: impl Into<FieldKey>, value: impl Into<Content>) -> Self
NoteTypeBuilder::cloze_field(self, key: impl Into<FieldKey>) -> Self
GenerationRule::all<I, K>(fields: I) -> GenerationRule
GenerationRule::any<I, K>(fields: I) -> GenerationRule
// 上面两个集合入口均要求 I: IntoIterator<Item = K>, K: Into<FieldKey>。
Template::generate_when(self, rule: GenerationRule) -> Self
```

cloze_field 将自定义模型声明为 cloze 模型；引用、模板和生成规则的匹配在模型 `.build()` 时统一校验。原 identity recipe 已有意删除，不为保持旧调用方式重新引入。

项目构造的首参数是稳定 namespace，显示标题单独设置；`Project::new` 验证 namespace。添加笔记的 key 必填，重复 key 报错，不默默覆盖。正文修改、显示名称修改、媒体内容修改不自动成为另一条笔记。

主接口删除隐式内容身份和自动 recipe 回退。没有业务主键的调用者需要选择并持久化 key；若明确要内容寻址，可以在数据导入层显式计算 key。这个负担是真实的，但从内容推导身份无法同时保证内容修改后的稳定更新。

模型、字段、模板、笔记的身份只依赖稳定 namespace/key 及版本化映射，显示名称不进入身份哈希。更新时使用基线维护 Anki 所需的 ID 和 ordinal；不把所有 Anki 身份都简单替换成自行计算的哈希。

类型区分用于防止符号混用；schema 校验用于处理运行时数据。两者不混为“全部错误都由编译器消灭”。参考 [Rust API Guidelines 的类型区分建议](https://rust-lang.github.io/api-guidelines/type-safety.html)。

### 3.4 模板也使用稳定 key

作者写 `{{front}}`，模板编译器绑定到 FieldKey，再生成 Anki 的 `{{正面}}`。Note 赋值、模板、卡片生成规则使用同一套符号，不要求用户维护两套引用。

此处需要真正的编译步骤：

1. 扩展现有模板解析器的源区间信息，定位字段引用；当前 token 并不足以无损重写所有原文。
2. 覆盖正反模板、browser 模板、条件段开闭、过滤器和 cloze；保留 `FrontSide` 等系统表达式。
3. key 和显示名称的冲突、系统保留符号冲突、未知引用在模型完成时报告。
4. 按精确区间重写引用，保留其他 HTML/CSS/脚本、注释和空白；不使用全局字符串替换。
5. 诊断位置指回用户原始模板。现成 Anki 模板的导入需要显式名称绑定步骤，不靠同时猜测 key/name。

名称修改会产生模型差异，仍需更新检查；“ID 不变”不能扩展成“任意模型结构修改都无风险”。

### 3.5 统一 Content，直到输出时才渲染

```rust
Note::basic(front: impl Into<Content>, back: impl Into<Content>)
Note::cloze(text: impl Into<Content>)
Note::field(self, key: impl Into<FieldKey>, value: impl Into<Content>) -> Self
```

这是目标签名草案。普通字符串统一转换成 Text；HTML 必须 `Content::html(...)`。Cloze 的 `{{c1::...}}` 仍保留 cloze 语法，字符串内的 HTML 不再因笔记种类而默认改变含义。

Content 使用私有表示，内部保留 Text、Html、Image、Sound、Sequence 等语义节点。`image.image()`、`audio.sound()` 不再提前生成 HTML 丢失媒体关系；`Content::sequence(...)` 统一组合内容。删除含义不明确的 `Content::Media` 裸文件名分支及成套重复的字段 setter。需要 filename 时显式使用媒体对象的 filename。

公开层不要求调用者先 render 再 html。HTML 转义、媒体引用收集和 Anki 序列化统一发生在构建管线。

### 3.6 媒体作为拥有快照的值，自动收集

```rust
let image = Media::file("assets/cell.png")?;
let note = Note::basic(image.image(), "细胞");
project.add("cell", note)?;
```

- Media::file 成功时已经取得库拥有的内容快照；之后源文件被删除或修改不影响该 Media。
- 文件流式复制和计算摘要；较大内容暂存到磁盘，内部共享所有权。快照构造失败清理中间文件。
- 原生 bytes 入口接管调用者提供的数据，媒体类型明确传入；移除因内部 Base64 表示而产生的 64 KiB 上限。资源预算和 I/O 失败仍正常报告，存储阈值只是内部策略。
- 默认使用确定性的内容摘要与媒体类型生成安全导出名，避免不同路径的同名文件冲突。媒体身份与源路径脱钩。
- 确实需要固定导出名时显式指定；同名不同内容报错，同内容共享底层存储。不静默覆盖。名称冲突采用统一的跨平台规则，显式名称仅大小写不同也拒绝，不能依赖当前宿主文件系统恰好允许它们并存。
- typed Content 自动收集依赖。CSS、字体、脚本和手写 HTML 中的文件，通过 `NoteTypeBuilder::asset(media)` 或 `Project::add_asset(media)` 明确声明；库不从任意字符串猜测文件内容。
- 删除公开 MediaRegistry、PendingMedia、add_file → export_as 的两步提交协议；不再用 must_use 弥补漏提交。

固定名称和非 AST 资产采用下面这一条明确的调用路径。构造器使用默认预算，`with_limits` 变体支持在读取之前调整预算：

```rust
Media::file(path: impl AsRef<Path>) -> Result<Media, MediaError>
Media::file_with_limits(path: impl AsRef<Path>, limits: MediaLimits)
    -> Result<Media, MediaError>
Media::bytes(bytes: Vec<u8>, media_type: impl Into<String>) -> Result<Media, MediaError>
Media::bytes_with_limits(bytes: Vec<u8>, media_type: impl Into<String>, limits: MediaLimits)
    -> Result<Media, MediaError>
Media::with_export_name(self, name: impl Into<String>) -> Result<Media, MediaError>
Media::filename(&self) -> &str
NoteTypeBuilder::asset(self, media: Media) -> Self
Project::add_asset(&mut self, media: Media) -> Result<(), AddError>
NoteType::from_bundle(path: impl AsRef<Path>) -> Result<NoteType, TemplateBundleError>
```

Media 在构造时即确定默认 filename。bytes 的媒体类型使用 MIME 字符串；语法错误或与已识别内容明确不符时返回 MediaError，不能仅依靠导出名扩展名认定类型。`with_export_name` 校验名称并返回共享同一快照的新值；此前 clone 或 Content 已捕获的名称不随之变化，因此命名应在生成引用之前完成。

名称必须满足目标格式的安全要求，不包含路径、控制字符或不可移植的保留名称。跨资产冲突键统一采用 Unicode NFC 后的默认大小写折叠，导出原名不静默改写；原名不同但冲突键相同也拒绝，即使内容相同。完全相同名称和内容幂等，同名异内容报错。

名称语法错误在 with_export_name 返回；模型内名称冲突在 NoteTypeBuilder::build 返回 SchemaError；跨模型、typed Content 和显式资产的冲突在 Project::add/add_asset 返回 AddError。这些项目修改都具有单次原子性。显式资产即使没有被静态分析识别到引用也进入包，不能被“未使用资源清理”误删。

from_bundle 根据 manifest 读取完整资产闭包并取得快照后才返回模型；失败清理本次未共享的临时文件，不留下部分模型。它只接受本方案的新 bundle 格式。库可以验证可解析的本地引用，但不把任意 HTML/CSS/脚本扫描当作依赖来源。

新 bundle 明确使用 `anki-template.yaml` 和 `format_version: template-bundle-v2`。`note_type.key` 是模型 key；字段和模板的 `key` 必填、`name` 可选并默认等于 key。设置 `cloze_field` 即选择 cloze 模型，不再同时声明另一项 kind；删除 identity/optional 等重复或旧语义字段，未知字段报错。模板正文和 browser 模板统一引用字段 key。manifest 的 `assets` 每项声明相对于 bundle 的 path 和固定 export_as，css_file 声明样式文件。绝对路径、跨平台路径逃逸和指向目录之外的符号链接均拒绝。

manifest 限制 256 KiB，单个模板/CSS 文本限制 2 MiB，既预检长度也按实际读取量检查；文本必须是 UTF-8。资产采用前述 MediaLimits。模板错误保留原文件路径、UTF-8 字节位置和原 SchemaError；I/O、YAML、UTF-8 和资产错误保留其真实 source。可复用的 NoteType 在源目录删除后仍能正常加入其他 Project 并构建。

快照保证的是构造成功后内容不再依赖源文件，不宣称对被其他进程同时修改的文件提供文件系统级原子快照。成本是导入时 I/O 和暂存空间；需要实测峰值内存、重复资源去重及 drop 清理。默认不增加另一套借用文件模式。

### 3.7 图片遮挡保留结构和稳定 mask key

图片遮挡笔记保留图片、mask 和其他字段的结构，直到构建才生成 Anki 内容。mask 具有稳定 key，例如 `Mask::rect("nucleus", 20, 20, 40, 40)`。

统一验证图片可解码、坐标和尺寸有效、矩形不越界、mask key 唯一；是否指定笔记 ID 不再改变校验分支。使用同一图像解码坐标约定，不能让 EXIF/方向处理与区域检查各用一套尺寸。

具体入口为 `Note::image_occlusion(Media) -> note::ImageOcclusionBuilder`，通过 `.mask(note::Mask::rect(key, x, y, width, height))` 和 `.mode(note::OcclusionMode)` 声明，`.build()` 返回经过局部校验的 Note。坐标支持有限浮点数及可转换的整数。默认 HideAllGuessOne；HideOneGuessOne 作为另一项显式模式。之后使用普通 Note::field 设置 header/back_extra/comments，使用普通 deck/tag 配置；image/occlusion 是结构化数据生成的保留字段，不能覆盖。直接从内置 IO 模型创建却没有结构化图片/mask 的笔记在添加时失败。

首次实现完整解码 PNG/JPEG/GIF/WebP/BMP；解码像素缓冲区限制 256 MiB，独立于源 Media 的压缩字节预算。坐标以 EXIF 旋转/镜像后的显示图像为准；有方向变换时保留方向已经应用的 PNG 快照，避免客户端各自解释 EXIF。局部构造失败返回 `note::ImageOcclusionError`，具备公开 kind/code、真实解码或 I/O source；不增加仅检查图片头就宣称可解码的捷径。后续支持新格式或调整解码预算时，必须补相同的方向和资源验收。

基线保存 mask key 到 cloze/card ordinal 的映射。调序或移动既有 mask 不重新编号既有卡；新增和删除 mask 作为明确的卡片变化报告。删除编号不直接分配给其他 mask。该能力需要真实 IO 导入更新验证，不能只看生成 HTML 相似。

需要保存删除记录和 ordinal 高水位，并检查编号耗尽：仓库随附的 [Anki cardgen 实现](/Users/hp/Desktop/2026/anki-forge/docs/source/anki/rslib/src/notetype/cardgen.rs:158) 将 cloze 编号映射到卡片 ordinal 0..499。按这一已核查目标，不能分配超过 500 的 cloze 编号；超限明确报错并要求拆分笔记，不能无限递增或静默压缩旧映射。未来扩大支持范围时重新验证目标客户端限制。

普通 Cloze 与自定义 cloze 模型也采用 1–500 的支持范围；超限在构建时明确拒绝，不能输出 Anki 会截断或无法读取的 ordinal。

这里必须替换当前 [IO renderer](/Users/hp/Desktop/2026/anki-forge/anki_forge/src/product/stock.rs:19)：它现在给所有矩形输出 c1，另一个模式使用 c1,2，并没有正确承载新设计的 mask 身份。根据仓库随附的 [Anki IO 编码](/Users/hp/Desktop/2026/anki-forge/docs/source/anki/ts/routes/image-occlusion/shapes/to-cloze.ts:179)，重新实现独立 cloze 编号、非当前遮挡的 oi 标记，以及从像素到归一化坐标的转换；不能只在旧 HTML 上增加一个 key 表。

### 3.8 成功结果与观察报告分开

```rust
Project::build(&self, options: BuildOptions) -> Result<BuildOutput, BuildError>

BuildOutput::artifact(&self) -> &ApkgArtifact
BuildOutput::report(&self) -> &BuildReport
```

BuildOutput 只能由成功构建产生，内部 artifact 必有，字段私有。用户不再处理 `Ok + status + Option<artifact> + ensure_success()` 四层信息。

BuildReport 只描述诊断、计数、比较和更新观察，不拥有文件，也不判断整个操作是否成功。完整结果快照由实际拥有结果上下文的 BuildOutput 或 BuildError 产生：

```rust
BuildReport::snapshot(&self) -> build::json::ReportSnapshot
BuildOutput::snapshot(&self) -> build::json::BuildSnapshot
BuildError::snapshot(&self) -> build::json::BuildSnapshot
```

`build::json::BuildSnapshot` 包含 schema_version、result 和 report。result 的公开类型 `BuildResultSnapshot` 区分：

- Success：artifact 路径、是否临时产物；只能由 BuildOutput 生成。
- Failure：kind、code、人读说明、原因链的文本快照和 PublicationSnapshot 列表；只能由 BuildError 生成。

ReportSnapshot 只含观察，不含 outcome、artifact 或发布状态。PublicationSnapshot 记录路径、实际发布阶段、临时性及持久化是否已确认，不能用一个路径列表掩盖“尚未发布”和“已发布但持久化未确认”的区别。需要保留临时失败产物时由运行时 BuildError 持有其所有权；任何 JSON DTO 都不持有文件。

这些入口都是 inherent 方法，不依赖库扩展 trait；DTO 及传递字段类型全部公开并实现 Serialize。调用者用 `serde_json::to_string(&output.snapshot())` 等标准方式编码。警告不改变成功快照；失败结果不得从诊断严重度、空产物字段或报告内容猜测。当前工具版本和快照 schema 版本分别记录，路径只是可观察信息，不延长临时文件寿命。

**错误的结构化消费约定：** BuildError、CompareError、SchemaError、AddError、MediaError、TemplateBundleError、PersistError 以及配置入口的错误均实现 `std::error::Error + Send + Sync + 'static`，提供 inherent `kind()` 和 `code() -> &str`。kind 返回各自领域的公开错误分类枚举，code 是登记并文档化的机器标识；Display 仅用于人读说明。

产物持久化明确改为 `ApkgArtifact::persist_to(&self, path: impl AsRef<Path>) -> Result<ApkgArtifact, build::PersistError>`，不再返回裸 io::Result。`build::PersistErrorKind` 提供操作分类，实际 std::io::Error 保留为 source；失败后原产物句柄仍可使用，错误保留目标文件是否已发布的事实。领域 code 的承诺只针对本库公开操作的错误，不要求标准库或调用者自行执行的文件 I/O 错误具有 code 方法。

错误创建时确定 primary code：直接操作失败使用对应原因码；聚合验证失败使用规范化诊断顺序中的首个 error。不能选 warning 代替实际失败，也不能从 Display/context 文本解析错误码。`source()` 保留真实底层错误，附加上下文不把错误先转换成字符串。

库的公共返回值不使用 anyhow，也不恢复 ErrorCodeExt。具体错误可直接通过 `?` 进入 Box<dyn Error> 或 anyhow；经过 anyhow context 后可 downcast 回原具体类型并读到相同 kind/code。其他保留 source 的包装器通过原因链查询。不向任意外部包装器承诺最外层自带 code 方法。

规范路径为 `build::{BuildError, BuildErrorKind, BuildReport, BuildOutput, PersistError, PersistErrorKind}`、`build::json::{BuildSnapshot, BuildResultSnapshot, ReportSnapshot, PublicationSnapshot}`；其他操作错误及分类枚举归其 schema、media、update 等领域，通用诊断模型归 diagnostics。遗漏的传递类型由默认消费者的显式命名测试和零 unnameable 门禁发现。

BuildOptions 字段私有，互斥配置使用 enum。显式指定持久输出 `BuildOptions::to(path)` 或临时输出 `BuildOptions::temporary()`；不再把临时文件销毁行为藏在一堆默认 None 里。ApkgArtifact 继续拥有临时文件生命周期，JSON 路径不拥有文件。

### 3.9 更新是完整操作，包内带齐证据

```rust
BuildOptions::to("biology-v1.apkg")
BuildOptions::to("biology-v2.apkg").update_from("biology-v1.apkg")
```

内部明确区分 Create 和 Update；Update 必有 baseline，默认执行完整检查。不再暴露 identity_lockfile、write_identity_lockfile、update_safety 等互相依赖的普通路径开关，也不再有名称容易误解的 self_contained。

新包携带版本化、完整的身份和映射证据，普通更新只要求上一份实际分发的包。缺证据、namespace 不同或无法验证时明确失败，不自动猜测或静默降级。

独立比较有一个不发布产物的入口，复用更新的分析逻辑，必要的临时构建或读取在返回时清理：

```rust
Project::compare(&self, options: update::CompareOptions)
    -> Result<update::ComparisonReport, update::CompareError>
CompareOptions::against(baseline: impl Into<PathBuf>) -> CompareOptions
ComparisonReport::findings(&self) -> &[update::RiskFinding]
ComparisonReport::highest_risk(&self) -> Option<update::RiskLevel>
ComparisonReport::policy(&self) -> &update::PolicyEvaluation
ComparisonReport::snapshot(&self) -> update::json::ComparisonSnapshot
```

比较完整完成即返回 Ok(report)，即使发现高风险、删除或策略阻断。无法读取基线、身份依据缺失/损坏、无效当前模型、检查超限等无法完成请求的情况才返回 CompareError。ComparisonReport 不需要 ensure_success，也不伪造构建 outcome；PolicyEvaluation 明确表示按给定策略是否允许发布。更新构建遇到相同策略阻断则返回 BuildError，不发布新包。失败比较提供诊断和已完成的观察，不能把部分观察伪装成完整比较报告。

完整证据采用 APKG 扩展条目 `ankiforge-identity.json`，`format_version` 为 `ankiforge-identity-v1`。内容包括：

| 证据 | 约定 |
| --- | --- |
| namespace 与模型身份 | namespace；模型 key → Anki model ID、kind、sort field key、活动状态、内容摘要、mtime |
| 字段和模板 | key → config ID、永久历史 slot、当前连续 ordinal 或退休状态；各自高水位 |
| 笔记身份 | key → namespace 隔离的 GUID、模型 key、活动状态、内容摘要、mtime |
| 卡片与 IO | 普通模板 key / cloze 编号 / mask key → 实际 card ordinal；mask 删除记录与高水位 |
| 内容绑定 | 解码后 collection 的 BLAKE3；媒体名 → 实际大小及 SHA-1；规范化 identity JSON 的 BLAKE3 |

identity 摘要对递归按 key 排序的紧凑 JSON 计算，保留数组顺序。摘要用于发现损坏与不一致，不声称提供发布者认证。读取时还要核对实际数据库的 model/config ID、kind、sort field、GUID、mtime、卡片 key 与 ordinal，以及实际媒体；不能仅通过 checksum 就信任映射。拒绝重复 ZIP 条目、未知格式、缺失和不完整的映射。新的完整证据独立于旧 notes.data 部分 metadata；不从后者恢复缺失的新证据。

字段/模板的历史 slot 与 Anki 物理 ordinal 是两个概念。现有声明仅调序时保留基线顺序和未显式更改的 sort field；删减后按历史 slot 为当前成员生成连续 ordinal，同时保留退休 config ID 与 slot，恢复原 key 时复用。新 key 不占用别人的历史身份。IO 的 cloze ordinal 可以留空洞，删除后不重新分给其他 mask。恢复曾整体退休的 note/model 时仍与其历史定义比较，不能只当作无风险新增。

笔记和模型的内容摘要用于决定是否推进 mtime：未改变时沿用；改变时取当前 Unix 秒数与旧值 + 1 的较大值，溢出报错。模型 key 不能在 normal/cloze 之间换种类；既有 note key 不能转给另一模型。这些变化使用新的 key，并按新增/省略风险处理。卡片按身份比较，不能以总数相同推断没有调度风险。

**实际客户端导入条件需要单独验证。** 本仓库 Anki 源码按 model ID 和 config ID 匹配；字段/模板物理 ordinal 必须连续。默认不启用 model merge，schema 增减可能复制模型并产生笔记冲突；启用 merge 后是并集合并，省略项仍保留在学习者集合中。APKG 分发不等于删除同步，恢复身份也不等于恢复学习者已删除的学习记录。字段/模板增减、排序字段变化、既有卡片省略属于默认阻断的 High 风险。

切换 sort field 或改变字段结构时，Anki 可能先将目标 notes.mtime 更新到客户端导入时间，再执行 IfNewer 判断，导致正文未更新。发布时刻不能证明大于未知客户端状态；即使 Always 也要核对同秒时间相等行为。真实导入 oracle 必须覆盖 merge 开/关、IfNewer/Always，以及内容更新、声明重排、删除和恢复，检查字段值、GUID、卡片集合与既有 scheduling。依据为仓库内 `rslib/src/import_export/package/apkg/import/notes.rs`、`notetype/merge.rs`、`notetype/schemachange.rs` 和 `storage/notetype/mod.rs`。

Anki 忽略额外 ZIP 证据条目，但重新导出不会保留它。`update_from` 的基线须为仍携带完整证据的原始分发包；Anki 再导出包缺证据时明确失败。客户端导入兼容性仍以真实 oracle 结果验收。

构建主要发布一个包含证据的 APKG。JSON 报告文件作为显式后续写入操作，避免“包成功但附加报告失败”被混入同一次普通构建。逐文件原子替换和可能的晚期发布事实仍需准确处理；不宣传多文件事务。

### 3.10 高级策略与资源预算

保留原风险阈值和检查预算对应的能力，使用明确的领域配置值，不保留散装更新开关。接口如下：

```rust
BuildOptions::inspect_limits(self, limits: build::InspectLimits) -> Self
BuildOptions::update_policy(self, policy: update::UpdatePolicy) -> Self
CompareOptions::inspect_limits(self, limits: build::InspectLimits) -> Self
CompareOptions::update_policy(self, policy: update::UpdatePolicy) -> Self
UpdatePolicy::fail_on(self, threshold: update::RiskLevel) -> Self
UpdatePolicy::allow(self, code: update::RiskCode) -> Self
```

UpdatePolicy 实现 Default，默认阻断 High 及以上风险，无放行项。RiskLevel 有 Info/Low/Medium/High/Critical 五档。RiskCode 是已登记的、允许由调用者接受的风险类别；从配置文本解析时拒绝未知码和硬错误码，不支持通配符或按 message 匹配。

allow(code) 显式放行该类别的所有 findings，不假装是逐条批准。原始风险级别、最高风险和 evidence 完整保留；PolicyEvaluation 另记录阈值、匹配的放行类别、仍阻断的 findings 和未匹配放行项。未匹配放行项产生 warning，不阻断。需要只接受部分同类变化时，调用者应拆分此次更新；不引入复杂审批对象。

无效 schema、缺失/损坏身份映射、namespace 不符、无法完成必需检查、资源超限等是硬错误，既不属于 RiskCode，也不能通过提高阈值或 allow 绕过。不能提供关闭必需检查的 inspect(false) 开关。

构建策略仅在 Update 模式应用。Create 若显式配置 update_policy，则在开始构建前返回配置错误，不默默忽略。setter 调用顺序不影响最终语义：先设置策略再 update_from，与反过来设置得到相同配置。Compare 使用同一策略计算可发布性，策略阻断仍是成功分析结果。

**检查预算：** InspectLimits 归 build，应用于 baseline 和 candidate 的每次检查，各包分别计数。它是独立数量上限的数据值，可通过 Default 后修改公开数值字段配置；BuildOptions/CompareOptions 自身字段仍私有。默认值以当前有限检查预算为起点，并明确写入 API 文档：

| InspectLimits 字段 | 默认上限 |
| --- | --- |
| max_archive_bytes | 2 GiB |
| max_entries | 100,000（ZIP 条目、媒体索引分别计数） |
| max_central_directory_bytes | 16 MiB |
| max_zip_entry_bytes / max_zip_total_bytes | 1 GiB / 4 GiB |
| max_meta_bytes / max_media_map_bytes | 64 KiB / 16 MiB |
| max_identity_bytes | 64 MiB，JSON 解析前限制完整身份清单 |
| max_collection_bytes / max_media_bytes | 512 MiB / 256 MiB |
| max_decoded_total_bytes / max_zstd_window_bytes | 4 GiB / 64 MiB |

超限错误结构化保留 resource、limit、observed 和相关 entry；调用者可显式提高对应预算后重试，库不自动放宽。0 表示该计数不允许大于 0，不表示无限制。

**媒体读取预算：** `media::MediaLimits` 是独立配置值，公开 `max_bytes: u64`，Default 为单资产 256 MiB。file/bytes 使用默认值，with_limits 变体在创建快照之前应用指定值。已知长度先预检，流式读取仍按实际读取量检查，超限清理本次未完成的快照。bytes 已占用的调用者内存不属于库可回溯控制的资源。

内存/暂存磁盘的切换阈值属于实现策略。MediaLimits 是单次资产读取预算，不是多个项目共享的进程级总预算；项目产物的总量受后续包检查预算限制。提高 MediaLimits 不会自动提高 InspectLimits；BuildOptions 更不能回溯限制早已完成的 Media::file 调用。模板 bundle 加载对每个资产使用同一默认媒体预算，并提供 `NoteType::from_bundle_with_limits(path, MediaLimits)` 入口调整它。

## 4. 推荐调用示例

### 4.1 普通创作与更新

以下展示目标体验，不是当前版本的可运行代码。实施时必须转为独立 consumer 和打包后的执行样例。

```rust
use ankiforge::{BuildOptions, Content, Field, Media, Note, NoteType, Project, Template};

fn lesson() -> Result<Project, Box<dyn std::error::Error>> {
    let mut project = Project::new("biology")?
        .name("生物")
        .default_deck("生物::细胞");

    project.add("cell-definition", Note::basic("什么是细胞？", "生命的基本单位"))?;

    let image = Media::file("assets/cell.png")?;
    let audio = Media::file("assets/cell.wav")?;
    project.add("cell-media", Note::basic(
        Content::sequence([
            Content::text("识别这个细胞："), image.image(), audio.sound(),
        ]),
        "植物细胞",
    ))?;

    let vocab = NoteType::builder("vocab")
        .name("词汇")
        .field(Field::new("front").name("正面"))
        .field(Field::new("back").name("背面"))
        .template(Template::new("recognition")
            .name("识别")
            .front("{{front}}")
            .back("{{FrontSide}}<hr>{{back}}"))
        .build()?;

    project.add("term:cell", vocab.note()
        .field("front", "cell")
        .field("back", Content::html("<b>细胞</b>")))?;
    Ok(project)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = lesson()?.build(BuildOptions::to("biology-v1.apkg"))?;
    println!("{}", output.artifact().path().display());
    Ok(())
}
```

第二次构建重新声明同一项目和笔记 key，更新正文，并使用 `.update_from("biology-v1.apkg")` 输出到新的 v2 路径。示例执行器需准备真实图片和音频、使用临时目录，并检查导出内容与身份连续性。

### 4.2 固定名称、字体/CSS 与手写 HTML

下面也是目标调用示例；字体和 PNG 必须是可解析的真实 fixture，不用任意字节冒充资源：

```rust
let font = Media::file("assets/labels.woff2")?
    .with_export_name("labels.woff2")?;
assert_eq!(font.filename(), "labels.woff2");

let model = NoteType::builder("labeled")
    .field(Field::new("front"))
    .field(Field::new("back"))
    .template(Template::new("recognition")
        .front("{{front}}")
        .back("{{FrontSide}}<hr>{{back}}"))
    .css("@font-face {font-family: labels; src: url('labels.woff2');}\n\
          .card {font-family: labels;}")
    .asset(font)
    .build()?;

let png = std::fs::read("assets/badge.png")?;
let badge = Media::bytes(png, "image/png")?
    .with_export_name("badge.png")?;
project.add_asset(badge)?;
project.add("named-assets", model.note()
    .field("front", Content::html("<img src=\"badge.png\">"))
    .field("back", "命名字体与图片"))?;

let output = project.build(BuildOptions::to("named-assets.apkg"))?;
let snapshot: ankiforge::build::json::BuildSnapshot = output.snapshot();
let json = serde_json::to_string_pretty(&snapshot)?;
```

另用相同模型与资产写一份新格式 bundle fixture，执行 `NoteType::from_bundle(...)` 后加入笔记并打包。两条路径都必须在 APKG 媒体索引中找到 labels.woff2/badge.png，且读取到正确内容；只比较 ZIP 字节或文件存在性不够。字体 fixture 附来源和可分发许可；需要随 crate 分发的示例及 fixture 同时纳入 Cargo.toml 的 include 范围和 PACKAGE_FILES.txt。

### 4.3 配置预算并读取完整比较结果

```rust
use ankiforge::build::InspectLimits;
use ankiforge::update::{CompareOptions, RiskLevel, UpdatePolicy};

let mut limits = InspectLimits::default();
limits.max_collection_bytes = 1 << 30;
let policy = UpdatePolicy::default().fail_on(RiskLevel::Medium);
let comparison = project.compare(CompareOptions::against("biology-v1.apkg")
    .inspect_limits(limits.clone())
    .update_policy(policy.clone()))?;
println!("{:?}", comparison.policy());

// 使用同一策略构建；若有阻断风险则返回带比较证据的 BuildError。
let output = project.build(BuildOptions::to("biology-v2.apkg")
    .update_from("biology-v1.apkg")
    .inspect_limits(limits)
    .update_policy(policy))?;
```

文档另外展示在业务代码中选定 RiskCode 后 `.allow(code)` 的明确含义；不把自动接受 comparison 返回的全部 findings 作为教程默认步骤。

## 5. 实施顺序

1. **先写目标 consumer。** 把本文已确定的接口契约落实成编译探针，覆盖最简笔记、自定义中文模型、组合及命名资产、IO、临时产物、比较和严格更新。验证泛型推断、所有权、完整类型路径和反向边界；不得把已明确的转换、错误标识或策略行为留到实施时临场决定。
2. **建立唯一模型。** Project/Note/不可变 NoteType，删除公开 Deck/Lane/Draft 和字符串模型查找协议；迁移 SDK 内部调用，避免继续维护第二套逻辑。
3. **统一内容与资产。** typed 内容树、媒体快照、读取预算、自动依赖收集、固定命名和显式资产入口；删除 PendingMedia、裸 Media 内容和原生 bytes 的内联限制。
4. **实现 key 与身份规则。** 完整转换矩阵、schema 校验、模板引用编译、名称与身份分离、结构化 IO 和 mask 映射；同时落实剩余值类型的 must_use。不保留历史空 key 或显示名称参与身份的逻辑。
5. **重构构建及更新结果。** BuildOutput/BuildError、kind/code/source、私有操作配置、风险策略和检查预算、独立比较、Create/Update 模式、完整包内证据、三种报告快照职责。保留已有可靠的原子写入和拥有型文件管理实现。
6. **收敛公开面和全部调用者。** 移除旧 prelude、旧 aliases、无用途的内部出口；更新 Rust 文档、网站、CLI/tools、Node/Python native、契约和 fixtures。分发内容核对 Cargo.toml 的 include 范围并同步 PACKAGE_FILES.txt；仅仓库使用的文件不加入分发清单。
7. **完成消费者与分发验收。** 默认 feature 和 packaged consumer、零 unnameable、真实 missing_docs、网站 check:docs/check:examples、跨平台/MSRV、SDK 重建与端到端导入更新验证。

直接按目标模型实施，删除不再需要的适配层。底层正确的解析、规范化、写入和检查实现可以复用，但最终只有一套创作模型及语义。

## 6. 验收目标

- 旧入口继续编译不再是成功标准；新根路径和领域模块完整可用才是。
- 中文字段通过显式 key 成功；空 key/空白 key 现在应失败，不再保持历史 parity。
- Basic/Cloze 字符串默认 Text；显式 HTML 可用且不会二次转义；图片/音频节点不会提前丢失依赖。
- 大于 64 KiB 的正常原生媒体成功；成功取得 Media 后修改/删除源文件，输出仍使用快照；失败清理、clone/drop、资源预算另测。
- 同一个 Media/NoteType 可跨函数和项目复用，无需 lifetime 参数；同 key 不同模型和显式同名不同媒体内容有确定错误。
- 模板使用 key，最终 Anki 字段使用显示名称；过滤器、条件段、注释、browser 模板及诊断源位置正确。
- 修改正文、模型/字段/模板显示名称不改变既有身份；重排模板和 IO mask 的 ordinal 按基线正确保留；删除/恢复、编号耗尽有明确规则。多 mask 的实际卡片数、两个遮挡模式、像素归一化均按目标 Anki 实现验证。
- 成功必有 artifact，失败保留事实；报告快照不持有临时文件；默认不再写独立 lockfile。
- 只有新的完整包内证据能支撑承诺的更新路径；缺失或损坏证据明确失败，不做历史包恢复兼容层。
- Node/Python 可以调整接口和预期，但支持的同一创作语义必须与 Rust 一致；不能保留一套旧行为靠 bindings 绕过新模型。

### 6.1 审核补项与强制验收

下列每行都是交付条件。它们补齐此前审核的五项设计要求与两项验收要求，不能以“示例能编译”替代：

| 编号 | 对应要求 | 必须通过的消费者验证 |
| --- | --- | --- |
| A1 | JSON 的职责和可调用性（3.8） | 无扩展 trait 调用 Report/Output/Error 的 snapshot；显式命名并序列化所有 DTO；带 warning 的成功仍为 Success；晚期失败有准确发布事实；纯 Report 没有 outcome；保存快照不延长临时文件寿命 |
| A2 | 标准错误、比较和机器标识（3.8–3.9） | 各类公开错误通过 `?` 进入 Box<dyn Error> 和 anyhow；context 后 downcast 的具体类型、kind/code 不变；自定义保留 source 的包装器可沿链取回；修改 Display 文案不改变机器判断。高风险比较返回 Ok，坏基线/缺证据/超限返回 Err |
| A3 | 高级策略和分阶段预算（3.10） | 默认阻断 High；匹配放行后保留原风险/evidence，未匹配放行产生 warning；未知码拒绝；硬错误不能放行。相同输入和配置下 compare 的策略结论与 build 一致。Create 显式配置更新策略报错，setter 顺序不影响结果。baseline/candidate 分别触发检查预算；显式提高对应预算可重试；Media 读取阶段超限会清理 |
| A4 | owned/borrowed key 可组合（3.3） | &str、String、&String、Cow、owned key、borrowed key 都通过字段声明/赋值、模板、generation 和 cloze 字段入口；借用 key 可重复使用；错误种类 key 的输入编译失败。空/冲突 key 的错误发生在规定的验证阶段 |
| A5 | 丢弃未完成值可发现（3.2） | 独立 probe 设置 `#![deny(unused_must_use)]`：忽略 NoteTypeBuilder、IO builder、Note 和未使用的消费型配置失败；完成 build/add 或将值保存、返回成功；显式丢弃不会隐式提交，临时资源正常清理；仅需要持久输出的 build 用法不被强迫读取报告 |
| A6 | 默认公开面和内部隔离（3.1） | 默认领域模块及所有返回类型可命名；内部模块、Project::lower/normalize、内部 IR 取得路径均 compile-fail。旧 Deck 及其三个 lowering 出口不存在。tools 仅在显式 feature 下可用，真实仓库工具调用通过；不能用 tools 下通过来替代默认面验收 |
| A7 | 非 AST 资产闭包（3.6、4.2） | 执行 bytes+MIME、命名字体/CSS、raw HTML、bundle 两条完整打包路径；检查媒体映射、内容及引用。验证错误 MIME/非法名称、同名异内容、大小写或 Unicode 规范化冲突；明确资产不会被误删。模型或项目加入失败前后状态相同 |

A2 的公开错误集合包括构造/模型、添加笔记、媒体、模板 bundle、配置、比较、构建和产物持久化；不是仅测 BuildError。比较或策略阻断产生的证据必须可读，不能通过 `.to_string()` 断言替代 code/source 验证。

A5 的编译失败需要匹配 unused_must_use 诊断；A6 的失败需要匹配预期不可见/不存在的符号。每个 probe 先验证同一环境下的合法控制样例能编译，不能把依赖解析失败或未关联错误当作通过。

### 6.2 检查执行与现有测试迁移

新增默认消费者合约测试 `anki_forge/tests/public_surface_contract_tests.rs`，把 A1–A7 的编译和运行场景放在独立临时 crate 中：显式 `default-features = false`、不处于主 workspace、不通过 Node/Python native 引入 internal-tools。该测试仅在仓库运行，加入 quality 脚本的定向测试列表，不加入 Cargo.toml 的分发 include 或 PACKAGE_FILES.txt。

解包后的关键成功路径由 check_rust_crate_package.sh 创建独立 consumer 执行。需要在分发包中使用的源码、示例和 fixture 同时核对 Cargo.toml include 与 PACKAGE_FILES.txt；以 cargo package --list 的实际结果为准，不能只向精确文件清单添加尚未被 include 收入的路径。

现有 stable_facade_boundary_tests 的“所有模块私有”断言改为第 3.1 节的明确允许列表；public_api_boundary_tests 中默认消费者断言不能继续依赖整个文件的 internal-tools 配置。Deck 删除后的行为测试迁移到唯一 Project 路径；仍有价值的 IR 专项断言留给工具测试，不通过给整个消费者文件加 feature gate 来避开失败。

文档执行器补齐媒体、固定名称资产、IO、比较和前后两次构建：创建临时工作目录、准备真实 fixture、执行完整 fn main 示例、读取 APKG 验证语义。不得只执行 basic/custom 两个旧样例或跳过缺少资源的代码。默认配置 rustdoc 必须可解析完整公开面；missing_docs 作用于真实定义和 impl，不用注释 pub use 或全局 allow 代替。

最终门禁在依赖安装完成、提交干净的分支上执行；离线步骤要求依赖已缓存：

```sh
cargo rustc --offline --locked -p ankiforge --lib --no-default-features -- -D unnameable_types
RUSTDOCFLAGS="-D warnings" cargo doc --locked -p ankiforge --no-default-features --no-deps
cargo test --locked -p ankiforge --no-default-features --test public_surface_contract_tests
cargo test --locked -p ankiforge --doc
bash scripts/check_rust_crate_quality.sh
bash scripts/check_rust_crate_package.sh
bash scripts/verify-ci.sh --ci
npm --prefix website run check:docs
npm --prefix website run check:examples
```

保留现有四平台 × Rust 1.92/stable 的 packaged-consumer 矩阵、tools/all-features 检查和 Node/Python native 重建回归。各工作包先跑相关定向检查，最终再跑完整门禁。只有本节原有功能验收和 A1–A7 都通过，才能宣称此前问题已经解决；本方案的补齐本身不代表实现或测试已经完成。

契约 bundle 随实际语义变化更新。key 接受范围、默认内容语义、身份推导、媒体存储、更新证据和报告形状都可能变更，需要修改实际受影响的 schema/registry/semantics/fixtures，并按[契约政策](/Users/hp/Desktop/2026/anki-forge/contracts/versioning/policy.md)记录有意的破坏性变更。版本和变更记录用于准确表达新设计，不要求保留兼容层或开发历史迁移工具。

## 7. 仍然不值得引入的设计

不引入 `Project<HasBaseline, HasOutput, ...>`、带项目 lifetime 的字段/媒体句柄、每个 setter 一个 Result、为每一种字符串都建立公开新类型、必须借助宏才能定义普通模型，以及把 IO 一开始压成无法检查的 HTML。

这版的目标是让正确用法自然成立：完整的模型、拥有的资源、明确的身份、必有产物的成功结果。强类型和 builder 只在确实减少调用者负担的位置使用。

## 8. 实施记录

起点：`115ed2761dd6bad66fb6cf4fb72919a49a0313d4`，当前 `main` 分支。按 implement skill 实施，最终进行 standards/spec 双轴审查并提交当前分支；不发布。此记录仅跟踪实施，不替代第 3–6 节任何要求。已确认测试边界为第 6 节的独立默认消费者、公开领域操作、APKG 语义与真实导入更新。

### 已接入的能力

- 根路径 Project/Note/NoteType/Field/Template/Content/Media 及领域模块已实现。模型共享不可变声明，Note 持有模型，Content 保留结构和资源。完整 key 转换矩阵、原文模板区间重写、schema 校验及主要值的 must_use 已有消费者检查；自定义模型已接入 APKG 写入。
- Project::add/add_asset 原子收集依赖；命名冲突失败不留下部分注册。Media file/bytes 使用有预算的拥有型快照、流式计数和大文件暂存；跨项目复用不依赖源路径。可移植名字使用 NFC + 默认大小写折叠检查冲突；typed image 正确编码 Unicode、空格及 URL 保留字符。
- 新 bundle-v2 返回同一种 NoteType，验证清单、文本、引用路径及资产；只接受新格式。真实 PNG/WAV/WOFF/TTF fixture 及源码生成说明保留在仓库测试目录，两条原生/bundle 资产闭包都核对 APKG 媒体和字段内容。原始字体为本项目生成、MIT，可用 fontTools 和 FreeType 解析。
- 结构化 IO 完整解码图片、统一 EXIF 显示坐标、归一化像素矩形；两种模式输出独立 cloze 卡片。更新保留 mask key/ordinal、退休记录和高水位，恢复原 key 复用编号，500 编号耗尽明确失败。普通 Cloze 也拒绝超过支持范围的编号。
- 构建返回必有产物的 BuildOutput；报告仅含观察，Output/Error 生成真实结果快照。错误保留 kind/code/原始 source；持久化失败保留发布阶段，快照不拥有临时产物。新的候选管线复用私有 normalize/writer，直接传入统一身份计划，不再走旧 update-safety/lockfile 推导。
- Create/Update 私有模式、update_from、独立 compare、风险类别解析、默认 High 阻断、显式类别放行和未匹配 warning 已接入。compare 返回完整风险分析，build 在发布前执行同一策略；硬错误不能放行，Create 显式配置策略报错，setter 顺序不影响语义。BuildReport 和 JSON snapshot 均可读取完整比较。
- APKG 携带完整身份清单并绑定实际 collection/媒体；namespace 隔离 GUID，保留模型/config IDs、字段/模板历史 slot、普通卡片与 IO 映射。内容/名称变更推进 mtime，重复相同版本保留修订。严格读取核对 JSON、摘要、DB 身份与映射，并分别执行 baseline/candidate 的有限检查预算。

### 本轮审查与验证

- 独立默认消费者合约 21/21 通过，包括 A1–A7、所有消费型配置的 `unused_must_use`、完整 key 转换和真实 APKG 检查。公开边界正向 3/3、反向编译 3/3 通过；默认 `unnameable_types` 为零。旧 prelude/Deck/lowering 出口及公开内部实现树已删除。
- 新公开行为测试：schema 7/7、项目值 9/9、媒体输出 4/4、产物生命周期 5/5、部分观察报告 3/3、使用场景 13/13、嵌入资源 smoke 1/1 通过。基线验证后候选失败保留 baseline_counts，比较尚未完成不会伪造结果。
- 媒体子进程生命周期测试 4/4 通过：拥有者 clone/drop、源改删后的实际输出、重复内容保留单份大文件快照、Unix 中途写入失败清理。实测文件 64/256 MiB 的进程峰值 RSS 为 6.45/6.47 MiB；重复导入返回后保留单份快照，但期间仍有第二份暂存，峰值逻辑磁盘约 128/512 MiB。raw evidence 记录单次样本/debug/macOS arm64 等局限，不能外推所有平台。
- Node 已迁移到默认 Rust API；17 项 native/APKG 合约、独立安装 ESM/CJS/types、只读安装目录、worker/lifetime 与文档示例通过。Python 已迁移，47 项 native/APKG 测试、类型正反探针、6 个示例和独立 wheel 安装通过。SDK 不再持有另一套旧 Deck/registry/identity 语义。
- CLI/tools 已迁移为精选操作及 DTO；原生 recipe 使用 ankiforge-project-v1。CLI 8 项通过，含报告原子替换、hardlink 别名拒绝、写报告失败保留真实构建快照。仓库工具调用与覆盖矩阵见 contract_tools/README.md。
- 真实 Anki oracle：7 条版本链 × 4 种导入配置，28 个场景、96 次实际导入；所有既有 note/card IDs、GUID、ordinal 与排程检查通过。85 次正文符合目标，11 次由模型合并/排序字段变更产生的客户端 mtime 或无 merge 冲突限制已明确断言。IO 两种模式均核对精确 mask 内容及 2→2→3→3 实际卡数。证据见 scripts/roundtrip_oracle/RESULTS.md 和 docs/plans/evidence/rust-api-clean-slate-2026-09-24/anki-import-oracle.json。Always 同秒跳过只核对上游源码，本次自然时序未触发，不宣称已实测。
- 契约 bundle 1.0.0 的实际 schema/semantics/fixture/error-registry 变更及精确摘要清单通过 release 治理验证。major bump 表达有意破坏性变更，独立于 Rust crate 0.2.0；无旧接口适配层。嵌入 archive 可重现重建、实际 package payload 精确清单和仓库外 packaged consumer 均通过验收。
- Rust/common/Node/Python 指南与示例已迁移。旧 api-design.md 大方案已替换为指向本方案的入口，历史 ADR 标记被 ADR 0023 取代，避免把历史实验当作当前接口约束。归档性能图明确不代表新 Project API 的测量结果。

### 旧测试的迁移去向

删除旧入口测试必须保持有用行为的覆盖，不用 feature gate 隐藏消费者失败：

| 旧测试族 | 当前覆盖或明确退休的契约 |
| --- | --- |
| Deck model/validation/lowering/export/facade/identity | project_value、rust_user_api_smoke、public_surface_contract 消费者；Deck 单独容器和导出路径删除 |
| Project/model/product pipeline/v3/native input | 新公开项目/schema/media 测试及 tools product_cli；旧 ProductDocument 原生输入不再接受 |
| custom merge/template entry parity/template bundle | schema 消费者、bundle 消费者、真实更新 APKG 与 Anki oracle；不保留两套创作入口的 parity |
| update_safety/phase4 baseline/reconcile/report/risk | 独立 update 消费者，核对完整包内证据、历史恢复、风险与策略、双预算、身份/mtime；lockfile/report-only/recipe 协议明确退休 |
| artifact/build report/media export | 新默认行为测试，检查真实发布、源快照、错误 source 与部分观察；确定性晚期 fsync 持久化失败 2 项已通过 |
| authoring_core/writer_core/APKG limits/product portability | 有效的解析、规范化、writer、diff、限额断言迁至 tests/internal；仅仓库测试访问私有算法 |
| Rust capability matrix | 23 个场景迁至唯一默认 Project API，并保留人工导入产物/清单生成器；本轮 23/23 通过 |
| Node/Python 旧 raw/structured/Deck/lockfile suites | 新 native public-api suites、独立 Rust 产物观察器、安装与类型测试；删除已退出产品面的协议测试 |

### 最终双轴审查

按 code-review skill 由两个 Astra subagent 独立检查规范和方案，首轮提出以下问题：

| 轴 | 问题 | 修复与验证 |
| --- | --- | --- |
| Standards / P1 | Python fork child 析构可删除 parent 媒体快照或等待继承锁 | ProcessOwned/ObjectState PID 防护，核心 Snapshot/弱缓存按 PID 隔离；47 项 Python 测试内含 5 类真实 fork 子进程，另有核心进程分支 3 项；独立复核 resolved |
| Standards / P2 | Python bundle 错误把预算 source 压成文字 | 遍历类型化 source_details，manifest/text/media 三类真实超限回归；独立复核 resolved |
| Spec / P2 | 归一化 CAS I/O 丢原始 source 且误分类 Validation | 原始 io::Error 贯通 CAS/归一化/BuildError，真实故障与原对象 downcast 回归通过；保留诊断、基线 counts；Spec 独立复核 resolved |
| Spec / P2 | Python bundle 缺少可调 MediaLimits | 接入 Rust from_bundle_with_limits，默认/放宽预算回归通过，Spec 独立复核 resolved |
| Spec / P2 | SDK 非有限 IO 坐标错误类型/阶段不一致 | Node N-API / Python PyO3 直接传浮点参数；NaN/±Inf 在 Rust build 返回 NOTE.IO_RECT_INVALID，回归通过；Node 安装协议同步升级并通过安装验收；Spec 独立复核 resolved |

额外活动调用者：benchmark Rust adapter 已迁至 Project；verifier 显式识别新身份元数据，保持未知 payload 拒绝和预算。46 项基准单测、Rust/genanki smoke、100 notes/100 cards/49 media 实包验证通过。未重做历史性能测量。

### 本地交付结果与发布边界

实现已提交为 `399bc19`；消费者执行竞争修复为 `289ccfc`，后续提交补齐同类验证工具隔离和最终记录。实现后的生产源码未再改变。完整命令、结果和适用范围见[最终验收记录](evidence/rust-api-clean-slate-2026-09-24/verification.md)。

- 根级 dead_code/unused_imports 放行已移除；默认 unnameable_types 为零，默认及 all-features rustdoc 均通过。quality 全部通过：默认库 121 项、all-feature 库 353 项、工作区集成测试、5 个 doctest、Clippy、契约嵌入重建、精确 payload、版本及依赖例外检查。
- A1–A7 独立默认消费者 21/21 通过，完整能力矩阵 23/23 通过。归一化原始 I/O 原因链、并行媒体失败、晚期 fsync 发布事实均有回归。Standards 2 项与 Spec 3 项发现已修复并独立复核，无未关闭发现。
- verify-ci 全部通过；Node 17/17、Python 47/47、类型及独立安装检查通过。最终 Python sdist 在仓库外离线重建 wheel，隔离安装后 47 项再次通过，包含五类真实 fork 持有者。
- Rust 1.92.0 和本机已安装的 1.98.1 均完成离线 package 和独立 packaged consumer；正常使用不依赖仓库工作目录或外部契约文件。
- 网站 check:docs、build、check、test、check:site 通过；文档执行器逐个执行完整程序并检查 ZIP/SQLite/媒体，每轮 16 次执行、22 个 APKG。最后发现的消费者可执行文件竞争已修复：默认消费者两进程各 21/21，网站两进程各 16/22；媒体探针也按进程隔离，4 项生命周期及测量脚本冒烟通过。
- cargo-deny 0.20.2 使用 2026-09-24 新刷新的 RustSec 数据库，advisories/licenses/bans/sources 全通过。64/256 MiB 测量和 96 次 Anki 导入的原始证据保留，并明确其平台、样本、版本及客户端限制。
- 四平台 × Rust 1.92/stable 的受保护 CI 矩阵保留。其他平台和当前 stable 通道仍须 hosted CI 验证；1.98.1 仅代表明确安装的工具链。正式发布的外部配置、来源审查及维护者批准仍按发布流程处理。本轮完成本地实现和验收，未推送、未创建 release tag、未发布。

### PR #51 审查与 CI 修复（2026-09-25）

以上本地交付记录对应首次提交；实现现已提交到目标为 `main` 的 [PR #51](https://github.com/morehardy/anki-forge/pull/51)。首轮 hosted CI 的四平台 × Rust 1.92/stable 打包消费者全部通过，审查和其他任务还发现以下需要修正的事项：

- 发布后验证示例改用 `BuildOptions::temporary()`；PR 测试直接编译并执行工作流中的实际 Rust 程序，防止只在发布后发现 API 过期。
- 模板 `target_deck` 在完成 NoteType 和加载 bundle 时复用项目牌组名称规则，拒绝空名称、空层级、首尾空白/冒号和控制字符，返回 `SCHEMA.NAME_INVALID`。两条入口都有先失败后通过的回归测试；后续 review 的 `:` / `Parent:::Child` 反例同时覆盖默认牌组和笔记覆盖入口，内部冒号的有效名称核对实际 APKG 输出。
- 网站离线消费者验证前显式 `cargo fetch --locked`。Node 消费者矩阵下载同平台构建任务产出的独立观察程序，避免依赖消费者机器的 Cargo 缓存；完整 17 项语义检查保留。
- Python wheel 隔离测试通过 `-I -X utf8` 启动，保留 Windows 中文目录覆盖，并消除重定向输出使用本地代码页导致的异常。
- crates.io 已存在 0.1.0，故 Rust clean-slate 版本改为 0.2.0；同步路径依赖、锁文件、Python 加载器版本断言和当前版本文档。保留 SemVer 检查，不为有意破坏性改动绕过门禁。
- 后续 review 补齐三项输入/证据边界：拒绝媒体 map 中重复的导出文件名，防止收集为映射时覆盖条目后误判完整证据；项目标签 schema 与 Rust 的精确 Unicode 空白/控制字符、保留前缀和去重规则保持一致，并重建尚未发布的 bundle 1.0.0；不同模型 key 的显示名按 writer 的 trim 与 SQLite BINARY 规则检查冲突，在 add 时原子拒绝，保留大小写、Unicode 组合形式及内部空格的有效区别。
- schema 的三个牌组入口同步运行时规则，IO masks 明确为 1–500 个，坐标非负且尺寸为正；显示名、稳定键、资产别名及导出文件名补齐同类标量约束和 Unicode 边界。新增 8 项 schema/API/loader 对照回归，连同标签和 schema gates 共 41 项通过；UTF-8 字节预算、图像尺寸和跨引用关系仍由运行时验证。
- CLI 在读取项目或构建前拒绝 APKG 输出覆盖项目 JSON；更新构建在检查基线和生成候选包前拒绝输出覆盖原始基线，返回 `BUILD.OUTPUT_INVALID`。直接、相对、符号链接和硬链接别名均有文件保留回归，正常独立输出仍可成功。
- Node `Media.bytes` 通过不复制数据的 Buffer 视图传入原生层，先解析预算并检查长度，再同步取得合法输入的自有副本。独立子进程验证显式 bigint 预算和默认 256 MiB 预算在拒绝时不会产生整块复制；子数组偏移及调用后立即修改输入的快照语义保持不变。Node 完整语义检查由 17 项增加至 19 项。
- bundle/project schema 对照审计补齐便携相对路径、导出文件名、目标牌组、具体 MIME、IO 可写字段、Cloze 模型结构和内联媒体预算；保留项目文件路径的真实文件系统语义。bundle YAML 先保留类型及重复键校验，再按严格结构解析，拒绝把 null、数字或布尔值隐式转成字符串。MIME 同时拒绝空子类型。
- 发布路径检查统一为只读的逐组件解析：先解析已有符号链接，再处理父目录和尚不存在的目录，最后比较文件身份。临时 artifact、更新基线和 CLI 项目/报告路径均拒绝 `missing/../source` 别名，不产生目录副作用；合法新目录、符号链接父目录语义和持久产物脱离临时 owner 后的生命周期均有回归。

这些调整补齐实现与验证边界，不引入兼容层。当前提交的最终 hosted CI 状态以 PR checks 为准；本次仍不创建 release tag 或发布包。
