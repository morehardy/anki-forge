# 从 genanki 的概念迁移

还在选库？阅读 [Anki Forge 与 genanki](../anki-forge-vs-genanki.md)。

AnkiForge 使用稳定数据键、已校验的不可变模型、拥有所有权的媒体快照以及显式构建和更新操作。这是一套新的内容编写模型，不模拟 genanki 对象，也不模拟已移除的 AnkiForge Deck/lockfile 接口。

| genanki 概念 | AnkiForge 对应方式 |
| --- | --- |
| 带数字 ID 的 Model | `NoteType.builder(stable_key)...build()` |
| 字段名用作标识符 | 稳定 Field 键与独立显示名称 |
| 携带字段字符串的 Note | 持有模型的 `model.note().field(key, content)` |
| Deck 作为笔记容器 | 带默认牌组的 Project；可选笔记级 `.deck(name)` |
| Package 的媒体文件路径 | 拥有所有权的 `Media.file(path)` 值，经类型化内容自动收集 |
| 写入 Package | `project.build(BuildOptions.to(path))` |
| 从字段派生的默认 GUID，或自定义 GUID 规则 | 显式稳定命名空间和笔记键 |

```python
from ankiforge import Project, NoteType, Field, Template, BuildOptions

model = (NoteType.builder('vocabulary').name('Vocabulary')
    .field(Field('word', name='Word', required=True))
    .field(Field('meaning', name='Meaning'))
    .template(Template('recognition', '{{word}}', '{{FrontSide}}<hr>{{meaning}}'))
    .build())
project = Project('vocabulary-course', default_deck='Languages::Vocabulary')
project.add('dictionary:cell', model.note().field('word', 'cell').field('meaning', '细胞'))
project.build(BuildOptions.to('vocabulary.apkg'))
```

从持久业务标识符选择键。重命名显示名称或更改定义文本，不应改变这些键。除非导入器明确要求按内容寻址身份，否则不要散列当前字段内容。没有键的源数据，应在 SDK 外分配并持久保存键。

模板表达式使用 Field 键。显示名称可以独立翻译，库将键引用编译为 Anki 字段名。模型完成时就会拒绝缺失或冲突声明，之后才可添加笔记。添加笔记自动收集模型依赖，并拒绝同一模型键下的冲突定义。

字符串是纯文本；有意加入 HTML 时使用 `Content.html(...)`。通过 `media.image()`、`media.sound()` 和 `Content.sequence(...)` 组合内容。原路径被移除或编辑后，文件快照仍有效。原始 HTML、CSS 或脚本引用的依赖，必须在模型构建器上用 `.asset(media)` 或通过 `project.add_asset(media)` 显式声明。

后续发布创建具有相同命名空间和数据键的新 Project，并调用 `BuildOptions.to(next_path).update_from(previous_original_apkg)`。原始 APKG 提供完整身份证据。此流程不会从任意旧包恢复身份，也不承诺自动转换已有 genanki 数字 ID/GUID。除非经过单独验证的数据迁移建立了对应关系，否则应把新模型的首次分发视为明确的身份边界。

参见[入门](../python/quick-start.md)、[API 参考](../python/api.md)和[更新策略](../python/diagnostics.md)。
