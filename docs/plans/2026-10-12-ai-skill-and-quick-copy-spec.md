# Anki Forge AI Skill 与快速复制入口设计

状态：设计稿

日期：2026-10-12

基线：`main` / `origin/main`，提交 `deee0f7b24fcbf6f08682f24fe64ab8268db44a7`

目标版本：公开 Rust、Node / TypeScript、Python 包 `0.3.0`

## 1. 背景与问题

Anki Forge 已有完整的 Rust、Node / TypeScript 和 Python 入门程序、媒体、模板、图片遮挡和更新文档。用户仍然需要自己判断：

- 应该选择哪种语言和安装命令；
- 哪些 API 属于公开的 `0.3.0` 包；
- 如何向 AI 描述稳定 note key、持久 APKG、媒体和更新基线；
- AI 生成代码后如何验证结果。

官网已经存在 `InstallCommand.astro` 和 `data-copy-block` 剪贴板逻辑，但安装组件当前没有被首页使用。README 的代码块可以被 GitHub 复制，但没有面向 AI 的任务上下文。

本方案增加两层入口：

1. **Anki Forge Skill**：给支持 Skill 的编码代理提供稳定的触发规则、决策流程、参考文档和验证脚本。
2. **AI 快速复制**：给未安装 Skill 的 ChatGPT、Cursor、Claude 或普通聊天窗口提供一段可直接粘贴的最小提示。

Skill 是可持续使用的知识入口，复制提示是零安装的启动入口。二者使用同一份版本、安装命令、文档链接和任务术语。

## 2. 目标与边界

### 2.1 目标

- AI 能从自然语言需求生成第一个可运行的 Anki Forge 程序。
- AI 能根据语言选择正确的公开包和 `0.3.0` API。
- AI 默认生成持久化 `.apkg`，使用稳定 namespace 和 note key。
- AI 知道何时查阅 Basic、Cloze、媒体、自定义 note type、Image Occlusion 和更新文档。
- AI 在生成后提供安装、运行、预期产物和验证步骤。
- 用户可以从官网一键复制 prompt，或从 README 使用 GitHub 的代码块复制功能。
- Skill、README、官网和公开安装文档不会出现版本或命令分叉。

### 2.2 不在本次范围

- 不调用任何 AI API，不在官网保存用户输入。
- 不为 Anki Forge 增加新的运行时 API。
- 不把整个 API reference 复制进 Skill。
- 不让 Skill 默认执行发布、上传、删除牌组或修改用户 Anki 数据库的操作。
- 不承诺所有 AI 工具自动识别同一个 Skill 目录；各工具适配说明单独记录。
- 不替换现有的公开安装文档和可运行示例。

## 3. 用户体验

### 3.1 已安装 Skill 的编码代理

用户在项目中提出：

> 使用 Anki Forge 把我的单词表生成一个带音频的西班牙语牌组。

代理应：

1. 根据项目文件判断 Rust、Node / TypeScript 或 Python；无法判断时询问一次。
2. 检查项目是否已有 `ankiforge` 依赖和版本。
3. 默认使用公开 `0.3.0` API；源码专属功能必须明确说明适用范围。
4. 读取对应语言的快速开始和媒体参考文档。
5. 先给出将创建或修改的文件和运行命令，再写入代码。
6. 运行语言对应的最小检查；生成 `.apkg` 后说明输出路径和内容。
7. 对稳定 namespace、note key、媒体文件名和后续更新提出具体建议。

### 3.2 官网快速复制

在首页“Choose your language”区域后增加 `Use with AI` 区块：

```text
Use Anki Forge with AI
Choose a language, describe your task, then paste the generated brief into your AI coding assistant.

[ Rust ] [ Node / TypeScript ] [ Python ]

What should the deck do?
┌──────────────────────────────────────────────┐
│ Create a Spanish vocabulary deck with audio. │
└──────────────────────────────────────────────┘

[Copy prompt for AI]  [Copy install command]
```

交互要求：

- 默认选择 Rust，与现有首页第一个语言入口一致。
- 语言 tab 使用 `role="tablist"`、`aria-selected` 和键盘左右键切换。
- 需求输入是可选的；为空时复制包含 `<describe your task>` 的模板。
- `Copy prompt for AI` 复制当前语言、当前版本和用户需求组合出的纯文本。
- `Copy install command` 只复制当前语言安装命令，不混入解释文字。
- 成功状态使用 `aria-live="polite"`，例如 `Prompt copied. Paste it into your AI coding assistant.`。
- Clipboard API 失败时选择 `<pre>` 内容，保持现有降级行为。
- 页面没有 JavaScript 时，仍然显示完整 prompt 和安装命令。
- 不把用户输入写入 localStorage、URL、日志或分析服务。

### 3.3 README 快速复制

在语言安装表之后、详细 Quick start 之前加入 `Use with an AI coding assistant`。README 使用静态模板：

```markdown
## Use with an AI coding assistant

Copy this brief into ChatGPT, Cursor, Claude Code, or another coding assistant.
Replace the language and request with your own details.

```text
[generated AI prompt]
```

[Open the interactive prompt builder](https://ankiforge.dev/#ai-quickstart)
```

中文 README 使用对应的中文说明和中文 prompt。README 不放自定义 JavaScript 按钮；GitHub 的 fenced code block 提供复制能力，官网提供交互式复制。

## 4. Skill 包设计

### 4.1 建议目录

仓库内维护平台中立的源目录：

```text
skills/anki-forge/
├── SKILL.md
├── references/
│   ├── rust.md
│   ├── node.md
│   ├── python.md
│   ├── workflows.md
│   └── troubleshooting.md
└── scripts/
    └── verify_quickstart.mjs
```

`SKILL.md` 保持在 100 行以内，负责触发条件和路由；参考文件按任务加载。目录不绑定某一个代理产品。README 说明如何将它映射到具体工具支持的 Skill 目录。

### 4.2 Skill 描述

Skill 的 front matter 使用明确的第三人称触发描述：

```yaml
---
name: anki-forge
description: Build and maintain Anki decks with the public Anki Forge 0.3.0 Rust, Node/TypeScript, or Python APIs. Use when a user mentions Anki Forge, ankiforge, APKG generation, Basic/Cloze cards, media packaging, custom note types, Image Occlusion, stable note IDs, or deck updates.
---
```

描述必须包含项目名、包名、语言和用户任务触发词。不要在描述中写会快速过期的 API 细节。

### 4.3 `SKILL.md` 草稿

```markdown
# Anki Forge

Use the public Anki Forge 0.3.0 package to build and maintain Anki decks from Rust,
Node/TypeScript, or Python applications.

## Quick start

1. Identify the application language and inspect its manifest before editing.
2. Use the published package and the matching quickstart reference:
   - Rust: `references/rust.md`
   - Node/TypeScript: `references/node.md`
   - Python: `references/python.md`
3. Start with a complete persistent `.apkg` example.
4. Use a stable project namespace and stable source-record keys.
5. Show exact setup, files, run command, expected output, and verification.

## Rules

- Use only public APIs documented for 0.3.0.
- Do not invent methods or silently use git/path dependencies, internal tools,
  source-only APIs, or unreleased package features.
- Keep the original distributed APKG when preparing an update; do not use an
  Anki re-export as the comparison baseline.
- Treat media filenames, MIME declarations, template assets, namespace values,
  and note keys as user-visible data that must remain stable.
- Ask focused questions only when the language, input shape, output behavior, or
  stable identity cannot be inferred from the project.
- Do not publish, delete, upload, or modify an Anki collection unless the user
  explicitly requests that action.

## Task routing

- Basic or Cloze cards: read `references/workflows.md`.
- Images, audio, video, fonts, or batch media: read `references/workflows.md`
  and the language reference.
- Custom fields, templates, CSS, or reusable assets: read the template section
  before writing code.
- Image Occlusion: read the Image Occlusion section and state its input shape.
- A revised deck or stable note IDs: read the update workflow before changing keys.
- Installation or native loading failures: read `references/troubleshooting.md`.

## Verification

Run the smallest relevant formatter, type check, build, or example command.
Inspect the resulting APKG when the repository provides an inspection command.
Report the output path, note/card/media counts when available, and any client
limitations that the documentation states.
```

### 4.4 参考文件职责

`references/rust.md`、`node.md` 和 `python.md` 各包含：

- 运行时要求；
- 精确的 `0.3.0` 安装命令；
- 最小完整程序；
- 语言特有的 artifact 生命周期规则；
- 语言对应的验证命令；
- 继续阅读链接。

`references/workflows.md` 按用户任务组织：

- Basic：`Project`、`Note::basic` / `Note.basic`、稳定 key、持久 APKG；
- Cloze：删除标记、卡片数量和 key 变化风险；
- Media：拥有快照、MIME、导出文件名、显式模板资产和批量导入；
- Custom note types：字段、模板、CSS、资产和模板包；
- Image Occlusion：输入图像、遮挡区域和可验证输出；
- Update：原始分发 APKG、`update_from` / `compare_to` 和风险报告。

`references/troubleshooting.md` 把常见错误映射到可执行检查，不要求 AI 记忆内部 protocol 数字。所有内容应链接到现有 `docs/` 源文档，而不是复制整篇 API reference。

### 4.5 Skill 验证脚本

`skills/anki-forge/scripts/verify_quickstart.mjs` 是确定性检查器，检查：

- 三份参考文档都含 `0.3.0` 和正确的公开安装命令；
- 示例导入符号与当前语言指南一致；
- 不出现 `internal-tools`、`path dependency`、`maturin develop` 作为普通用户安装方式；
- 关键文档链接存在；
- prompt 中的版本、安装命令和官网 URL 与共享数据一致。

脚本只验证文本和链接，不代替三语言的实际消费者测试。

## 5 共享数据与生成

### 5.1 权威数据文件

新增 `docs/ai-quickstart.json`，作为 prompt、Skill 元数据和官网组件的数据源：

```json
{
  "schemaVersion": 1,
  "publicVersion": "0.3.0",
  "sitePromptUrl": "https://ankiforge.dev/#ai-quickstart",
  "languages": {
    "rust": {
      "label": "Rust",
      "runtime": "Rust 1.92+",
      "install": "cargo add ankiforge@0.3.0",
      "quickstart": "https://ankiforge.dev/docs/quickstart/"
    },
    "node": {
      "label": "Node / TypeScript",
      "runtime": "Node.js 22.13+",
      "install": "npm install --include=optional ankiforge@0.3.0",
      "quickstart": "https://ankiforge.dev/docs/node-quickstart/"
    },
    "python": {
      "label": "Python",
      "runtime": "CPython 3.11+",
      "install": "python -m pip install ankiforge==0.3.0",
      "quickstart": "https://ankiforge.dev/docs/python-quickstart/"
    }
  },
  "prompts": {
    "en": "...",
    "zh-CN": "..."
  }
}
```

公共版本仍然需要与现有公开安装文档和 registry 检查一致。该 JSON 不取代 package manifest 的版本治理；它描述面向用户的公开版本，符合仓库现有“公开包版本独立维护”的规则。

### 5.2 生成流程

新增 `website/scripts/ai-quickstart.mjs`，提供：

```sh
npm run check:ai
npm run sync:ai
```

`sync:ai`：

- 从 `docs/ai-quickstart.json` 渲染 README 英文和中文的带标记区块；
- 把 JSON 写入被忽略的 `website/src/generated/ai-quickstart.json`；
- 生成官网组件需要的语言数据和 prompt 模板。

`check:ai`：

- 检查两个 README 的标记区块与 JSON 一致；
- 检查现有 `docs/installation.md`、`docs/node/quick-start.md`、`docs/python/quick-start.md` 的版本和命令一致；
- 检查三个 prompt 都包含公共包、稳定 key、持久 APKG、验证步骤和对应文档链接；
- 检查 prompt 不包含用户输入、密钥、隐藏 HTML 或网站按钮文字。

`prepare-site.mjs` 在构建前只执行 check 和生成，不修改受 Git 跟踪的 README。README 更新通过显式 `npm run sync:ai` 完成并审核 diff。

## 6 Prompt 内容契约

### 6.1 英文模板

官网和英文 README 按所选语言渲染 `{language}`、`{runtime}`、`{install}`、`{quickstart}` 和 `{request}`：

```text
You are helping me use Anki Forge {version} in a {language} application.

Use the published `ankiforge` package and only the public API documented for
this version. Do not use git/path dependencies, internal tools, source-only
APIs, unreleased methods, or invented methods.

My goal:
{request}

Please:
1. Inspect the existing project before changing files.
2. Give the exact setup and installation command: {install}
3. Create the smallest complete runnable program and name every file to add or edit.
4. Produce a persistent `.apkg` file and use stable namespace and note keys.
5. Include the run command, expected output, and a verification step.
6. If media, custom templates, Image Occlusion, or updates are needed, use the
   documented workflow instead of approximating the API.
7. Ask only the questions required to resolve an unknown language, input shape,
   output path, or identity rule.

Quickstart reference: {quickstart}
```

### 6.2 中文模板

```text
请帮助我在 {language} 项目中使用 Anki Forge {version}。

只使用该版本公开文档中的 `ankiforge` 包和公开 API。不要使用 git/path
依赖、internal tools、仅源码可用的 API、未发布方法或猜测出来的方法。

我的目标：
{request}

请按以下要求完成：
1. 先检查现有项目，再决定需要新增或修改哪些文件。
2. 给出准确的安装命令：{install}
3. 生成最小但完整、可以直接运行的程序。
4. 输出持久化的 `.apkg`，并使用稳定的 namespace 和 note key。
5. 给出运行命令、预期输出和验证步骤。
6. 如果涉及媒体、自定义模板、图片遮挡或牌组更新，请遵循文档中的流程，
   不要自行猜测 API。
7. 只有在语言、输入格式、输出路径或身份规则无法判断时才提问。

快速开始文档：{quickstart}
```

### 6.3 Prompt 安全和长度

- 默认 prompt 控制在 500 个词或约 700 个中文字符以内。
- 只包含公开文档 URL，不要求 AI 上传用户文件或密钥。
- 用户需求作为纯文本插入；复制前对 HTML 不做解释执行。
- 网站只在用户点击按钮后读取和写入剪贴板。
- 复制内容不包含“Copied”状态文字、按钮标签或代码高亮标记。

## 7 官网实现方案

### 7.1 组件

新增 `website/src/components/AiQuickstart.astro`：

- 从 `src/generated/ai-quickstart.json` 获取语言、版本、安装命令和 prompt 模板；
- 输出 tab、需求 `textarea`、prompt 预览、复制 prompt 按钮和复制安装命令按钮；
- 使用稳定的 `data-ai-quickstart`、`data-ai-language`、`data-ai-request`、`data-ai-prompt` 属性；
- 使用 `Icon` 的 `copy` 和 `check` 图标；
- 设置 `id="ai-quickstart"`，方便 README 锚点跳转。

### 7.2 客户端逻辑

扩展 `website/src/scripts/site.ts` 的现有复制逻辑：

- 将通用成功消息从固定的 `Command copied.` 改成 `data-copy-success`；
- 监听语言 tab，更新 prompt 预览和安装命令；
- 监听 textarea 的 `input`，只更新内存中的 prompt；
- 优先 `navigator.clipboard.writeText`；失败时选择内容节点；
- 复制成功后暂时把按钮文字改成 `Copied`，随后恢复；
- 在 `pagehide` 时移除监听器或清理引用。

现有 `InstallCommand.astro` 可以迁移到一个小型 `CopyBlock.astro`，但只在安装命令和 AI prompt 都复用时抽象。不要引入第三方 clipboard 包。

### 7.3 样式和响应式

沿用现有 token、`source-label`、`install-command` 和 `text-button` 风格。新增样式只覆盖：

- tab 状态；
- prompt 预览的滚动和换行；
- textarea；
- 双按钮在窄屏下的堆叠；
- focus-visible 和 reduced-motion。

360px 宽度下不得产生页面横向滚动；代码和 prompt 自己滚动。无脚本时预览文本仍然完整可读。

## 8 README 和文档集成

修改：

- `README.md`：英文 prompt 区块、官网 builder 链接、Skill 源码链接；
- `README.zh-CN.md`：中文 prompt 区块和中文说明；
- `docs/documentation.md`：记录 AI prompt 的生成和校验规则；
- `website/package.json`：增加 `check:ai`、`sync:ai`；
- `.github/workflows/website.yml`：将 AI 数据、Skill、脚本和 README 纳入触发路径，并在构建 job 中执行 `npm run check:ai`。

README 的 Skill 入口文字：

```markdown
### For AI coding assistants

The repository includes an Anki Forge Skill with language-specific setup,
public API rules, task workflows, and verification guidance:

[Open the Anki Forge Skill](skills/anki-forge/SKILL.md)
```

官网 Skill 入口指向 GitHub 的 `skills/anki-forge/` 目录，并同时提供复制 prompt。链接文字需要说明这是给 coding assistant 使用的，不把它包装成普通用户必须安装的依赖。

## 9 测试与验收

### 9.1 自动检查

在 `website` 执行：

```sh
npm run check:ai
npm test
npm run check:docs
npm run check:examples
npm run check
npm run build
npm run check:site
```

Skill 脚本还需要通过 Node 直接运行：

```sh
node skills/anki-forge/scripts/verify_quickstart.mjs
```

### 9.2 内容验收

- 三种语言的安装命令都指向公开 `0.3.0` 包。
- 三种语言的 prompt 都包含公开 API 限制、持久 `.apkg`、稳定 identity、运行命令和验证步骤。
- Rust 示例使用 `BuildOptions::to`、`Project` 和 `Note::basic`；Node 使用 `Project`、`Note.basic` 和 artifact close；Python 使用对应公开 API。
- 任何内部协议、`internal-tools`、源码 checkout 或本地 tarball 都不出现在普通用户 prompt 中。
- README 标记区块、官网预览和 JSON 完全一致。

### 9.3 交互验收

- 官网点击复制后，剪贴板文本与预览完全一致。
- Clipboard API 被拒绝时，用户仍能选中文本复制。
- 键盘可以切换语言和触发复制按钮。
- 屏幕阅读器能读到当前语言、复制结果和错误状态。
- 无 JavaScript 时，安装命令、prompt 和 Skill 链接仍可使用。
- 明暗主题、窄屏和 reduced-motion 设置下布局可用。

### 9.4 AI 任务验收

在独立临时目录分别验证以下四类 prompt / Skill 任务：

1. 生成一个 Basic 西班牙语牌组，得到 `spanish.apkg`。
2. 生成包含图片和音频的牌组，并保留显式媒体文件名。
3. 定义一个自定义 note type 和模板。
4. 修改已有 note 的答案，使用上一份原始 APKG 作为更新基线。

每个任务都检查：安装命令、源文件、运行结果、APKG 内容和 AI 是否避免调用未发布 API。三语言消费者的完整验证继续使用现有网站 CI 和 SDK CI，不用 AI 输出替代现有测试。

## 10 实施顺序

### P0：内容和契约

1. 新增 `docs/ai-quickstart.json`。
2. 新增 `skills/anki-forge/SKILL.md` 和参考文档草稿。
3. 新增校验脚本，先让版本、命令、链接和 prompt 契约可检查。
4. 在 README 加入生成标记区块。

完成标准：`check:ai` 能发现版本、命令或 prompt 标记不一致。

### P1：官网入口

1. 新增 `AiQuickstart.astro`。
2. 将现有安装复制组件接入通用复制逻辑。
3. 在首页语言安装区之后渲染 AI 区块。
4. 增加键盘、降级复制、无脚本和响应式验证。

完成标准：官网可以为三种语言复制 prompt 和安装命令，且与 JSON 一致。

### P2：Skill 深化和工具适配

1. 用现有 `docs/` 内容补齐 Rust、Node、Python 和 workflow references。
2. 验证 Skill 中的示例与公开包消费者测试一致。
3. 增加各支持 AI 工具的加载说明；每个说明只描述已验证的目录和命令。
4. 在 README 和官网增加 Skill 源码入口。

完成标准：代理能完成四类任务验收，并能在不支持 Skill 的工具中使用复制 prompt。

## 11 风险和决策记录

### Skill 识别差异

不同代理可能使用不同的 Skill 目录、规则文件或导入方式。仓库保留平台中立的 `skills/anki-forge/` 源目录；平台适配文件只在确认工具规范后增加，不改变主 Skill 内容。

### 版本更新

`0.3.0` 是当前公开包版本。未来发布新版本时，先更新 `docs/ai-quickstart.json` 和对应的公开安装文档，再同步 Skill reference、README 和官网。CI 必须在其中一个位置落后时失败。

### AI 生成错误

Prompt 和 Skill 都要求使用公开 API、先检查项目、生成验证命令和报告限制。AI 输出仍需经过现有编译、消费者和 APKG 检查；官网功能不把复制成功表示成代码正确。

### 维护成本

Skill 主文件只包含路由规则，详细知识链接到现有文档。Prompt 只包含最小任务契约，避免复制整篇 README，降低版本变动时的同步成本。

## 12 完成定义

本 spec 实施完成后：

- 用户从 README 可以复制一个可用的 AI 上手 brief；
- 用户从官网可以选择语言、输入需求并一键复制 prompt；
- 支持 Skill 的代理能按公开 `0.3.0` API 执行常见 Anki Forge 任务；
- 不支持 Skill 的代理仍可通过 prompt 工作；
- Skill、README、官网和文档安装命令由同一数据契约校验；
- 所有现有网站、SDK 和文档检查继续通过。
