# 用 AI 助手生成 Anki 卡片

让 AI 助手根据笔记起草问题和答案，对照原文审核后，再用 Anki Forge 导出 Anki
`.apkg` 牌组。在这个流程中，Anki Forge 提供开源卡片生成库：AI 助手负责起草
内容或编写脚本，本地 Python、Node.js 或 Rust 程序负责生成卡片。

你可以先[复制编程助手任务说明](/#ai-quickstart)并选择开发语言，也可以按下面的
CSV 流程操作。网站上的任务说明生成器在本地拼接文本，不会调用 AI 模型或上传笔记。

<!-- section: choose-input -->
## 从可以核对的原始材料开始

选择主题明确的短材料，例如自己的课程笔记或词汇表。如果来源是 PDF、扫描件
或课件，先用其他工具提取并核对文字。Anki Forge 通过 API 接收你提供的内容，
不负责 PDF 文本提取或 OCR。

每个问题尽量只考查一个知识点，并保留必要的上下文，让卡片脱离原文后也能独立理解。

<!-- section: draft-cards -->
## 让 AI 起草 CSV 卡片

将下面的示例粘贴给助手。制作自己的牌组时，替换原始材料，并相应调整笔记数量与新笔记 ID：

```text
Turn the source below into exactly three Basic Anki notes.
Use only facts stated in the source. Do not add facts from memory.
Each question must test one fact and be understandable on its own.
Return only UTF-8 CSV with the header id,front,back, with no Markdown fence.
Use the IDs es:hola, es:gracias and es:adios for these three vocabulary items.
Quote any CSV field containing a comma, quote or newline correctly.

Source:
In Spanish, hola means hello, gracias means thank you, and adiós means goodbye.
```

这段提示词要求仅使用原文事实，生成 3 条 Basic 笔记，保留指定 ID，并按 CSV 规则输出。
输出内容应与以下示例等价：

```csv
id,front,back
es:hola,hola,hello
es:gracias,gracias,thank you
es:adios,adiós,goodbye
```

助手可能采用不同的问题表述。请逐行对照原文，修正含糊或缺少依据的答案，再保存为
`cards.csv`。新笔记使用唯一 ID，之后编辑时保持不变；不要让助手为已有牌组重新生成 ID。

<!-- section: export-apkg -->
## 将审核后的卡片导出为 APKG

[用 Python 将 CSV 转为 Anki](csv-to-anki.md)提供完整程序、安装命令和输入示例。
脚本检查 CSV 格式和 ID 唯一性，生成持久保存的包。示例包含 3 张 Basic 卡片，不含媒体。

如果已有 JSON 数据，或需要图片与音频，可使用 [JSON 转 Anki 教程](json-to-anki.md)，
其中提供完整的 Python、Node / TypeScript 和 Rust 程序。应用程序先解析 CSV 或 JSON，
再将笔记传给 Anki Forge。

需要助手编写自定义程序时，复制[编程助手任务说明](/#ai-quickstart)，补充输入字段、
卡片类型和输出文件名。支持加载 Skill 的助手还可以使用
[Anki Forge Skill](../ai-assistants.md)，读取公开 API 参考与任务流程。

<!-- section: check-result -->
## 学习前检查卡片

在 Anki 中打开生成的 APKG，检查卡片数量、问题、答案和排版。CSV 示例应在 Spanish
牌组中生成 3 条笔记和 3 张卡片。成功导出说明包已生成，并不代表 AI 的答案已经核实，
或适合作为学习材料。

填空题可以使用 [Cloze 卡片](cards.md)。图表标注可以使用
[图片遮挡](image-occlusion.md)，由你提供图片和遮挡坐标；它不会自动识别图中的标签。

保留原始分发 APKG，后续版本按[更新流程](updates.md)操作：保持命名空间和笔记 ID，
检查比较结果，并考虑 Anki 导入设置与较新的本地编辑。
