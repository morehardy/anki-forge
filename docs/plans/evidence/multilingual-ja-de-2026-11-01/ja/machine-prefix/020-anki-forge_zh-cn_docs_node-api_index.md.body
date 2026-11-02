# Node 与 TypeScript API

Canonical page: https://morehardy.github.io/anki-forge/zh-cn/docs/node-api/
Language: zh-CN
Description: 公开方法、异步行为、构建选项和报告。

`ankiforge` 入口提供唯一的 Project 内容编写模型。类型定义位于[公开源码](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/index.ts)、[报告类](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/report.ts)及 [JSON 快照声明](https://github.com/morehardy/anki-forge/blob/main/bindings/node/src/snapshots.ts)。从[入门](https://morehardy.github.io/anki-forge/zh-cn/docs/node-quickstart/)开始。

## Project 与笔记

| API | 行为 |
| --- | --- |
| `new Project(namespace)` | 校验稳定发布命名空间 |
| `.defaultDeck(name)` | 设置默认目标；返回项目 |
| `.add(key, note)` | 原子收集笔记、模型和媒体；重复键失败 |
| `.addAsset(media)` | 包含显式原始 HTML/CSS/脚本资源 |
| `.clone()` | 独立可编辑项目，共享不可变值 |
| `.length` | 已注册笔记数 |
| `Note.basic(front, back)` / `Note.cloze(text)` | 持有内置模型的不可变笔记 |
| `model.note().field(key, content)` | 持有模型的不可变自定义笔记 |
| `note.deck(name).tag(tag).tags(iterable)` | 返回重新配置的笔记 |

字符串始终成为 Text，包括 Cloze 字符串。显式标记使用 `Content.html`。`Content.sequence(iterable)` 组合文本、HTML 与类型化媒体。使用显式业务键；内容和显示名称都不会派生身份。

## 不可变模型

```js
import { NoteType, Field, Template, GenerationRule } from 'ankiforge';
const model = NoteType.builder('vocabulary')
  .name('词汇')
  .field(new Field('front', { name: '正面', required: true, sort: true }))
  .field(new Field('back', { name: '背面' }))
  .template(new Template('recognition', {
    front: '{{front}}',
    back: '{{FrontSide}}<hr>{{back}}',
    generation: GenerationRule.all(['front']),
  }))
  .build();
const note = model.note().field('front', 'cell').field('back', '细胞');
```

构建器返回新值。完成的模型没有修改字段或模板的方法。`builder.clozeField(key)` 选择自定义 Cloze 语义；`builder.css` 与 `builder.asset` 提供样式和显式依赖。模板还接受 `name`、`browserFront`、`browserBack` 和 `targetDeck`。模板引用及生成条件使用字段键，显示名称编译后供 Anki 使用。`GenerationRule.ankiDefault()` 选择 Anki 默认行为；`all` 和 `any` 选择显式条件。

`await NoteType.fromBundle(path, mediaLimits?)` 从 `template-bundle-v2` 返回相同的不可变模型。成功后导入文本和资源不再依赖源目录。模型可跨项目复用；同一模型键下不兼容的定义会原子地失败。

## 媒体与图片遮挡

`await Media.file(path, limits?)` 和 `await Media.bytes(Uint8Array, mime, limits?)` 返回拥有所有权的快照。await 成功后源文件可改变或消失。`withExportName(name)` 返回重命名值，不改变之前的克隆或内容。通过属性读取 `filename`、`mediaType` 和 `byteLength`。

`media.image()` 与 `.sound()` 返回自动将资源带入项目的类型化 Content。显式 `project.addAsset` 与模型 `builder.asset` 处理原始 HTML/CSS/脚本引用。媒体限制接受 `{ maxBytes }`，默认为 256 MiB；原生字节没有 64 KiB 内联限制。

```js
import { Media, Note, Mask } from 'ankiforge';
const image = await Media.file('diagram.png');
const note = Note.imageOcclusion(image)
  .mask(Mask.rect('nucleus', 10, 10, 20, 20))
  .mask(Mask.rect('wall', 50, 50, 20, 20))
  .mode('hide_one_guess_one')
  .build().field('header', 'Cell');
```

默认模式为 `hide_all_guess_one`；两种模式都为带键遮罩生成独立卡片。`.build()` 校验解码尺寸、有限矩形和唯一键。`image` 与 `occlusion` 是保留的生成字段。解码与更新限制见[图片遮挡](https://morehardy.github.io/anki-forge/zh-cn/docs/image-occlusion/)。

## 构建、比较与输出

| API | 结果 |
| --- | --- |
| `await project.build(BuildOptions.to(path))` | 拥有持久产物的 BuildOutput |
| `await project.build(BuildOptions.temporary())` | 拥有临时产物的 BuildOutput |
| `await project.compare(CompareOptions.against(path))` | 完成的 ComparisonReport，包括被阻止的策略 |
| `options.updateFrom(path)` | 配置基于完整原始分发证据的更新 |
| `options.inspectLimits(limits)` | 覆盖独立检查计数器 |
| `options.updatePolicy(policy)` | 配置更新/比较风险策略 |

成功输出提供 `.artifact`、`.report` 与 `.snapshot()`。`BuildReport` 提供 `.counts`、`.baselineCounts`、`.durationMs`、`.diagnostics`、`.comparison` 和 `.snapshot()`；没有结果或文件所有权。`.comparison` 是 `ComparisonReport` 或 `null`，与 `project.compare()` 使用同一类。比较对象提供 `.findings`、`.highestRisk`、`.policy`、`.diagnostics` 和 `.snapshot()`。快照字段名对应 Rust JSON，包括 `allows_publication` 与 `schema_version`，并非原生报告的 camelCase 副本。检查非 `null` 后，在比较对象上使用 `report.comparison.highestRisk`，在 JSON 快照中使用 `report.snapshot().comparison.highest_risk`。

`new UpdatePolicy()` 阻止 High/Critical。`.failOn('medium')` 改变阈值。`.allow('RISK.NOTE_REMOVED')` 显式接受整个类别，保留证据和严重程度。未知码和硬错误不能放行。仅新建构建上设置策略会报错。需要相同发布决策时，对 compare 和 build 使用相同策略与限制。

检查选项包括 `maxArchiveBytes`、`maxEntries`、`maxCentralDirectoryBytes`、`maxZipEntryBytes`、`maxZipTotalBytes`、`maxMetaBytes`、`maxMediaMapBytes`、`maxIdentityBytes`、`maxCollectionBytes`、`maxMediaBytes`、`maxDecodedTotalBytes` 和 `maxZstdWindowBytes`。值可为非负安全整数，或不超过 u64 最大值的 `bigint`；零是实际零预算。计数器独立应用于基线与候选包。未知选项失败。

## 产物寿命与并发

JSON 快照路径使用导出的 `PathSnapshot` 类型。Unicode 路径为字符串；非 Unicode 原生路径使用 `{ encoding: "unix_bytes", bytes: number[] }` 或 `{ encoding: "windows_wide", units: number[] }`。数组保留精确的 Unix 字节或 Windows UTF-16 代码单元，适用于构建快照中的产物路径、失败发布路径及结构化错误路径详情。只在原平台解码路径；Unix 中 `Buffer.from(value.bytes)` 为 Node 文件系统函数保留字节路径。Node 内容编写方法仍接受字符串路径。

相对媒体和模板包路径在调用读取时锚定；构建目标和基线在构建选项时锚定。POSIX 中，符号链接后接 `..` 保留文件系统遍历语义。`persistTo` 将相对目标锚定到原构建捕获的工作目录，包括克隆和持久化句柄。之后的工作目录变化不会重定向这些路径。

消费 `.path` 时保持产物所有者存活。`.clone()` 创建独立所有者；`await .close()` 释放该所有者。最后一个临时所有者删除文件。`await .persistTo(path)` 返回持久产物，失败时原所有者也仍可用。保留所有者期间，标准 Node 文件流可以读取产物。

快照是冻结的 JSON 数据，不保留原生文件。持久文件在所有者清理后仍存在。调用方不能构造 `BuildOutput`；它只由成功构建创建。

Build 与 compare 异步运行，使用调用时捕获并拥有所有权的项目快照。后续编辑不能改变该请求。媒体读取和模板包加载也在工作线程上执行。Node ESM 与 CJS 共享相同原生模块和类身份。

## 错误

`SchemaError`、`AddError`、`MediaError`、`TemplateBundleError`、`ImageOcclusionError`、`PolicyError`、`ConfigurationError`、`CompareError`、`BuildError` 和 `PersistError` 继承 `ForgeError`。它们提供 `kind`、`code`、`domain`、`causes` 中的原生来源链文本、用于已识别 I/O/schema/媒体/限制原因的结构化 `sourceDetails`，以及操作 `details`。原始原生异常保留为 JS `cause`。

`BuildError.snapshot()` 保留失败与发布事实。`BuildError.report` 和 `CompareError.report` 与 `BuildOutput.report` 一样返回 `BuildReport`。报告使用 `.baselineCounts` 和 `.comparison`；snake\_case JSON 字段通过 `.snapshot()` 访问。失败比较可能只有部分观察结果，且 comparison 为 `null`。`PersistError.publication` 记录目标是否已发布、持久性是否确认。媒体或检查错误保留限制详情。请使用机器字段，不解析错误消息。`ArtifactClosedError` 表示使用已关闭所有者；`NativeLoadError` 表示原生包缺失、不兼容或陈旧。

## 添加上下文

添加失败不会改变项目。结构化上下文记录原始笔记或模型键和目标，并可附带冲突或媒体用法详情。Rust 通过 `AddError::context()` / `detail()` 提供 `note::{AddContext, AddTarget, AddDetail}`；SDK 通过 `AddError.details` 提供类型化 context/detail。Node 字段使用 camelCase 且深度冻结；Python 使用 snake\_case TypedDict。标签保留插入下标；字段位置保留原始序列下标（null/None 指字段，\[\] 指其根）以及可选 UTF-8 字节范围。构建产生的默认牌组错误不带笔记或模型键，仍可从来源链读取。

添加时，类型化图片要求 image/\* MIME，声音引用接受 audio/\* 或 video/\*。其他类别产生 `NOTE.MEDIA_USAGE_INVALID`。构造函数保持不会失败；显式资源和原始 HTML 不受影响。检查使用导入时保留的 MIME，不认证播放能力。构建的独立 MIME/扩展名校验仍然适用。完整 target/detail 表见[设计](https://github.com/morehardy/anki-forge/blob/main/docs/plans/2026-09-28-rust-api-validation-and-errors-design.md)。

## 批量媒体与预备发布

批量媒体与预备发布已包含在公开 `0.3.1` 包中。

### 预备、审阅与发布

```js
import { Project, Note, BuildOptions, BuildError } from 'ankiforge';
const project = new Project('review-once').add('one', Note.basic('Question', 'Answer'));
const previous = await project.build(BuildOptions.to('previous.apkg'));
await previous.artifact.close();
const prepared = await project.preparePublication(
  BuildOptions.to('next.apkg').updateFrom('previous.apkg'),
);
try {
  const report = prepared.report; // immutable observations, no file ownership
  console.log(report.comparison?.snapshot());
  if (report.comparison?.policy.allows_publication !== false) {
    const output = await prepared.publish();
    await output.artifact.close(); // persistent next.apkg remains
  }
} catch (error) {
  if (error instanceof BuildError) console.log(error.snapshot().result);
  // Inspect publication/durability facts: a late failure can follow replacement.
  throw error;
} finally {
  await prepared.close(); // idempotent; cleans an unpublished candidate
}
```

`publish()` 立即取得原生所有者并执行一个工作线程任务。每次尝试都会消耗它，包括策略或 I/O 拒绝。重复调用以 `PreparedPublicationStateError` 拒绝，码为 `BUILD.PREPARED_UNAVAILABLE`，`reason` 为 `consumed` 或 `closed`。发布开始后关闭不会取消它。关闭后报告仍可读。选项在预备调用时绑定；改变策略或目标需要重新预备。耗时不含审阅等待。报告和 JSON 都不能转换回所有者。

构建、比较、预备和克隆共享不可变原生 Project 版本。内容编写更改仍同步且隔离。当另一快照存活时，第一次更改可能复制 Project；之后独占更改不会。这将复制移出 Promise 提交阶段，但不承诺首次编辑更快、最差定时器延迟更好或 RSS 更低。JavaScript 字节输入仍在提交工作线程前复制。此包装层使用协议 6 二进制。

## TypeScript

在应用中安装 `typescript` 和 `@types/node`。使用 `main.mts` 或 `"type": "module"`，以及以下 `tsconfig.json`：

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "outDir": "dist"
  }
}
```

使用 `npx tsc` 编译；`.mts` 入口随后运行 `node dist/main.mjs`。
