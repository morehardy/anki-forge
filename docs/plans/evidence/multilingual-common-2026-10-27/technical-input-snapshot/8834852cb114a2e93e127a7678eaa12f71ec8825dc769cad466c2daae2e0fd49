# 用 Python 生成 Anki APKG

创建并激活虚拟环境，然后安装[公开 PyPI 包](https://pypi.org/project/ankiforge/)：

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

Windows 使用 `.venv\Scripts\Activate.ps1` 激活。使用普通 CPython 3.11 或更高版本及匹配的平台 wheel；已有发布检查覆盖 CPython 3.11–3.14，参见[已验证环境](../compatibility.md)。

## 导出牌组

将以下程序保存为 `main.py`：

<!-- source: bindings/python/examples/quickstart.py -->
```python
from ankiforge import Project, Note, BuildOptions

project = Project('spanish', default_deck='Spanish')
project.add('es:hola', Note.basic('hola', 'hello'))
output = project.build(BuildOptions.to('spanish.apkg'))
print(output.artifact.path)
```
<!-- /source -->

```sh
python main.py
```

在 Anki 中打开持久保存的 `spanish.apkg`，其中 **Spanish** 牌组包含 **hola → hello**。后续编辑时保持命名空间 `spanish` 和笔记键 `es:hola` 不变。导出不需要安装 Anki。

## 继续学习

完整媒体与更新任务见 [JSON 到 Anki](../json-to-anki.md#python)。

- [图片与音频](../media.md)及 [Python 媒体 API](../python/api.md#media-and-occlusion)
- [自定义笔记类型](../custom-notetypes.md)和[模板包](../template-bundles.md)
- [图片遮挡](../image-occlusion.md)
- [比较与更新](../updates.md)
- [Python API](../python/api.md)及[诊断](../python/diagnostics.md)
- [从 genanki 迁移](../python/genanki-migration.md)

原生源码编译见[源码构建](../source-builds.md)。
