// Only these topics have real translations. Do not enable Starlight locale fallback.
export const corePages = [
  { id: 'home', en: '/', zh: '/zh-cn/', enSource: 'website/src/pages/index.astro', zhSource: 'website/src/pages/zh-cn/index.astro', title: '首页' },
  { id: 'languages', title: '选择开发语言', description: '选择 Rust、Python 或 Node.js，用公开的 Anki Forge 包生成 Anki 牌组。', enSource: 'website/src/content/docs/docs/languages.mdx' },
  { id: 'quickstart', title: '用 Rust 生成第一个 Anki 牌组', description: '安装 Rust 包，编写完整程序，生成可导入 Anki 的 APKG 文件。', enSource: 'website/src/content/docs/docs/quickstart.mdx' },
  { id: 'python-quickstart', title: '用 Python 生成第一个 Anki 牌组', description: '安装 Python wheel，运行完整程序，生成可导入 Anki 的 APKG 文件。', enSource: 'docs/python/quick-start.md' },
  { id: 'node-quickstart', title: '用 Node.js 生成第一个 Anki 牌组', description: '安装 Node.js 包，运行完整程序，生成 APKG，并了解 TypeScript 的编译方法。', enSource: 'docs/node/quick-start.md' },
  { id: 'json-to-anki', title: '用 JSON 数据生成并更新 Anki 牌组', description: '用 Rust、Python 或 TypeScript 读取同一份 JSON，打包图片和音频，检查变更后发布更新。', enSource: 'docs/json-to-anki.md' },
  { id: 'anki-forge-vs-genanki', title: 'Anki Forge 与 genanki：如何选择', description: '按任务、运行环境和迁移成本选择 Anki 牌组生成库，了解更新流程与性能数据的适用范围。', enSource: 'docs/anki-forge-vs-genanki.md' },
].map(page => ({
  en: `/docs/${page.id}/`, zh: `/zh-cn/docs/${page.id}/`, zhSource: `docs/zh-cn/${page.id}.md`,
  author: 'Anki Forge maintainers', reviewedAt: '2026-10-25', ...page,
}));

export const translatedDocs = corePages.filter(page => page.id !== 'home').map(page => ({
  source: page.zhSource, route: page.zh, name: page.id, title: page.title, description: page.description,
}));

export function pagePair(pathname, base = '/') {
  const path = base === '/' ? pathname : pathname.slice(base.length);
  return corePages.find(page => page.en === path || page.zh === path);
}
