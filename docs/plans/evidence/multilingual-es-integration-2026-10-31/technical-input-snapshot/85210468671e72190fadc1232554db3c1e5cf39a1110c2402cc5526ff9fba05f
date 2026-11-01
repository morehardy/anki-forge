# 排查问题

使用失败操作的结构化 `kind` 和 `code`，结合来源链与观察结果进行排查。显示文本面向人类，可能变化。操作失败与比较已完成但策略阻止发布，是不同的结果。

## 安装

| 现象 | 检查 |
| --- | --- |
| Rust 拒绝工具链 | 运行 `rustc --version`；使用 Rust 1.92+，并以 `cargo tree -i ankiforge` 检查安装版本 |
| Node 无法加载原生包 | 运行 `node -p "process.platform + '/' + process.arch"` 和 `npm ls ankiforge`；用 `npm install --include=optional ankiforge@0.3.1` 重装 |
| Node 报告原生层与包装层不匹配 | 使用匹配版本；从相同 registry 版本重装，或从同一源码目录重建两者 |
| Python 无法加载扩展 | 运行 `python -m pip show ankiforge` 和 `python -m pip debug --verbose`；检查解释器以及 wheel 的 OS/CPU 标签 |
| Python 选择源码发行包 | 选择匹配 wheel/解释器，或按[源码构建](../source-builds.md)配置编译器 |

参见[已验证环境](../compatibility.md)、[Node 安装](../node/quick-start.md)和 [Python 安装](../python/quick-start.md)。

## 模型与笔记

| 现象 | 修正 |
| --- | --- |
| 无效或空键 | 显式提供稳定键；显示名称另行设置 |
| `NOTE.KEY_DUPLICATE` | 每条逻辑源记录使用唯一键 |
| `NOTE.FIELD_UNKNOWN` | 按字段键赋值，不使用显示名称 |
| `NOTE.FIELD_REQUIRED` | 添加前填写声明了 `required()` 的字段 |
| `NOTE.MODEL_CONFLICT` | 同一模型键复用相同的已完成模型 |
| 模板引用错误 | 使用已声明键；检查原始源码字节范围 |
| HTML 显示为原文 | 仅对有意加入的标记使用 `Content::html` |
| 缺少 Cloze 卡片 | 使用 `{{c1::answer}}` 语法，支持编号 1–500 |

添加失败是原子的。修正笔记或模型后重试，不需要撤销部分模型或媒体注册。模型构建器必须先完成 `.build()`，才能创建自定义笔记。

## 媒体与模板

| 现象 | 修正 |
| --- | --- |
| 文件导入失败 | 检查调用 `Media::file` 时的源文件；之后删除源文件是支持的 |
| MIME 与内容不匹配 | 向字节构造函数传入实际 MIME 类型 |
| 导出名称冲突 | 选择唯一可移植名称；大小写与 Unicode 规范化变体也冲突 |
| 缺失原始 HTML/CSS/脚本资源 | 使用模型 `asset` 或项目 `add_asset` 声明 |
| 媒体导入超限 | 导入前显式提高 `MediaLimits.max_bytes` |
| 构建检查超限 | 为本次操作调整相关 `InspectLimits` 计数器 |
| 模板包加载失败 | 使用 bundle-v2、显式键、按键模板以及包内路径 |
| IO 矩形失败 | 检查解码后显示尺寸、有限坐标和唯一遮罩键 |

所有笔记类型的文本默认行为一致。媒体节点收集依赖；手写文件名不会导入其字节。即使包有效，仍需对计划使用的编解码器和字体进行客户端播放检查。

## 更新与输出

基线应为含完整身份证据的原始分发 APKG。Anki 重新导出的包不保留该证据。更新策略不能放行命名空间不匹配或证据损坏。

完成的比较可以有 `allows_publication == false`。选择显式风险类别放行前，检查原始发现项和证据。不要全部放行。若希望得到相同决策，必须向实际更新构建传入相同策略。在仅新建请求上设置策略会报错。

临时路径消失时，应将输出或产物所有者保留到全部读取者结束。报告和 JSON 快照不是所有者。持久化失败时，检查错误的发布阶段和持久性：目标已发布后也可能返回错误。参见[输出保证](../build-guarantees.md)。
