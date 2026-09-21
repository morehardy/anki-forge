# README 改版方案

日期：2026-09-21。依据当前根 README、Rust / Node / Python 指南与 2026-09-21 基准报告制定。本文件是设计方案。

## 推荐方向

把 README 的阅读路径改为：**看见成果 → 看见选择理由 → 跑出第一个牌组 → 了解进阶能力**。

面向把自己的数据转换成 Anki 牌组的开发者，默认保留英文主 README、Rust 主示例，同时给 Node.js / TypeScript 与 Python 清晰入口。核心定位为：

> 用代码生成丰富的 Anki 牌组，并为后续更新建立可检查的构建流程。

以卡片成果建立第一印象，以实测速度吸引继续阅读，以更新能力形成差异。Rust 是实现与性能依据；对读者的直接价值是生成牌组、处理媒体、持续维护内容。

## 当前页面的具体问题

| 当前呈现 | 阅读成本 | 改法 |
| --- | --- | --- |
| 开头为介绍段落和四条功能列表 | 能知道功能，但看不到成果 | 首屏加入真实卡片效果与对应代码 |
| 很快进入 clone、编译、path dependency | 尚未产生兴趣就先看到环境成本 | 先展示短代码和结果，再给可执行步骤 |
| Basic 与 Project 两段完整示例连续展开 | 内容重复，更新价值仍需读技术说明 | Basic 留在快速开始；Project 用来展示一个具体的更新场景 |
| 主要视觉是底部折叠的性能热力图 | 最有说服力的数据难以发现 | 首屏给一个数据结论；正文放简洁比较图 |
| stable identities、baseline、lockfile 并列 | 读者必须自己推导它们的用途 | 先讲“修正已分发牌组中的内容”，再解释实现 |
| 版本和限制占据较多连续正文 | 关键状态与内部细节混在一起 | 相关入口保留关键条件，详细说明链接到指南 |

## 首屏设计与文案

建议标题：

> **Turn your data into Anki decks.**

副标题：

> Create rich flashcards with Rust, Node.js, or Python. Package your media and rebuild with explicit update checks.

品牌区保留 `anki-forge`。只保留少量真实、有用的标识，例如 MIT 与 CI；当前不添加暗示已发布的包版本徽章。

首屏视觉为一张横向的“代码 → 卡片”示意图：左侧使用仓库真实的 Basic authoring 片段，右侧展示 `hola → hello` 的正反面，底部标明输出 `spanish.apkg`。最终素材使用可复现例子生成的卡片截图或实际模板渲染；方案中的示意效果明确标为 schematic，不冒充 Anki 截图。

图旁或图下保留可选择、可复制的文本介绍。完整可复制代码放在快速开始，避免同一段代码连续出现两次。

紧接着展示一个数据结论：

> **1,000 text notes: 53.9 ms vs 115.5 ms with genanki.**

同时就近注明：Rust Deck、Apple M1 Pro、2026-09-21 的源码快照、10 次计时的中位数；链接完整报告。首屏主张可以概括为“该场景导出耗时减少 53.3%”，不使用跨平台、全场景或多语言 SDK 的性能承诺。

提供三个清晰入口：`Quick start`、`Examples`、`Benchmarks`，链接到正文锚点或已有指南。

## 正文信息顺序

| 顺序 | 区块 | 读者得到什么 | 呈现方式 |
| --- | --- | --- | --- |
| 1 | Hero + code to card | 知道项目用途，并看到结果 | 标题、一句介绍、成果图、一个实测结论 |
| 2 | Why anki-forge | 理解三个主要选择理由 | Rich cards / Fast exports / Builds that support updates；各一句收益和证据入口 |
| 3 | Quick start | 从当前 checkout 跑出 `spanish.apkg` | 保留已有完整 Rust 例子和真实运行命令；说明产物及如何导入 |
| 4 | More than a text card | 看见 Cloze、自定义样式、媒体的效果 | 一张三列卡片成果图；各自链接可运行例子 |
| 5 | Build once. Keep improving. | 理解修改已分发牌组时为何需要此工具 | 修改前后内容示例，加首次/后续构建流程；链接更新指南 |
| 6 | Performance you can inspect | 查看速度与可信度 | 五个 1,000-note 场景的小型横条图；原始数据与方法折叠 |
| 7 | Choose your language | 根据使用语言继续接入 | Rust / Node / Python 三行入口表，标注实际安装状态 |
| 8 | Status and compatibility | 了解采用前的实质条件 | 简短状态、重要限制、贡献和许可证链接 |

快速开始处已经提供 Node / Python 的跳转链接，避免这两类读者必须读完 Rust 示例才找到入口。第 7 区块负责完整入口与状态，不重复三套长示例。

## 将功能写成可以感受到的价值

| 核心优势 | 建议英文表述 | 配套证据 |
| --- | --- | --- |
| 丰富卡片与媒体 | **Make the cards your content needs.** Basic, Cloze, custom templates, images, audio, and video. | 三种卡片效果与例子链接；注明自定义样式属于示例 |
| 导出速度 | **Spend less time exporting.** | 同一份基准报告中的实测时间 |
| 持续维护内容 | **Build once. Keep improving.** Compare against a previous release and check identity changes before publishing. | 修正答案的前后示例、baseline / lockfile 流程 |

校验与结构化诊断作为构建可靠性的支撑说明，放在更新流程中；不将四五个内部概念都作为首屏独立卖点。

更新例子可用同一个 `es:hola`，将答案从 `hello` 修订为 `hello; hi`。图示显示 `v1 + edited source → comparison / identity checks → v2`。强调 stable ID 加先前 APKG 或持续维护的 lockfile 提供修订依据。不能声称单独设置 stable ID 就保证导入更新，也不承诺覆盖 Anki 中更新的本地编辑或忽略导入设置。

## 性能展示方式

全部选用 [2026-09-21 的同一份报告](../../benchmarks/results/20260921-readme-genanki/report.md)，不混用其他日期的数据。

| 1,000 notes 场景 | Rust Deck | genanki | 耗时减少 |
| --- | ---: | ---: | ---: |
| Text only | 53.9 ms | 115.5 ms | 53.3% |
| Unique images | 238.6 ms | 365.5 ms | 34.7% |
| Unique audio | 175.5 ms | 298.8 ms | 41.3% |
| Mixed, unique media | 160.1 ms | 279.5 ms | 42.7% |
| Mixed, shared media | 69.1 ms | 124.0 ms | 44.3% |

正文图采用从零开始的统一毫秒刻度，每个场景配对显示两种实现，直接标明数值。保持场景顺序固定，避免只挑最佳结果。现有热力图与完整矩阵仍放在报告中，首页减少读图成本。

方法摘要就近说明：单机单次测量会话、包含进程启动、两者默认 APKG 格式不同、Node / Python SDK 未参与测量。内存结果留在同一入口可见的报告中，不宣称全面省内存。例如 1,000 张独立图片场景的 Rust RSS 为 40.25 MiB，genanki 为 35.77 MiB。

可加一行验证依据：该会话 840 次导出内容检查、40 次 Anki 导入/内容/代表性渲染检查通过。标注为这次基准的检查结果，不包装成 GUI 全功能认证。

## 视觉语言和 GitHub 落地

视觉方向为简洁的开发工具页面：正文保持 Markdown，品牌与示意素材使用少量铜橙色强调，性能比较用稳定的双色区分；增加段落留白，减少宽表格和连续长代码。

复杂排版仅存在于图片中，正文用 Markdown、常规链接和 `<details>`。预览中的设计切换属于方案比较工具，不作为 GitHub README 的交互功能。

建议产出三组素材：

1. `docs/assets/readme/code-to-card-light.svg` / `code-to-card-dark.svg`：主视觉，短代码、APKG 产物、卡片正反面。
2. `docs/assets/readme/card-showcase-light.png` / `card-showcase-dark.png`：Basic、Cloze、媒体自定义卡片的真实成果图。
3. `docs/assets/readme/export-times-light.svg` / `export-times-dark.svg`：由归档数据生成的五场景比较图，保留生成脚本。

长宽比和字号以 GitHub 正文实际宽度检查，手机下保证关键字可读。所有图片有 alt 文本，关键收益与数据也出现在可搜索的正文中。短动图可后续补充，第一版优先保证静态展示清晰。

## 实施顺序与验收

第一步：重写首屏与阅读顺序，沿用已验证的 Rust 最小示例，整理语言入口。第二步：制作代码到卡片主视觉、真实卡片成果和简洁基准图。第三步：补齐更新场景、采用状态和链接，检查 GitHub 实际显示。

验收以这些结果为准：

- 首屏能回答“是什么、能生成什么、为什么继续看”，并能找到快速开始。
- 从 Quick start 复制运行后能得到文中描述的 APKG；依赖和运行目录完整。
- 示例卡片与实际导出一致；示例主题不被误认为库的默认样式。
- 每项数字可追溯至同一基准快照，图形比例与表格一致。
- Node / Python 入口不暗示尚未核实的 registry 发布状态。
- Image Occlusion 若展示，只演示已有可用的 `hide-all-guess-one` 路径，并紧邻相关限制链接。
- 更新能力说明保留 baseline / lockfile 和 Anki 导入条件，避免无条件保证。
- 浅色、深色、移动宽度下可读；核心内容不依赖动画、悬停或展开才能理解。

参考：[当前 README](../../README.md)、[Rust 更新指南](../rust-guide.md#updating-distributed-decks)、[Node SDK 状态](../../bindings/node/README.md)、[Python 源码安装](../../bindings/python/README.md)、[最新基准证据](../../benchmarks/results/20260921-readme-genanki/report.md)。

## 实施记录

已按该顺序重写根 README。主视觉与卡片展示采用 Anki 核心渲染的内容，输出 PNG；性能图读取归档 CSV，输出 SVG。三组素材均提供深浅色及移动版，并附可直接导入的 `showcase.apkg`。

新增 `readme_showcase` 和 `readme_update` 两个可运行 Rust 示例，以及渲染、素材生成和复现说明。快速开始与展示牌组已通过 Anki 导入/渲染检查；连续导入更新示例的 v1、v2 后仍为一张卡片，答案已更新。已核验 58 个本地链接/素材及 Markdown 锚点，并检查桌面与 360px 宽度下的页面显示和图片选择。

素材的中性预览样式与浏览器音频控件在说明中明确标注，未声称为 Anki Desktop 界面截图。完整复现入口见 [素材说明](../assets/readme/README.md)。
