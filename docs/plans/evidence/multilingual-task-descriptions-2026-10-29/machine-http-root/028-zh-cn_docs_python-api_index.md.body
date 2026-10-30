# Python API

Canonical page: https://ankiforge.dev/zh-cn/docs/python-api/
Language: zh-CN
Description: Project、媒体、构建选项、报告和临时产物的所有权。

SDK 通过原生扩展调用默认 Rust 公开 API，包含 `py.typed` 和经过检查的公开签名。从[入门](https://ankiforge.dev/zh-cn/docs/python-quickstart/)开始。

## Project 与笔记值

| 入口 | 行为 |
| --- | --- |
| `Project(namespace, *, default_deck=None)` | 校验显式稳定命名空间并配置默认牌组 |
| `project.add(key, note)` | 原子收集笔记、模型及拥有所有权的媒体；重复笔记键失败 |
| `project.add_asset(media)` | 包含显式原始 HTML/CSS/脚本依赖 |
| `len(project)` | 成功添加的笔记数 |
| `Note.basic(front, back)` | 内置 Basic 模型；字符串表示 Text |
| `Note.cloze(text)` | 内置 Cloze 模型；字符串表示 Text 并保留填空语法 |
| `note.field(key, content)` | 返回已赋值的新笔记；添加时校验键 |
| `note.deck(name)` / `.tag(tag)` / `.tags(tags)` | 返回配置后的笔记 |
| `note.note_type` | 拥有所有权的不可变模型 |

普通字符串成为 Text。`Content.text(value)` 和 `Content.html(value)` 是显式构造函数；`Content.sequence(iterable)` 组合字符串与内容，不预先渲染或丢失媒体依赖。字段不接受原始文件名作为媒体引用。

## 模型与模板

`NoteType.builder(key)` 返回不可变构建器。`.name`、`.field`、`.template`、`.css`、`.cloze_field` 与 `.asset` 返回更新的构建器。`.build()` 校验完整定义并返回不可变 NoteType。`model.note()` 创建拥有该模型的笔记。`NoteType.from_bundle(path, limits=MediaLimits())` 从当前模板包格式加载相同模型及资源；可选预算在读取前分别应用于每个资源。

`Field(key, name=None, required=False, sort=False)` 分离稳定键与显示名称。`Template(key, front, back, name=None, browser_front=None, browser_back=None, target_deck=None, generation=GenerationRule())` 的每个源字符串使用字段键。`GenerationRule.all(keys)` 和 `.any(keys)` 配置卡片生成，默认推断 Anki 的字段存在条件。构建器的 `.cloze_field(key)` 声明自定义填空生成。

## 媒体与遮挡

| 入口 | 行为 |
| --- | --- |
| `Media.file(path, *, limits=MediaLimits())` | 立即读取拥有所有权的快照 |
| `Media.bytes(data, media_type, *, limits=MediaLimits())` | 拥有带显式 MIME 类型的字节 |
| `media.with_export_name(name)` | 返回共享快照的新文件名值；之前的引用保留原名 |
| `media.filename`, `media.media_type`, `len(media)` | 只读快照元数据 |
| `media.image()` / `.sound()` | 保留快照的类型化内容 |

`MediaLimits(max_bytes=256 << 20)` 在创建快照前控制每次导入。大媒体可写入拥有所有权的临时存储。名称冲突在全部显式与自动收集资源间校验。

`Note.image_occlusion(media)` 返回 ImageOcclusionBuilder。链式调用 `.mask(Mask.rect(key, x, y, width, height))`、可选 `.mode(OcclusionMode.HIDE_ONE_GUESS_ONE)`，再调用 `.build()`。默认模式为 `HIDE_ALL_GUESS_ONE`。完成时解码图像，并校验稳定遮罩键和有限、位于图内的像素坐标。在返回的 Note 上可正常设置 `header`、`back_extra` 或 `comments`。生成的 `image` 与 `occlusion` 字段不能手动赋值。

## 操作与结果

| 入口 | 结果 |
| --- | --- |
| `project.build(BuildOptions.to(path))` | 保证有持久产物的 BuildOutput |
| `project.build(BuildOptions.temporary())` | 拥有临时产物的 BuildOutput |
| `options.update_from(path)` | 返回基于完整基线证据更新的选项 |
| `project.compare(CompareOptions.against(path))` | 完成的 ComparisonReport，含独立发布策略决定 |
| `options.update_policy(policy)` | 为 Update/Compare 应用显式风险策略；Create 不适用 |
| `options.inspect_limits(limits)` | 为基线与候选包分别应用有限限制 |

BuildOutput 提供 `.artifact`、`.report` 和 `.snapshot()`。BuildReport 提供 `.counts`（含 notes/cards/media 的 BuildCounts）、`.diagnostics`、`.comparison` 和 `.snapshot()`。报告没有操作结果或文件所有者。ComparisonReport 提供 `.findings`、`.policy`、`.allows_publication` 与 `.snapshot()`。快照是普通可 JSON 序列化字典，不含产物所有权。

ApkgArtifact 提供 `.path`、`.persist_to(path)` 和 `.close()`。使用 `copy.copy` 复制会保留另一个所有者；上下文管理器在退出时释放该所有者。最后一个所有者释放后，临时输出消失。持久路径在句柄清理后仍存在。持久化失败不影响源句柄继续使用。

`UpdatePolicy()` 阻止 High 和 Critical 发现项。`.fail_on(RiskLevel.MEDIUM)` 改变阈值。`.allow('RISK.NOTE_REMOVED')` 显式接受整个已注册类别，保留证据。未知码立即失败。`InspectLimits` 提供全部有限字节或数量预算，包括 `max_identity_bytes`；不能禁用必需检查。

## 错误与并发

失败抛出 SchemaError、AddError、MediaError、ImageOcclusionError、TemplateBundleError、CompareError、PolicyError、BuildError 或 PersistError。它们提供 `kind`、`code`、`details`、`source_details` 中的结构化观察结果和 `causes` 中的来源链文本，并将原生异常保留为 `__cause__`。可用时 I/O 原因保留链接的 OSError。BuildError 提供 `.report` 和 `.snapshot()`；CompareError 提供部分 `.report`；PersistError 详情包含实际发布事实。

原生 Project 操作释放 GIL，并以 `BINDING.PROJECT_BUSY` 拒绝同时使用同一对象；独立项目可并发运行。进程 fork 后重新创建全部原生值与句柄。继承的句柄以 `BINDING.FORKED_OBJECT` 拒绝操作，其清理不能删除父进程拥有的文件。子进程创建的值可正常操作和清理。参见[诊断](https://ankiforge.dev/zh-cn/docs/python-diagnostics/)。

## 添加上下文

添加失败不会改变项目。结构化上下文记录原始笔记或模型键和目标，并可附带冲突或媒体用法详情。Rust 通过 `AddError::context()` / `detail()` 提供 `note::{AddContext, AddTarget, AddDetail}`；SDK 通过 `AddError.details` 提供类型化 context/detail。Node 字段使用 camelCase 且深度冻结；Python 使用 snake\_case TypedDict。标签保留插入下标；字段位置保留原始序列下标（null/None 指字段，\[\] 指其根）以及可选 UTF-8 字节范围。构建产生的默认牌组错误不带笔记或模型键，仍可从来源链读取。

添加时，类型化图片要求 image/\* MIME，声音引用接受 audio/\* 或 video/\*。其他类别产生 `NOTE.MEDIA_USAGE_INVALID`。构造函数保持不会失败；显式资源和原始 HTML 不受影响。检查使用导入时保留的 MIME，不认证播放能力。构建的独立 MIME/扩展名校验仍然适用。完整 target/detail 表见[设计](https://github.com/morehardy/anki-forge/blob/main/docs/plans/2026-09-28-rust-api-validation-and-errors-design.md)。

## 源码 API 详情

批量媒体与预备发布已包含在公开 `0.3.1` 包中。

### 预备、审阅与发布

```python
from ankiforge import Project, Note, BuildOptions, BuildError
project = Project('review-once').add('one', Note.basic('Question', 'Answer'))
previous = project.build(BuildOptions.to('previous.apkg'))
previous.artifact.close()
prepared = project.prepare_publication(
    BuildOptions.to('next.apkg').update_from('previous.apkg')
)
try:
    report = prepared.report  # observations; does not own the candidate
    if report.comparison is None or report.comparison.allows_publication:
        output = prepared.publish()
        output.artifact.close()  # persistent next.apkg remains
except BuildError as error:
    print(error.snapshot()['result'])  # actual publication/durability facts
    raise
finally:
    prepared.close()  # idempotent; deletes an unused candidate
```

一次发布尝试消耗所有者，包括策略或 I/O 失败。之后调用抛出 `PreparedPublicationStateError`，码为 `BUILD.PREPARED_UNAVAILABLE`，原因是 `closed` 或 `consumed`；close 不取消已经运行的发布。相对路径在预备调用时绑定，Project 更改不能修改已审阅 APKG。报告关闭后仍可读，但不保留文件。改变选项或重试需重新预备。耗时不含审阅等待。晚期错误可能表示替换成功但持久性未确认；决定下一步前检查发布事实。要求原生绑定协议 1 和契约包 2.1.0；更旧的同版本二进制会被拒绝。
