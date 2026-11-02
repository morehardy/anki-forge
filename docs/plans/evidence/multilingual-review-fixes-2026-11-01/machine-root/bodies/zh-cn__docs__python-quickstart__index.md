# 用 Python 生成 Anki 卡片

Canonical page: https://ankiforge.dev/zh-cn/docs/python-quickstart/
Language: zh-CN
Description: 从 PyPI 安装 ankiforge，用简短的 Python 脚本生成 Anki 问答卡片，并导出可导入 Anki 的 APKG 牌组。

创建并激活虚拟环境，然后安装[公开 PyPI 包](https://pypi.org/project/ankiforge/)：

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install ankiforge==0.3.1
```

Windows 使用 `.venv\Scripts\Activate.ps1` 激活。使用普通 CPython 3.11 或更高版本及匹配的平台 wheel；已有发布检查覆盖 CPython 3.11–3.14，参见[已验证环境](https://ankiforge.dev/zh-cn/docs/compatibility/)。

## 导出牌组

将以下程序保存为 `main.py`：

```python
from ankiforge import Project, Note, BuildOptions

project = Project('spanish', default_deck='Spanish')
project.add('es:hola', Note.basic('hola', 'hello'))
output = project.build(BuildOptions.to('spanish.apkg'))
print(output.artifact.path)
```

```sh
python main.py
```

在 Anki 中打开持久保存的 `spanish.apkg`，其中 **Spanish** 牌组包含 **hola → hello**。后续编辑时保持命名空间 `spanish` 和笔记键 `es:hola` 不变。导出不需要安装 Anki。

## 继续学习

已有词汇表时，参考 [CSV 转 Anki](https://ankiforge.dev/zh-cn/docs/csv-to-anki/)。需要根据原始笔记起草卡片时， 可从 [AI 辅助制卡](https://ankiforge.dev/zh-cn/docs/ai-anki-cards/)开始。

完整媒体与更新任务见 [JSON 到 Anki](https://ankiforge.dev/zh-cn/docs/json-to-anki/#python)。

-   [图片与音频](https://ankiforge.dev/zh-cn/docs/media/)及 [Python 媒体 API](https://ankiforge.dev/zh-cn/docs/python-api/#media-and-occlusion)
-   [自定义笔记类型](https://ankiforge.dev/zh-cn/docs/custom-notetypes/)和[模板包](https://ankiforge.dev/zh-cn/docs/templates/)
-   [图片遮挡](https://ankiforge.dev/zh-cn/docs/image-occlusion/)
-   [比较与更新](https://ankiforge.dev/zh-cn/docs/updates/)
-   [Python API](https://ankiforge.dev/zh-cn/docs/python-api/)及[诊断](https://ankiforge.dev/zh-cn/docs/python-diagnostics/)
-   [从 genanki 迁移](https://ankiforge.dev/zh-cn/docs/genanki-migration/)

原生源码编译见[源码构建](https://ankiforge.dev/zh-cn/docs/development/)。
