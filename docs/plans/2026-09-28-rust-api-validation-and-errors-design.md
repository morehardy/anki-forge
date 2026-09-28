# Rust API：媒体用途、添加错误与项目标题优化方案

日期：2026-09-28。核查基线：`a0b44a5`。

状态：设计完成，已通过三位 GPT-6 Astra 子代理交叉审查，待实现。用户已确认删除项目标题，并采用“用途匹配、不保证可解码”的媒体校验。本文件只规定这三项变更；实现和验收尚未执行。

本文是 [clean-slate 设计](2026-09-23-rust-api-clean-slate-design.md) 的定向修订方案。实施时同步更新其对应章节和 [设计入口](../api-design.md)，避免保留两份冲突的目标。现有 API 文档在实现落地时一并迁移。

## 1. 已确定的目标与范围

| 问题 | 决定 | 完成标准 |
| --- | --- | --- |
| WAV 可以被当作图片、PNG 可以被当作声音且构建无诊断 | `Project::add` 校验 typed 内容节点的媒体用途 | 不兼容用途返回有精确位置的 `AddError`，项目不发生部分登记 |
| `AddError` 的字段、标签、资源等信息藏在文案里 | 增加原生结构化上下文及必要的冲突证据 | Rust 和 SDK 调用者不解析 `Display` 就能定位问题 |
| `Project::name` 被保存和校验，却不进入产物或报告 | 删除项目标题及各语言、工具输入对应入口 | 只有 namespace 和实际卡组名称承担原有各自职责 |

保留 `Project → Note/Content → build` 主流程、拥有快照的 Media、不可变 NoteType、原子添加和现有身份更新规则。

本次不增加发布元数据、不优化 compare/build 重复构建、不增加批量校验或返回被拒绝 Note 的接口，不重构全部公开错误，不扩展普通媒体的解码格式或播放承诺。

## 2. 删除项目标题

### 2.1 最终使用方式

```rust
use ankiforge::{BuildOptions, Note, Project};

let mut project = Project::new("biology-course")?
    .default_deck("生物::细胞");
project.add("cell", Note::basic("什么是细胞？", "生命的基本单位"))?;
let output = project.build(BuildOptions::to("biology.apkg"))?;
```

- `namespace` 是稳定身份域，继续参与 GUID、模型和配置身份；不是供随意改名的展示标题。
- `default_deck`、`Note::deck` 和模板目标卡组决定实际 Anki 卡组及层级；一个 Project 可以包含多个卡组。
- 应用自己的任务标题、目录条目或 UI 标签由应用保存。库不新增 `label`、`title` 或 `PublicationMetadata` 替代入口。
- `NoteTypeBuilder::name`，以及 Field/Template 的 `.name()` 保留；它们有实际 Anki 展示语义。
- 不把历史项目标题自动搬到 default_deck、输出文件名、标签、卡组描述或 namespace。

### 2.2 必须迁移的入口

| 入口 | 变更 |
| --- | --- |
| Rust | 删除 Project 的 `name` 字段、`.name()`、`.display_name()`，以及构建时的标题校验 |
| Node | 删除公共和 native Project 的 `.name()`，同步 internal native interface、声明文件和示例 |
| Python | 删除 Project 的 `name=`、`.name` property、`_name` 缓存，更新 native 构造器和 `.pyi` |
| 工具 JSON | 使用 `ankiforge-project-v2`，删除顶层 `name`；更新 Input、流式 decoder、schema、fixtures 和 loader |
| 文档与验证 | 移除项目标题用法；保留模型、字段、模板改名及真实卡组改名的验证 |

工具输入只接受 v2，不保留接受后忽略的兼容分支。旧 v1 即使没有 `name` 也应拒绝；v2 中 `name: null` 和任何其他顶层 `name` 值都作为未知字段拒绝。模型/字段/模板内的 `name` 继续有效。

项目标题从未写入 collection、身份清单或报告，删除不要求迁移既有 APKG 身份，也不改变笔记/model 的内容摘要、mtime 或更新基线格式。

## 3. 媒体用途契约

### 3.1 构造接口保持不变

```rust
Media::image(&self) -> Content
Media::sound(&self) -> Content
Project::add(&mut self, key: impl Into<String>, note: Note)
    -> Result<(), note::AddError>
```

`image()` 和 `sound()` 构造有语义的内容值，不登记资源。`Project::add` 检查该内容是否适合进入项目，并能附上 note/model/field 的上下文。普通组合内容不因此增加一串 `Result` 或 `?`。

### 3.2 接受矩阵

检查 Media 在导入时保留的 MIME 的 type/subtype（essence），忽略参数及 ASCII 大小写，不按当前导出文件名猜测用途。

| Media 的 MIME 类别 | Image 节点 | Sound 节点 |
| --- | --- | --- |
| `image/*`，含 SVG | 接受 | 拒绝 |
| `audio/*` | 拒绝 | 接受 |
| `video/*` | 拒绝 | 接受 |
| `application/octet-stream`、text、font、PDF 等其他类别 | 拒绝 | 拒绝 |

表中的 `*` 表示类别，不表示 `Media::bytes` 接受 MIME 通配符。它继续要求具体、合法的 MIME。

Sound 表示 Anki 的 `[sound:...]` 内容；它同时支持音频和视频，不能限定为 audio。仓库随附的 Anki `pylib/anki/sound.py` 明确采用这一约定。

“接受”仅表示用途类别匹配，不表示文件完整、格式可解码，或所有客户端一定能显示／播放。普通图片不套用图片遮挡的格式白名单。图片遮挡的主图继续在 IO builder 中做完整解码、方向和尺寸校验；其普通可编辑字段使用相同 Content 用途规则。

### 3.3 未知内容、显式 MIME 与命名

- 保留现有 Media 导入规则：高置信内容识别优先，兼容容器提示按已有规则处理，文件扩展名只能按既有规则补充识别。
- `Media::bytes` 的明确合法 MIME 可以作为未知内容的用途声明；若它与高置信内容识别冲突，仍在导入阶段返回原有 `MediaError`。
- 已保存为 `application/octet-stream` 的 Media 改名 `.png`，仍不能通过 typed Image 的添加校验。需要显式声明时，在导入 bytes 时声明；不增加事后随意改 MIME 的 setter。
- `with_export_name` 只改变返回值的导出名，不改变保存的 MIME，也不追溯修改先前 Content 捕获的名称。
- 新校验不重新读取源路径，不解码媒体，不复制或重新创建快照。

**添加通过不等于构建通过。** 当前 native lowering 仍按暂存文件、内容签名和导出名执行既有检查，本次不重构 MIME 传递管线。例如：

| 输入 | 添加阶段 | 后续构建阶段 |
| --- | --- | --- |
| 真 PNG，导出名 `wrong.mp3`，使用 `.image()` | MIME 用途匹配，通过 | 既有高置信签名／扩展名冲突检查拒绝 |
| octet-stream 改名 `picture.png`，使用 `.image()` | 拒绝 | 不进入构建 |
| 未识别二进制 bytes 显式声明 `image/x-example` | 用途检查通过 | 继续既有行为；可能出现 UNKNOWN_MIME warning，不保证原声明被下层保留 |
| 真 PNG 改用未知扩展名 | 用途检查通过 | 不因“扩展名与默认值不同”自动拒绝；仍做原有检查 |

### 3.4 显式 HTML 与资产

用途检查仅针对 Image/Sound 语义节点。`Content::html`、模板 HTML/CSS 和脚本内容的媒体用途仍由作者负责，不新增字符串扫描来推断用途。

这些内容仍须通过 `NoteTypeBuilder::asset` 或 `Project::add_asset` 声明依赖；已有本地引用、缺失资源、命名冲突和构建检查继续生效。显式资产可以是字体、CSS、PDF 或未知资源，不因用途矩阵而被禁止。

不增加 `unchecked_image`、`unchecked_sound` 或跳过整个验证流程的选项。

## 4. 结构化 AddError

### 4.1 公开接口

保留 `AddError` 的私有字段和现有 `kind()`、`code()`；增加两个访问器。新增类型全部位于既有领域模块，不导出内部 IR，也不要求经过 JSON 才能读取原生信息。

```rust
// ankiforge::note
impl AddError {
    pub fn context(&self) -> &AddContext;
    pub fn detail(&self) -> Option<&AddDetail>;
}

pub struct AddContext { /* private */ }

impl AddContext {
    pub fn note_key(&self) -> Option<&str>;
    pub fn model_key(&self) -> Option<&str>;
    pub fn target(&self) -> &AddTarget;
}

#[non_exhaustive]
pub enum AddTarget {
    NoteKey,
    Note,
    Model,
    #[non_exhaustive]
    Field {
        field_key: schema::FieldKey,
        content_path: Option<Vec<usize>>,
        byte_range: Option<std::ops::Range<usize>>,
    },
    #[non_exhaustive]
    Tag { index: usize, value: String },
    #[non_exhaustive]
    Deck { name: String, inherited: bool },
    #[non_exhaustive]
    ModelAsset { media_name: String },
    #[non_exhaustive]
    OcclusionImage { media_name: String },
    #[non_exhaustive]
    ExplicitAsset { media_name: String },
    #[non_exhaustive]
    ProjectDefaultDeck { name: String },
}

#[non_exhaustive]
pub enum AddDetail {
    ModelDefinitionConflict,
    #[non_exhaustive]
    ModelNameConflict {
        existing_model_key: String,
        conflicting_name: String,
    },
    #[non_exhaustive]
    MediaConflict {
        kind: media::MediaConflictKind,
        existing_name: String,
        incoming_name: String,
    },
    #[non_exhaustive]
    MediaUsage {
        requested: media::MediaUsage,
        media_name: String,
        media_type: String,
    },
}

// ankiforge::media
#[non_exhaustive]
pub enum MediaConflictKind { PortableNameCollision, DifferentContent }

#[non_exhaustive]
pub enum MediaUsage { Image, Sound }
```

上述为接口规格，省略 derives、文档和原有方法。新 struct/enum 保持 `Debug + Clone + PartialEq + Eq`；两个纯类别 enum 另有 `Copy`。AddError 继续支持 `std::error::Error + Send + Sync + 'static`。

对新公开 enum 及含字段的变体使用 `#[non_exhaustive]`，示例匹配使用 `..` 和兜底分支；不为此重做全库既有错误 enum。新增 `AddErrorKind::InvalidMediaUsage` 是明确的接口变更，按本次版本迁移处理。

### 4.2 上下文不变量

- `Project::add` 产生的错误总是具有尝试添加的原始 note key 和 model key，即使 note key 本身无效。
- `add_asset` 的错误没有 note/model key，target 为 ExplicitAsset。
- 构建对项目默认卡组的检查仍可保留 AddError 为 BuildError 的真实 source；该错误没有 note/model key，target 为 ProjectDefaultDeck。
- AddContext 内部用私有 scope enum 表达这三种来源，避免两个 key 任意组合；外部只读访问。
- target 只有一个。不存在 field、tag、deck 等一组互相矛盾的可选位置。
- `content_path = None` 表示字段本身的问题；`Some([])` 表示字段内容根节点；`Some([2, 0])` 表示第三个 sequence 元素里的第一个元素。全部索引从 0 开始。
- 路径对应原始嵌套 Content，验证前不 flatten。不存在的字段或被覆盖的 IO 保留字段不虚构内容路径。
- `byte_range` 仅用于 Text/Html 叶子中原始 U+001F 的 UTF-8 半开区间，且必须同时存在 content_path；它不指向转义后的 HTML。媒体节点没有字节区间。
- ModelAsset 用模型实际持有的导出名定位，不承诺恢复 `.asset()` 的原始添加索引：模型完成时已经排序、去重。
- 新 detail 的合法组合由私有专用构造函数保证，`None` 表示没有额外证据；不靠缺少 detail 区分两类模型冲突。

### 4.3 错误覆盖表

| 失败情形 | target | detail |
| --- | --- | --- |
| note key 无效／重复 | NoteKey；原始 key 在 context | None |
| IO 缺少结构化图片／mask | Note | None |
| 覆盖 IO 的 image／occlusion 字段 | Field，具体 key，path=None | None |
| 相同 model key 对应不同定义 | Model | ModelDefinitionConflict |
| 不同 model key 对应冲突显示名 | Model | ModelNameConflict，含既有模型 key 和 trim 后参与比较的名称 |
| 未知字段／必填字段无值 | Field，path=None | None |
| 原始 U+001F | Field，path 和 byte_range 均存在 | None |
| 笔记显式卡组无效 | Deck，inherited=false | None |
| 笔记继承的项目卡组无效 | Deck，inherited=true | None |
| 构建检查项目默认卡组失败 | ProjectDefaultDeck | None |
| 标签无效、保留或重复 | Tag，原始 value 和失败的 index | None |
| 模型携带资产与其他资产冲突 | ModelAsset | MediaConflict |
| IO 主图与其他资产冲突 | OcclusionImage | MediaConflict |
| 字段内容携带媒体发生冲突 | Field，含 content_path | MediaConflict |
| 显式 add_asset 冲突 | ExplicitAsset | MediaConflict |
| typed Image／Sound 用途不合 | Field，含 content_path | MediaUsage |

保留现有 code 及已有校验含义，不借机细分“必填字段未赋值／赋空值”或标签所有子原因。媒体冲突保留现有两个 code，并提供双方原始文件名；不暴露摘要算法，不持有双方媒体字节，也不建立既有资产的长期来源索引。

新增用途错误固定为：

```text
kind: InvalidMediaUsage
code: NOTE.MEDIA_USAGE_INVALID
```

用途不匹配不属于 MediaConflict；不存在两项资源冲突。

### 4.4 文案、source 与资源所有权

`Display` 供人阅读，不是解析协议。新上下文的传输标签采用显式映射，不由 `Debug` 文案生成。

直接验证失败保持 `source() = None`；不制造一个未发生过的 MediaError 或 I/O cause。已有 BuildError → AddError source 链仍可 downcast 获取上下文。

错误只拥有标识、文件名、MIME、索引和范围等诊断数据，不持有 Note、NoteType、Media、Project 或临时文件句柄。保存错误不能延长媒体快照的文件寿命。

`add(key, Note)` 继续消费 Note，失败只保证项目不变。Rust 调用者若需要重试可预先 clone；本次不增加 `try_add`、RejectedNote 或借用式添加。Node/Python 继续按已有方式 clone native Note，保持其现有行为。

## 5. 私有实现安排

1. 将内部 AssetConflict 从 `code + message` 改为有类型的冲突类别及双方文件名；code/message 从事实生成。Assets 的碰撞、幂等和 content equality 规则不变。
2. SchemaError 与 AddError 均从该内部事实构造各自错误；本次不扩展 SchemaError 的公共上下文接口，不改变它已有的 template location。
3. Content 增加一个私有校验 helper，统一原有 U+001F 检查和新用途检查，失败返回局部路径、范围及原因。Project 为其补 note/model/field 上下文。
4. 媒体收集保留独立的私有遍历职责，callback 携带借用的 sequence 路径。不要先 flatten 成失去来源的 `Vec<Media>`；只在需要记录错误时克隆路径。
5. 项目已登记 Assets 和本次添加的暂存 Assets 都执行同一冲突检查。两种失败均返回 incoming 的位置及双方文件名，不要求返回既有资产最早来自哪条笔记。
6. 所有检查完成后才合并资产、登记模型和笔记。成功路径不增加媒体解码、磁盘读取或内容摘要计算。

首错顺序明确为：note key → IO 结构／保留字段 → 模型冲突 → 字段及内容 → required 字段 → deck → tags → 依赖冲突。字段按稳定 key 顺序，Content 按深度优先、从左到右的原始 sequence 顺序；文本叶子返回第一个 U+001F。依赖收集顺序为模型已排序资产、IO 主图、按字段 key 和内容顺序遍历的媒体。

本次不保证一个同时包含新旧多种错误的输入仍返回旧版本相同的首错；保证新顺序确定、旧单一错误的 code 不变。新增媒体用途检查失败时也不能留下任何登记状态。

## 6. Node、Python 与工具接入

### 6.1 SDK 原生错误桥

Node/Python 增加 AddError 的专用映射，顶层错误及 source chain 中的 AddError 都覆盖。映射使用 Rust 的 context/detail 访问器，禁止解析 Display。

- Node：`error.details.context` 与 `error.details.detail`；context 使用 `noteKey`、`modelKey`、`target`，target/detail 字段用 camelCase。
- Python：`error.details["context"]` 与 `error.details["detail"]`；相应字段用 snake_case。
- context 中两种 key 始终存在，非 Note 来源为 null／None；detail 没有额外证据时为 null／None。
- target/detail 的 `type` 为显式 snake_case 标签，与 Rust 变体一一对应；usage 为 `image`／`sound`，conflict kind 为 `portable_name_collision`／`different_content`。
- range 编码为 `{start, end}`，content path 编码为索引数组或 null；字符串保留原值。
- Node 让 `AddError.details` 本身具有精确的只读 context/detail 联合类型，并沿用深冻结；不能只导出类型却继续把该属性留为 `Record<string, unknown>`。
- Python 让 `AddError.details` 具有 TypedDict/联合类型，并保留原有 `error_kind`、`causes`、`source_details` 字段。为避免 TypedDict 覆写 `dict[str, Any]` 属性的类型冲突，仅在类型层将 ForgeError 参数化：`DetailsT` 约束为 `Mapping[str, Any]`，AddError 绑定 `AddErrorDetails`，其他异常显式绑定既有 `dict[str, Any]`。需要表示任意异常的内部类型注解用 `ForgeError[Any]`。异常继承、构造数据、cause 链及运行时 `.details` 行为不变，不新增全库运行时验证框架。
- 保持现有 `.details` 访问习惯，不增加一套并行 snapshot 或 context/detail property API。
- Node sourceDetails / Python source_details 中的 AddError 项包含 `type: "add"`、kind、code、context、detail；不只剩 message。

示例：PNG 被放入声音节点，且它位于 front 字段的嵌套序列 `[1, 0]`，Node 中必须可以直接读取：

```json
{
  "context": {
    "noteKey": "cell",
    "modelKey": "vocabulary",
    "target": {
      "type": "field",
      "fieldKey": "front",
      "contentPath": [1, 0],
      "byteRange": null
    }
  },
  "detail": {
    "type": "media_usage",
    "requested": "sound",
    "mediaName": "cell.png",
    "mediaType": "image/png"
  }
}
```

不新增全库 Serialize 要求、Rust 错误 snapshot DTO 或 JSON 错误框架。绑定层为既有传输协议生成 JSON，原生 Rust 调用者始终读取有类型的值。

Node native 协议从当前 4 升到 5，SDK/native 版本检查和安装后测试同步；旧 native 不能被新 wrapper 静默接受。Python wrapper、native extension、`.pyi` 和 wheel/sdist 一起迁移发布；在已有 metadata 的版本检查中增加预期 `contract_version == "2.0.0"` 的精确校验，不另引入协议系统。这样即使 wrapper/native 仍同为未发布的 0.2.0，新 wrapper 也会拒绝携带 bundle 1.0.0 的旧 native；不对无法追溯修改的旧 wrapper 作同等保证。

### 6.2 工具 JSON 与 CLI

Image/Sound JSON 内容必须通过同一个 Content → Project::add 路径，不在 loader 中另写一套 MIME 规则。

`tools::load_project` 继续用 anyhow 包装，具体 AddError 可通过 source chain/downcast 获得。CLI 的现有失败输出不因此被宣称升级成结构化 BuildSnapshot；本次不新增 CLI 错误 envelope。流式输入预算和拒绝未知／重复字段规则继续保留。

## 7. 版本与契约治理

删除有效的项目标题输入是破坏性变更，不能仅因为使用场景少就标为兼容修复。

| 版本轴 | 处理 |
| --- | --- |
| 工具 recipe | `ankiforge-project-v1` → `ankiforge-project-v2`，仅接纳新格式 |
| Contract bundle | 相对当前已合并的 1.0.0，提升到 2.0.0，分类为 `behavior_changing_incompatible`，附 migration_notes |
| Rust crate | 实施前核查权威发布状态：0.2 未发布则并入 0.2.0；若已发布则进入下一个 minor，至少 0.3.0 |
| Node/Python 包 | 各自按真实已发布版本执行破坏性版本策略；wrapper/native/package metadata 一致 |
| Node native 内部协议 | 4 → 5 |
| APKG 身份证据 | `ankiforge-identity-v1` 不变 |
| build/report/comparison snapshot | 形状未变，版本不变 |

本地 changelog 的 Unreleased 不能替代正式发布状态核查；这是一条执行前事实检查，不是待定的产品选择。不得为了避免 bundle major 而选取陈旧 Git 基线，也不得让 crate version 与 bundle version 强制同步。

按照 [契约变更政策](../process/contract-change-policy.md)：

1. 实施前记录对应 RFC/ADR，说明用途校验、结构化上下文和项目标题移除的行为变化。
2. 注册 `NOTE.MEDIA_USAGE_INVALID`。原 `BUILD.NAME_INVALID` 保留条目并标记 deprecated，不删除、不复用，也不为它保留旧 setter。
3. 更新 project-input schema、native-authoring semantics、相关 fixtures、manifest 和实际变更的 component bookkeeping。
4. 相对真实 PR 基线，用 contract tooling 生成准确 before/after inventory 和 `versioning/changes/2.0.0.yaml`；任何后续资产变更后重新生成，不手写摘要。
5. 更新嵌入 bundle、crate/bundle 映射、发布检查、changelog 和当前文档；保留历史验证记录原貌。

创建 release tag、发布包均不属于本次设计任务。

## 8. 实施切片

| 切片 | 内容 | 局部验证 |
| --- | --- | --- |
| A：错误定位 | 新 typed context/detail、AssetConflict 事实、带路径的内容/依赖遍历、各既有 AddError 构造点 | Rust 原生 consumer 覆盖完整错误表及 source chain；行为规则不变 |
| B：媒体用途 | 接入 MIME 用途矩阵、新错误类别/code、原子失败路径 | 真媒体、嵌套路径、快照和构建阶段分界案例 |
| C：跨语言错误 | native mapper、SDK 类型、顶层与 source details、协议版本 | Node/Python 同一错误的定位与证据一致，安装产物测试 |
| D：移除项目标题 | Rust/SDK/tool recipe/schema/decoder/fixtures 整体迁移 | 编译拒绝旧入口；schema/loader v2 一致；展示和身份行为回归 |
| E：契约与分发收口 | 补齐 RFC/ADR 的实现及验证记录、registry/manifest/inventory、嵌入资源、文档和示例 | 完整治理、工作区、packaged consumer、文档和发布门禁 |

RFC/ADR 的初始设计记录在 A 之前完成。切片属于同一项变更的实施顺序，不把临时未完成的 schema/registry 状态单独标记为可发布。每个实际提交保持可编译，最终 contract inventory 必须反映全量实际变更。

## 9. 验收标准

### 媒体

- 真 WAV→Image、PNG→Sound 在 add 失败，kind/code/context/detail 正确；失败后同 note key 可用于合法重试。
- PNG/SVG→Image、WAV→Sound、真实 MP4/WebM→Sound 接受；检查产物中的实际引用和媒体 bytes，不只检查文件存在。
- `application/octet-stream` 拒绝 typed 节点；显式未知 image/video MIME 在 add 通过，构建结果按现有规则检查，不宣称格式认证。
- MIME 大小写/参数、显式声明与高置信识别冲突、改导出名的四种阶段案例符合第 3 节。
- 普通、自定义、Cloze 和 IO 可编辑字段使用同一校验；嵌套 sequence 返回准确路径。
- 修改／删除原文件、跨项目复用、clone/drop、命名前后捕获、特殊文件名 URL/sound 转义的既有保证不变。
- raw HTML、字体/CSS 和显式资产保持既有行为；缺失引用仍被拒绝。

### 错误与事务性

- 第 4.3 节每个产生路径均断言结构化字段，不解析 Display。
- 中文/emoji 前缀后的 U+001F 返回原始叶子的 UTF-8 byte range；根、嵌套、空路径与 None 的语义分别覆盖。
- 两类模型冲突、两类媒体冲突、标签 index、继承／显式 deck、默认 deck source context 全部覆盖。
- 资产冲突分别覆盖项目已有项、本次 Note 内部多个依赖和显式 add_asset；相同资产仍幂等。
- 失败不登记 note/model/assets；用后续合法重试和导出内容证明，而非只断言 Project::len。
- 错误可在 Note/Project 被丢弃后读取，且不延长临时媒体快照寿命。
- 原生消费者能命名所有传递类型，不依赖 internal-tools 或 serde；anyhow/source 链保留 AddError 的上下文。
- SDK 顶层与嵌套 source details 均保留相同事实，新 wrapper 拒绝旧 native。Python 另覆盖“binding/core 均为 0.2.0、contract 为 1.0.0”的旧 native，不能仅靠包版本相等放行。
- TypeScript 与 Python 静态消费者直接从捕获的 `AddError.details` 读取 context/detail，按 `type` 标签收窄并访问字段；不能依赖类型断言或退回 Any。Python 使用项目支持版本的 typing 语法与现有类型检查器验证。

### 标题移除与版本

- Rust 旧 `.name/.display_name` 不再编译；Node 旧 `.name` 不存在且类型检查拒绝；Python 旧 `name=` 不被接纳，旧 property 不再存在。
- v1 无 name 的最小输入因版本被拒绝；v2 中 name 的字符串/null 均被拒绝；合法 v2 的 schema 和 loader 结果一致。
- 模型/字段/模板显示名称仍可用；default_deck、Note::deck、多卡组目的地仍正确。
- 从原有 identity-v1 基线继续构建更新，GUID、模型/config 身份、ordinal 和未变内容的 mtime 继续保持；不要求历史 APKG 重新生成。
- 正确更新 bundle 版本和嵌入清单；build/report/comparison JSON 形状保持原契约。

实施后的检查包括相关 default-feature 与外部消费者测试、Node/Python native 重建及安装后测试、文档/示例检查、contract verify/summary/package/governance，以及仓库 `make verify-ci` 要求的完整检查。测试结果、MSRV／平台与实际 Anki 导入证据分别记录，不能以工作区测试代替分发或客户端验收。

## 10. 设计审查记录

本方案由主代理与三个 GPT-6 Astra 子代理分别研究媒体、错误和标题后汇总。已采纳的反证与简化：

- Sound 同时包含视频，不能以 audio-only 校验。
- MIME 用途通过仅是 add 阶段事实；现有 lowering 仍有后续签名／扩展名校验。
- 不公开新 digest，不储存媒体既有来源图，不添加整套错误 snapshot 框架。
- 默认 deck 的 AddError 可能来自 build，不能声称所有 AddError 都有 note key。
- 删除标题需要更新正式工具输入契约及 bundle 版本，不能只删除 Rust setter。
- 交叉审查发现 Python 仅比较包版本不足以拦截同版本旧 native；已利用现有 contract_version metadata 增加精确检查。
- 新 enum 和其含字段变体分别声明 non_exhaustive，明确未来新增变体与新增字段的不同兼容含义。
- SDK 精确类型落实到 AddError.details 的实际属性；Python 通过最小的类型参数化保留运行时行为，避免 TypedDict 的无效属性覆写。

最终交叉审查：媒体语义和生命周期无遗留问题；错误接口的属性可扩展性与 SDK 类型问题已修正；标题迁移的 Python 旧 native 检查与 RFC/ADR 顺序问题已修正。没有待定的产品选择；发布状态核查属于实施前事实检查。

本文件保留已批准的设计规格；实现结果、实际验收证据与平台边界见 [实施记录](2026-09-28-rust-api-validation-and-errors-implementation.md)。
