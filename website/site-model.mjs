import { catalogs } from './src/i18n/messages.mjs';

import { configuredLocales, locales, localeIds, localePath, resolveLocalePath } from './locale-model.mjs';
export { configuredLocales, locales, localeIds, releaseLocales, stripBase } from './locale-model.mjs';

// Public topic identities are independent of translated titles and URL prefixes.
// Translated text is keyed by locale ID; adding a release locale adds keys, not fields.
const documents = [
  {
    "name": "docs",
    "source": "website/src/content/docs/docs/index.mdx",
    "route": "/docs/",
    "title": { "en": "Build Anki decks from code", "zh-CN": "用代码生成 Anki 牌组" },
    "description": { "en": "Choose a language, build your first deck, and learn how to add media, templates and repeatable updates.", "zh-CN": "选择开发语言，生成第一个牌组，再添加媒体、模板和可重复的更新。" }
  },
  {
    "name": "languages",
    "source": "website/src/content/docs/docs/languages.mdx",
    "route": "/docs/languages/",
    "title": { "en": "Choose your language", "zh-CN": "选择开发语言" },
    "description": { "en": "Install the public Rust, Node or Python package and export your first Anki deck.", "zh-CN": "安装公开 Rust、Node 或 Python 包，导出第一个 Anki 牌组。" }
  },
  {
    "name": "quickstart",
    "source": "website/src/content/docs/docs/quickstart.mdx",
    "route": "/docs/quickstart/",
    "title": { "en": "Generate an Anki APKG with Rust", "zh-CN": "用 Rust 生成 Anki APKG" },
    "description": { "en": "Install the public Rust package, write a Spanish flashcard, and export a persistent APKG file.", "zh-CN": "安装公开 Rust 包，编写西班牙语卡片并导出持久保存的 APKG。" }
  },
  {
    "source": "docs/json-to-anki.md",
    "route": "/docs/json-to-anki/",
    "name": "json-to-anki",
    "title": { "en": "Generate and update Anki decks from JSON", "zh-CN": "用 JSON 数据生成并更新 Anki 牌组" },
    "description": { "en": "Read JSON with Rust, Python or TypeScript, package images and audio, then review an answer-only APKG update.", "zh-CN": "用 Rust、Node 或 Python 读取 JSON，打包图片和音频，再检查仅修改答案的更新。" }
  },
  {
    "source": "docs/csv-to-anki.md",
    "route": "/docs/csv-to-anki/",
    "name": "csv-to-anki",
    "title": { "en": "CSV to Anki: generate flashcards with Python", "zh-CN": "CSV 转 Anki：用 Python 批量生成卡片" },
    "description": { "en": "Convert a UTF-8 vocabulary spreadsheet to an Anki APKG with a complete Python script, sample CSV, stable note IDs and import checks.", "zh-CN": "用完整 Python 脚本将 UTF-8 CSV 词汇表转换为 Anki APKG，包含示例数据、稳定笔记 ID 和导入检查步骤。" }
  },
  {
    "source": "docs/ai-anki-cards.md",
    "route": "/docs/ai-anki-cards/",
    "name": "ai-anki-cards",
    "title": { "en": "AI-assisted Anki flashcards: from notes to APKG", "zh-CN": "AI 辅助生成 Anki 卡片：从笔记到 APKG" },
    "description": { "en": "Draft Anki cards with an AI assistant, review the answers, then export an APKG using Anki Forge. Follow a CSV workflow or use a coding-assistant brief.", "zh-CN": "让 AI 助手根据笔记起草卡片，核对答案后用 Anki Forge 导出 APKG。提供 CSV 制卡流程和编程助手任务说明入口。" }
  },
  {
    "source": "docs/anki-forge-vs-genanki.md",
    "route": "/docs/anki-forge-vs-genanki/",
    "name": "anki-forge-vs-genanki",
    "title": { "en": "Anki Forge vs genanki: choose for your workflow", "zh-CN": "Anki Forge 与 genanki：如何选择" },
    "description": { "en": "Compare Anki APKG authoring workflows, runtime requirements, update review and migration costs.", "zh-CN": "比较 Anki APKG 编写流程、运行环境、更新检查和迁移成本。" }
  },
  {
    "source": "docs/source-builds.md",
    "route": "/docs/development/",
    "name": "development",
    "title": { "en": "Build SDKs from source", "zh-CN": "从源码构建 SDK" },
    "description": { "en": "Build SDKs and examples from a source checkout.", "zh-CN": "从仓库源码构建 SDK，运行示例并区分公开包与源码功能。" }
  },
  {
    "source": "docs/rust-guide.md",
    "route": "/docs/rust-guide/",
    "name": "rust-guide",
    "title": { "en": "Rust authoring guide", "zh-CN": "Rust 编写指南" },
    "description": { "en": "Projects, validation, media and updates through the supported Rust API.", "zh-CN": "通过受支持的 Rust API 创建 Project、校验内容、处理媒体与更新。" }
  },
  {
    "source": "docs/template-bundles.md",
    "route": "/docs/templates/",
    "name": "templates",
    "title": { "en": "Reusable Anki card templates, CSS and media", "zh-CN": "复用 Anki 卡片模板、CSS 与媒体" },
    "description": { "en": "Import complete reusable Cloze and normal templates, CSS and media.", "zh-CN": "导入完整的普通与 Cloze 模板、CSS 和媒体。" }
  },
  {
    "source": "docs/api-overview.md",
    "route": "/docs/api/",
    "name": "api",
    "title": { "en": "Public API overview", "zh-CN": "公开 API 总览" },
    "description": { "en": "Find Rust, Python and Node APIs for authoring, media, diagnostics and updates.", "zh-CN": "查找 Rust、Node 和 Python 的内容编写、媒体、诊断与更新 API。" }
  },
  {
    "source": "docs/installation.md",
    "route": "/docs/installation/",
    "name": "installation",
    "title": { "en": "Add to your application", "zh-CN": "集成到应用" },
    "description": { "en": "Integrate the Rust crate, configure Cargo dependencies and choose the next guide.", "zh-CN": "安装公开 Rust 包，配置 Cargo 依赖并选择下一步指南。" }
  },
  {
    "source": "docs/concepts.md",
    "route": "/docs/concepts/",
    "name": "concepts",
    "title": { "en": "Core concepts", "zh-CN": "核心概念" },
    "description": { "en": "Projects, owned notes and media, stable keys, text and update identity.", "zh-CN": "理解 Project、拥有所有权的笔记与媒体、稳定键、文本和更新身份。" }
  },
  {
    "source": "docs/cards.md",
    "route": "/docs/cards/",
    "name": "cards",
    "title": { "en": "Generate Anki Basic and Cloze flashcards", "zh-CN": "生成 Anki 问答卡片与 Cloze 填空卡片" },
    "description": { "en": "Build two notes and three cards with a complete verified Rust example.", "zh-CN": "运行完整 Rust 示例，用两个笔记生成三张卡片。" }
  },
  {
    "source": "docs/custom-notetypes.md",
    "route": "/docs/custom-notetypes/",
    "name": "custom-notetypes",
    "title": { "en": "Custom note types", "zh-CN": "自定义笔记类型" },
    "description": { "en": "Declare fields, card templates and generation rules for your own content.", "zh-CN": "为自己的内容定义字段、卡片模板与生成规则。" }
  },
  {
    "source": "docs/media.md",
    "route": "/docs/media/",
    "name": "media",
    "title": { "en": "Add images and audio to Anki cards", "zh-CN": "为 Anki 卡片添加图片与音频" },
    "description": { "en": "Import images and audio, compose typed content, and manage export names, MIME types and media budgets.", "zh-CN": "导入图片和音频，组合类型化内容，并管理导出名称、MIME 类型与媒体预算。" }
  },
  {
    "source": "docs/image-occlusion.md",
    "route": "/docs/image-occlusion/",
    "name": "image-occlusion",
    "title": { "en": "Generate Anki Image Occlusion cards", "zh-CN": "生成 Anki 图片遮挡卡片" },
    "description": { "en": "Create rectangle-based image questions in both occlusion modes.", "zh-CN": "用矩形区域创建图像问题，了解两种遮挡模式。" }
  },
  {
    "source": "docs/updates.md",
    "route": "/docs/updates/",
    "name": "updates",
    "title": { "en": "Update Anki decks with stable note IDs", "zh-CN": "用稳定笔记 ID 更新 Anki 牌组" },
    "description": { "en": "Compare and build releases using verified original APKG baselines.", "zh-CN": "以原始 APKG 为基线比较并构建新版本。" }
  },
  {
    "source": "docs/build-guarantees.md",
    "route": "/docs/build-guarantees/",
    "name": "build-guarantees",
    "title": { "en": "Build and output guarantees", "zh-CN": "构建与输出保证" },
    "description": { "en": "Understand temporary artifacts, path protection and candidate publication.", "zh-CN": "理解临时产物、路径保护和候选版本发布。" }
  },
  {
    "source": "docs/troubleshooting.md",
    "route": "/docs/troubleshooting/",
    "name": "troubleshooting",
    "title": { "en": "Troubleshooting", "zh-CN": "排查问题" },
    "description": { "en": "Resolve installation, media, template, identity and output errors.", "zh-CN": "解决安装、媒体、模板、身份与输出错误。" }
  },
  {
    "source": "docs/compatibility.md",
    "route": "/docs/compatibility/",
    "name": "compatibility",
    "title": { "en": "Compatibility and releases", "zh-CN": "兼容性与发布" },
    "description": { "en": "Published packages, verified environments and Anki client conditions.", "zh-CN": "查看公开包、已验证运行环境和 Anki 客户端条件。" }
  },
  {
    "source": "docs/rust-api.md",
    "route": "/docs/rust-api/",
    "name": "rust-api",
    "title": { "en": "Rust API guide", "zh-CN": "Rust API 指南" },
    "description": { "en": "Find public authoring methods, build defaults, reports and diagnostic behavior.", "zh-CN": "查找内容编写方法、构建默认值、报告和诊断行为。" }
  },
  {
    "source": "docs/node/quick-start.md",
    "route": "/docs/node-quickstart/",
    "name": "node-quickstart",
    "title": { "en": "Generate an Anki APKG with Node.js", "zh-CN": "用 Node.js 生成 Anki APKG" },
    "description": { "en": "Install the public Node package and export your first deck.", "zh-CN": "安装公开 Node 包并导出第一个牌组。" }
  },
  {
    "source": "docs/node/api.md",
    "route": "/docs/node-api/",
    "name": "node-api",
    "title": { "en": "Node and TypeScript API", "zh-CN": "Node 与 TypeScript API" },
    "description": { "en": "Public methods, async behavior, build options and reports.", "zh-CN": "公开方法、异步行为、构建选项和报告。" }
  },
  {
    "source": "docs/python/quick-start.md",
    "route": "/docs/python-quickstart/",
    "name": "python-quickstart",
    "title": { "en": "Generate an Anki APKG with Python", "zh-CN": "用 Python 生成 Anki APKG" },
    "description": { "en": "Install the public Python wheel and generate your first Anki APKG.", "zh-CN": "安装公开 Python wheel 并生成第一个 Anki APKG。" }
  },
  {
    "source": "docs/python/api.md",
    "route": "/docs/python-api/",
    "name": "python-api",
    "title": { "en": "Python API", "zh-CN": "Python API" },
    "description": { "en": "Project, media, build options, reports and temporary artifact ownership.", "zh-CN": "Project、媒体、构建选项、报告和临时产物的所有权。" }
  },
  {
    "source": "docs/python/diagnostics.md",
    "route": "/docs/python-diagnostics/",
    "name": "python-diagnostics",
    "title": { "en": "Python diagnostics", "zh-CN": "Python 诊断" },
    "description": { "en": "Understand native authoring errors, build reports and update evidence.", "zh-CN": "理解原生内容编写错误、构建报告和更新证据。" }
  },
  {
    "source": "bindings/python/MIGRATION.md",
    "route": "/docs/python-migration/",
    "name": "python-migration",
    "title": { "en": "Python API transition", "zh-CN": "Python API 迁移" },
    "description": { "en": "Use the current owned-value SDK and understand the clean-slate boundary.", "zh-CN": "使用当前拥有值所有权的 SDK，理解新版 API 的迁移边界。" }
  },
  {
    "source": "docs/python/genanki-migration.md",
    "route": "/docs/genanki-migration/",
    "name": "genanki-migration",
    "title": { "en": "Move from genanki", "zh-CN": "从 genanki 迁移" },
    "description": { "en": "Understand content, template and media behavior when moving to Anki Forge.", "zh-CN": "了解迁入 Anki Forge 时内容、模板与媒体行为的变化。" }
  }
];

export const docGroups = [
  { labels: { en: 'Start here', 'zh-CN': '从这里开始' }, ids: ['docs', 'languages', 'quickstart', 'installation', 'concepts', 'anki-forge-vs-genanki'] },
  { labels: { en: 'Guides', 'zh-CN': '使用指南' }, ids: ['csv-to-anki', 'json-to-anki', 'ai-anki-cards', 'cards', 'media', 'custom-notetypes', 'templates', 'image-occlusion', 'updates'] },
  { labels: { en: 'Language guides', 'zh-CN': '开发语言指南' }, ids: ['rust-guide', 'node-quickstart', 'python-quickstart', 'python-migration', 'genanki-migration'] },
  { labels: { en: 'Reference', 'zh-CN': '参考' }, collapsed: true, ids: ['api', 'rust-api', 'node-api', 'python-api', 'build-guarantees', 'compatibility'] },
  { labels: { en: 'Help', 'zh-CN': '帮助' }, ids: ['troubleshooting', 'python-diagnostics', 'development'] },
];
/** @type {{ id: string, label: Record<string, string> }[]} */
export const navigation = [
  { id: 'docs', label: { en: 'Guide', 'zh-CN': '指南' } },
  { id: 'examples', label: { en: 'Examples', 'zh-CN': '示例' } },
  { id: 'blog', label: { en: 'Blog', 'zh-CN': '博客' } },
];
export const footerNavigation = [
  { id: 'articles', pageId: 'blog', resource: '', label: 'footer.articles' },
  { id: 'feed', pageId: '', resource: 'feed', label: 'footer.rss' },
  { id: 'license', pageId: '', resource: 'license', label: 'footer.license' },
  { id: 'repository', pageId: '', resource: 'repository', label: 'nav.repository' },
];
export const nextTasks = [
  { pageId: 'json-to-anki', label: 'home.json', primary: true },
  { pageId: 'csv-to-anki', label: 'home.csv', primary: false },
  { pageId: 'ai-anki-cards', label: 'home.aiCards', primary: false },
  { pageId: 'anki-forge-vs-genanki', label: 'home.compare', primary: false },
];
export const homepageModules = ['hero', 'install', 'ai-quickstart', 'performance', 'examples', 'updates', 'features', 'next'];
const common = [
  { id: 'home', route: '/', type: 'home', section: 'home', source: 'website/src/components/HomePage.astro', capabilities: homepageModules },
  { id: 'examples', route: '/examples/', type: 'examples', section: 'examples', source: 'website/src/components/ExamplesPage.astro', capabilities: ['gallery', 'updates'] },
  { id: 'blog', route: '/blog/', type: 'blog-list', section: 'blog', source: 'website/src/components/BlogPage.astro', capabilities: ['articles', 'feed'] },
];
const articles = ['your-first-anki-deck', 'stable-ids-and-deck-updates'].map(id => ({
  id: `blog:${id}`, route: `/blog/${id}/`, type: 'article', section: 'blog',
  sources: Object.fromEntries(Object.entries(configuredLocales).map(([locale, config]) => [locale, `website/src/content/blog/${config.prefix ? config.prefix.slice(1) + '/' : ''}${id}.mdx`])),
  capabilities: ['article', 'date'], metadata: { source: 'frontmatter', title: 'title', description: 'description' },
}));
export const pages = [
  ...common,
  ...documents.map(doc => ({
    id: doc.name, route: doc.route, type: 'docs', section: 'docs', sources: Object.fromEntries(Object.entries(configuredLocales).map(([locale, config]) => [locale, locale === 'en' ? doc.source : `docs${config.prefix}/${doc.name === 'docs' ? 'index' : doc.name}.md`])),
    title: doc.title, description: doc.description,
    capabilities: ['sidebar', 'search', 'toc', 'pagination', 'edit', 'code-copy'],
  })),
  ...articles,
].map((page, order) => {
  const title = ['home', 'examples', 'blog'].includes(page.id)
    ? Object.fromEntries(localeIds.map(locale => [locale, catalogs[locale]?.[`${page.id}.title`]])) : page.title;
  const description = ['home', 'examples', 'blog'].includes(page.id)
    ? Object.fromEntries(localeIds.map(locale => [locale, catalogs[locale]?.[`${page.id}.description`]])) : page.description;
  const sources = page.sources ?? Object.fromEntries(localeIds.map(locale => [locale, page.source]));
  return {
    ...page, title, description, sources, order,
    metadata: Object.fromEntries(localeIds.map(locale => [locale, { title: title?.[locale], description: description?.[locale] }])),
  };
});
export const docPages = pages.filter(page => page.type === 'docs');

export function pagePath(id, locale = 'en', section = '') {
  const page = pages.find(page => page.id === id);
  if (!page || !Object.hasOwn(locales, locale)) throw new Error(`Unknown page or release locale: ${id}/${locale}`);
  return `${localePath(page.route, locale)}${section ? `#${section}` : ''}`;
}
export function resolvePage(pathname, base = '/') {
  const resolved = resolveLocalePath(pathname, base);
  if (!resolved) return null;
  const page = pages.find(page => page.route === resolved.route);
  return page ? { page, locale: resolved.locale, path: localePath(page.route, resolved.locale) } : null;
}
export function pagePair(pathname, base = '/') {
  return resolvePage(pathname, base)?.page;
}
export function localeFromPath(pathname, base = '/') {
  return resolveLocalePath(pathname, base)?.locale ?? 'en';
}
export function documentationEntries(active = localeIds) {
  return docPages.flatMap(page => active.filter(locale => !page.sources[locale]?.startsWith('website/')).map(locale => ({
    id: page.id, name: page.id === 'docs' ? 'index' : page.id, locale,
    source: page.sources[locale], route: pagePath(page.id, locale),
    title: page.title[locale], description: page.description[locale],
    output: `src/content/docs${locales[locale].prefix}/docs/${page.id === 'docs' ? 'index' : page.id}.md`,
  })));
}
export const importedDocs = documentationEntries().filter(entry => entry.locale === 'en');
// This compatibility name now covers every non-English release locale.
export const translatedDocs = documentationEntries().filter(entry => entry.locale !== 'en');
