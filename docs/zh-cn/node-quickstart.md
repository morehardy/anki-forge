# 用 Node.js 生成 Anki APKG

使用 Node.js 22.13 或更高版本。在新的应用目录中运行：

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.1
```

这会安装[公开 npm 包](https://www.npmjs.com/package/ankiforge)及匹配的原生可选依赖。平台覆盖范围见[已验证环境](../compatibility.md)。

## 导出牌组

将以下程序保存为 `main.mjs`：

<!-- source: bindings/node/examples/quickstart.mjs -->
```js
import { Project, Note, BuildOptions } from 'ankiforge';

const project = new Project('spanish').defaultDeck('Spanish');
project.add('es:hola', Note.basic('hola', 'hello'));
const output = await project.build(BuildOptions.to('spanish.apkg'));
console.log(output.artifact.path);
await output.artifact.close();
```
<!-- /source -->

```sh
node main.mjs
```

在 Anki 中打开持久保存的 `spanish.apkg`，其中 **Spanish** 牌组包含 **hola → hello**。后续编辑时保持命名空间 `spanish` 和键 `es:hola` 不变。关闭持久产物对象后，文件仍留在磁盘上。导出不需要安装 Anki。

## TypeScript 与下一步任务

[JSON 到 Anki 示例](../json-to-anki.md#node)包含完整的 TypeScript 编译与运行流程。

ESM 使用 `.mjs`，CommonJS 使用 `require('ankiforge')`。包中附带 TypeScript 声明；参见 [TypeScript 配置](../node/api.md#typescript)。

- [图片与音频](../media.md)及 [Node 媒体 API](../node/api.md#media-and-image-occlusion)
- [自定义笔记类型](../custom-notetypes.md)和[模板包](../template-bundles.md)
- [图片遮挡](../image-occlusion.md)
- [比较与更新](../updates.md)
- [Node API](../node/api.md)及[排查问题](../troubleshooting.md)

源码编译与本地打包见[源码构建](../source-builds.md)。
