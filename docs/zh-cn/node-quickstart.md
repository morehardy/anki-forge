# 用 Node.js 生成第一个 Anki 牌组

使用 Node.js 22.13 或更新版本，安装时保留对应平台的原生可选依赖。本页使用 JavaScript，可以直接由 Node.js 执行。

## 安装

在新的项目目录中安装公开版本 0.3.1：

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.1
```

具体平台范围见[兼容性说明（英文）](../compatibility.md)。

## 生成 APKG

将下面的完整程序保存为 `main.mjs`：

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

运行：

```sh
node main.mjs
```

程序会写出持久保存的 `spanish.apkg`。在 Anki 中导入后，可以在 Spanish 牌组学习 **hola → hello**。
生成 APKG 不需要安装 Anki。后续修改答案时，保持 namespace `spanish` 和 note key `es:hola` 不变。

## 继续完成实际任务

[JSON 教程](json-to-anki.md#node)包含图片、音频和更新流程，以及完整的 TypeScript 编译运行命令。
发布更新时请保留上一份原始分发 APKG，不能用 Anki 重新导出的文件代替。
查看[更新条件（英文）](../updates.md)、[API 参考（英文）](../api-overview.md)，
或[选择其他开发语言](languages.md)。
