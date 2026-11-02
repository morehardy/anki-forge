# 用 Node.js 生成 Anki 卡片

Canonical page: https://ankiforge.dev/zh-cn/docs/node-quickstart/
Language: zh-CN
Description: 从 npm 安装 ankiforge，用 Node.js 或 TypeScript 生成 Anki 卡片，并导出可导入 Anki 的 APKG 牌组。

使用 Node.js 22.13 或更高版本。在新的应用目录中运行：

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.1
```

这会安装[公开 npm 包](https://www.npmjs.com/package/ankiforge)及匹配的原生可选依赖。平台覆盖范围见[已验证环境](https://ankiforge.dev/zh-cn/docs/compatibility/)。

## 导出牌组

将以下程序保存为 `main.mjs`：

```js
import { Project, Note, BuildOptions } from 'ankiforge';

const project = new Project('spanish').defaultDeck('Spanish');
project.add('es:hola', Note.basic('hola', 'hello'));
const output = await project.build(BuildOptions.to('spanish.apkg'));
console.log(output.artifact.path);
await output.artifact.close();
```

```sh
node main.mjs
```

在 Anki 中打开持久保存的 `spanish.apkg`，其中 **Spanish** 牌组包含 **hola → hello**。后续编辑时保持命名空间 `spanish` 和键 `es:hola` 不变。关闭持久产物对象后，文件仍留在磁盘上。导出不需要安装 Anki。

## TypeScript 与下一步任务

[JSON 到 Anki 示例](https://ankiforge.dev/zh-cn/docs/json-to-anki/#node)包含完整的 TypeScript 编译与运行流程。

ESM 使用 `.mjs`，CommonJS 使用 `require('ankiforge')`。包中附带 TypeScript 声明；参见 [TypeScript 配置](https://ankiforge.dev/zh-cn/docs/node-api/#typescript)。

-   [图片与音频](https://ankiforge.dev/zh-cn/docs/media/)及 [Node 媒体 API](https://ankiforge.dev/zh-cn/docs/node-api/#media-and-image-occlusion)
-   [自定义笔记类型](https://ankiforge.dev/zh-cn/docs/custom-notetypes/)和[模板包](https://ankiforge.dev/zh-cn/docs/templates/)
-   [图片遮挡](https://ankiforge.dev/zh-cn/docs/image-occlusion/)
-   [比较与更新](https://ankiforge.dev/zh-cn/docs/updates/)
-   [Node API](https://ankiforge.dev/zh-cn/docs/node-api/)及[排查问题](https://ankiforge.dev/zh-cn/docs/troubleshooting/)

源码编译与本地打包见[源码构建](https://ankiforge.dev/zh-cn/docs/development/)。
