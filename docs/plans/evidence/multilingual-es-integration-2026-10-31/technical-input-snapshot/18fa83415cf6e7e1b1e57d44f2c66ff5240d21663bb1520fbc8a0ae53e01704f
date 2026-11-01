# Python 诊断与更新策略

成功构建返回保证包含产物的 BuildOutput。失败构建抛出 BuildError，其中含观察结果和发布事实。不能仅凭诊断判定成功。

```python
import json
from ankiforge import Project, Note, BuildOptions, BuildError

project = Project('course').add('term', Note.basic('Question', 'Answer'))
try:
    output = project.build(BuildOptions.to('course.apkg'))
except BuildError as error:
    print(error.kind, error.code)
    print(json.dumps(error.snapshot(), ensure_ascii=False))
else:
    print(output.artifact.path)
    print(json.dumps(output.report.snapshot(), ensure_ascii=False))
```

根据机器 `kind` 和 `code` 作决策，不解析消息。所有公开领域异常在 `causes` 中保留来源链文本，并链接原生异常。`details` 携带操作相关证据：模板源位置、媒体路径与预算测量、不完整比较的观察结果，或发布事实。可用时，I/O 来源会成为链接的 OSError。`source_details` 保留类型化嵌套错误：模板包文本预算原因含 `limit` 和 `observed`，媒体原因含 `code`、`path` 和 `limit_exceeded`。

BuildReport 只有观察结果。其快照含数量、诊断、耗时和已完成的比较。Output 与 BuildError 快照包含实际 Success/Failure 结果。失败快照区分目标从未发布与已发布后持久性失败。JSON 路径不拥有文件，也不延长临时产物寿命。

## 发布前比较

```python
from ankiforge import CompareOptions, UpdatePolicy

comparison = project.compare(CompareOptions.against('course-v1.apkg'))
print(comparison.findings)
print(comparison.policy)
```

完成的比较即使发现 High 风险也正常返回。通过 `comparison.allows_publication` 查看策略决定。无法读取的基线、缺失完整身份证据、无效候选包或预算超限会抛出 CompareError；`.report` 包含失败前完成的观察结果。

默认策略阻止 High 和 Critical 发现项。审阅特定风险类别后，可显式接受：

```python
policy = UpdatePolicy().allow('RISK.NOTE_REMOVED')
comparison = project.compare(CompareOptions.against('course-v1.apkg').update_policy(policy))
output = project.build(BuildOptions.to('course-v2.apkg')
    .update_from('course-v1.apkg').update_policy(policy))
```

接受范围是指定类别中的全部发现项。发现项保留原风险与证据。未匹配的放行产生警告；警告不会把成功发布变为失败。未知码、缺失证据和其他硬错误不能被接受。不要自动接受比较返回的每个类别。

`update_from` 要求上一次原始分发包及其完整内嵌证据。Anki 重新导出不保留该证据。APKG 中省略笔记或卡片，不会自动从学习者的集合中删除它们。请结合目标 Anki 导入设置检查结构变化。

## 预算与产物所有权

```python
from ankiforge import InspectLimits

limits = InspectLimits(max_archive_bytes=3 << 30)
options = BuildOptions.to('course-v2.apkg').update_from('course-v1.apkg').inspect_limits(limits)
```

预算分别应用于每个被检查的基线和候选包。媒体导入有自己的、更早执行的 MediaLimits 预算。预算失败在 `details['limit_exceeded']` 下保留资源、限制和观察值；有意提高相关限制后再重试。模板包可用 `NoteType.from_bundle(path, limits=MediaLimits(...))` 调整每个资源的读取预算；manifest 与模板文本仍保留独立固定限制。必需检查不能禁用。

不需要持久路径时使用 `BuildOptions.temporary()`。消费文件时保留输出或产物句柄；`artifact.persist_to(path)` 创建持久副本。持久化失败时，PersistError 详情记录目标发布阶段，原句柄仍可用。

快照和错误详情中的路径，在有效 Unicode 时为字符串。Python 以 `surrogateescape` 表示的 Unix 字节名称，在 JSON 中为 `{"encoding": "unix_bytes", "bytes": [...]}`；可用 `Path(os.fsdecode(bytes(value["bytes"])))` 恢复。包含未配对 UTF-16 代理项的 Windows 名称使用 `{"encoding": "windows_wide", "units": [...]}`，每个数组元素保存原始 16 位代码单元。编码路径描述其原平台。运行时 `artifact.path` 仍是 `pathlib.Path`，`BuildOptions.to`、`update_from` 和 `CompareOptions.against` 通过绑定保留原生路径值。

缺失或不匹配的原生扩展会抛出带 `BINDING.EXTENSION_UNAVAILABLE` 或 `BINDING.VERSION_MISMATCH` 的导入错误。安装匹配 wheel，或用 maturin 重建源码目录。同一 Project 上的并发操作被拒绝；并发工作应使用独立项目。
