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
    "title": {"en": "Build Anki decks from code", "zh-CN": "用代码生成 Anki 牌组", "es": "Crea mazos de Anki con código"},
    "description": {"en": "Choose a language, build your first deck, and learn how to add media, templates and repeatable updates.", "zh-CN": "选择开发语言，生成第一个牌组，再添加媒体、模板和可重复的更新。", "es": "Elige un lenguaje, crea tu primer mazo de Anki y aprende a añadir imágenes, audio, plantillas y actualizaciones reproducibles."}
  },
  {
    "name": "languages",
    "source": "website/src/content/docs/docs/languages.mdx",
    "route": "/docs/languages/",
    "title": {"en": "Choose your language", "zh-CN": "选择开发语言", "es": "Elige tu lenguaje de programación"},
    "description": {"en": "Install the public Rust, Node or Python package and export your first Anki deck.", "zh-CN": "安装公开 Rust、Node 或 Python 包，导出第一个 Anki 牌组。", "es": "Instala el paquete publicado de Rust, Node.js o Python y exporta tu primer mazo de Anki."}
  },
  {
    "name": "quickstart",
    "source": "website/src/content/docs/docs/quickstart.mdx",
    "route": "/docs/quickstart/",
    "title": {"en": "Generate Anki flashcards with Rust", "zh-CN": "用 Rust 生成 Anki 卡片", "es": "Genera un APKG de Anki con Rust"},
    "description": {"en": "Install the public Rust package, write a Spanish flashcard, and export a persistent APKG file.", "zh-CN": "安装公开 Rust 包，编写西班牙语卡片并导出持久保存的 APKG。", "es": "Instala el paquete de Rust, crea una tarjeta de vocabulario y exporta un archivo APKG persistente con un programa completo."}
  },
  {
    "source": "docs/json-to-anki.md",
    "route": "/docs/json-to-anki/",
    "name": "json-to-anki",
    "title": {"en": "Generate and update Anki decks from JSON", "zh-CN": "用 JSON 数据生成并更新 Anki 牌组", "es": "Genera y actualiza mazos de Anki desde JSON"},
    "description": {"en": "Read JSON with Rust, Python or TypeScript, package images and audio, then review an answer-only APKG update.", "zh-CN": "用 Rust、Node 或 Python 读取 JSON，打包图片和音频，再检查仅修改答案的更新。", "es": "Lee JSON con Rust, Python o TypeScript, incluye imágenes y audio y revisa una actualización de APKG que solo cambia una respuesta."}
  },
  {
    "source": "docs/csv-to-anki.md",
    "route": "/docs/csv-to-anki/",
    "name": "csv-to-anki",
    "title": {"en": "CSV to Anki: generate flashcards with Python", "zh-CN": "CSV 转 Anki：用 Python 批量生成卡片", "es": "CSV a Anki: crea tarjetas con Python"},
    "description": {"en": "Convert a UTF-8 vocabulary spreadsheet to an Anki APKG with a complete Python script, sample CSV, stable note IDs and import checks.", "zh-CN": "用完整 Python 脚本将 UTF-8 CSV 词汇表转换为 Anki APKG，包含示例数据、稳定笔记 ID 和导入检查步骤。", "es": "Convierte una hoja de vocabulario CSV UTF-8 en un APKG con un script completo de Python, ID de nota estables y comprobaciones de importación."}
  },
  {
    "source": "docs/ai-anki-cards.md",
    "route": "/docs/ai-anki-cards/",
    "name": "ai-anki-cards",
    "title": {"en": "AI-assisted Anki flashcards: from notes to APKG", "zh-CN": "AI 辅助生成 Anki 卡片：从笔记到 APKG", "es": "Crea tarjetas de Anki con ayuda de IA"},
    "description": {"en": "Draft Anki cards with an AI assistant, review the answers, then export an APKG using Anki Forge. Follow a CSV workflow or use a coding-assistant brief.", "zh-CN": "让 AI 助手根据笔记起草卡片，核对答案后用 Anki Forge 导出 APKG。提供 CSV 制卡流程和编程助手任务说明入口。", "es": "Prepara tarjetas a partir de tus apuntes con un asistente de IA, revisa las respuestas y exporta un APKG con Anki Forge mediante CSV o código."}
  },
  {
    "source": "docs/anki-forge-vs-genanki.md",
    "route": "/docs/anki-forge-vs-genanki/",
    "name": "anki-forge-vs-genanki",
    "title": {"en": "Anki Forge vs genanki: choose for your workflow", "zh-CN": "Anki Forge 与 genanki：如何选择", "es": "Anki Forge frente a genanki: cuál elegir"},
    "description": {"en": "Compare Anki APKG authoring workflows, runtime requirements, update review and migration costs.", "zh-CN": "比较 Anki APKG 编写流程、运行环境、更新检查和迁移成本。", "es": "Compara la creación de APKG, los entornos de ejecución, la revisión de actualizaciones y los costes de migración de Anki Forge y genanki."}
  },
  {
    "source": "docs/source-builds.md",
    "route": "/docs/development/",
    "name": "development",
    "title": {"en": "Build SDKs from source", "zh-CN": "从源码构建 SDK", "es": "Compila los SDK desde el código fuente"},
    "description": {"en": "Build SDKs and examples from a source checkout.", "zh-CN": "从仓库源码构建 SDK，运行示例并区分公开包与源码功能。", "es": "Compila los SDK y ejecuta los ejemplos desde el repositorio. Distingue estas compilaciones de los paquetes publicados."}
  },
  {
    "source": "docs/rust-guide.md",
    "route": "/docs/rust-guide/",
    "name": "rust-guide",
    "title": {"en": "Rust authoring guide", "zh-CN": "Rust 编写指南", "es": "Guía de desarrollo con Rust"},
    "description": {"en": "Projects, validation, media and updates through the supported Rust API.", "zh-CN": "通过受支持的 Rust API 创建 Project、校验内容、处理媒体与更新。", "es": "Crea proyectos, valida notas, gestiona recursos multimedia y actualiza mazos con la API pública de Rust."}
  },
  {
    "source": "docs/template-bundles.md",
    "route": "/docs/templates/",
    "name": "templates",
    "title": {"en": "Reusable Anki card templates, CSS and media", "zh-CN": "复用 Anki 卡片模板、CSS 与媒体", "es": "Reutiliza plantillas de Anki, CSS y recursos multimedia"},
    "description": {"en": "Load a reusable Anki note type from one bundle of card templates, CSS and media, for normal or Cloze cards, with Rust, Python or Node.js.", "zh-CN": "从一个模板包加载可复用的 Anki 笔记类型，包含卡片模板、CSS 和媒体，支持普通与 Cloze 卡片，可使用 Rust、Python 或 Node.js。", "es": "Carga un tipo de nota de Anki reutilizable desde un paquete de plantillas, CSS y recursos multimedia, para tarjetas normales o Cloze, con Rust, Python o Node.js."}
  },
  {
    "source": "docs/api-overview.md",
    "route": "/docs/api/",
    "name": "api",
    "title": {"en": "Public API overview", "zh-CN": "公开 API 总览", "es": "Resumen de la API pública"},
    "description": {"en": "Find Rust, Python and Node APIs for authoring, media, diagnostics and updates.", "zh-CN": "查找 Rust、Node 和 Python 的内容编写、媒体、诊断与更新 API。", "es": "Encuentra las API de Rust, Python y Node para crear notas, añadir recursos multimedia, diagnosticar errores y revisar actualizaciones."}
  },
  {
    "source": "docs/installation.md",
    "route": "/docs/installation/",
    "name": "installation",
    "title": {"en": "Add to your application", "zh-CN": "集成到应用", "es": "Integra Anki Forge en tu aplicación de Rust"},
    "description": {"en": "Integrate the Rust crate, configure Cargo dependencies and choose the next guide.", "zh-CN": "安装公开 Rust 包，配置 Cargo 依赖并选择下一步指南。", "es": "Instala el crate publicado, configura las dependencias de Cargo y elige cómo añadir recursos multimedia y actualizaciones."}
  },
  {
    "source": "docs/concepts.md",
    "route": "/docs/concepts/",
    "name": "concepts",
    "title": {"en": "Core concepts", "zh-CN": "核心概念", "es": "Conceptos fundamentales de Anki Forge"},
    "description": {"en": "Projects, owned notes and media, stable keys, text and update identity.", "zh-CN": "理解 Project、拥有所有权的笔记与媒体、稳定键、文本和更新身份。", "es": "Entiende proyectos, notas, mazos, claves estables, recursos multimedia y la identidad de las actualizaciones."}
  },
  {
    "source": "docs/cards.md",
    "route": "/docs/cards/",
    "name": "cards",
    "title": {"en": "Generate Anki Basic and Cloze flashcards", "zh-CN": "生成 Anki 问答卡片与 Cloze 填空卡片", "es": "Crea tarjetas Basic y Cloze de Anki"},
    "description": {"en": "Generate Anki Basic question-and-answer and Cloze fill-in-the-blank cards from code. A verified Rust example builds two notes and three cards.", "zh-CN": "用代码批量生成 Anki 问答（Basic）卡片与 Cloze 填空卡片。经过验证的 Rust 示例用两条笔记生成三张卡片。", "es": "Genera desde código tarjetas Basic de Anki de pregunta y respuesta y tarjetas Cloze con huecos. Un ejemplo verificado de Rust crea dos notas y tres tarjetas."}
  },
  {
    "source": "docs/custom-notetypes.md",
    "route": "/docs/custom-notetypes/",
    "name": "custom-notetypes",
    "title": {"en": "Custom note types", "zh-CN": "自定义笔记类型", "es": "Crea tipos de nota personalizados para Anki"},
    "description": {"en": "Declare fields, card templates and generation rules for your own content.", "zh-CN": "为自己的内容定义字段、卡片模板与生成规则。", "es": "Define campos, plantillas y reglas de generación con claves estables y modelos inmutables reutilizables."}
  },
  {
    "source": "docs/media.md",
    "route": "/docs/media/",
    "name": "media",
    "title": {"en": "Add images and audio to Anki cards", "zh-CN": "为 Anki 卡片添加图片与音频", "es": "Añade imágenes y audio a tus tarjetas Anki"},
    "description": {"en": "Add images and audio to generated Anki cards with Rust, Python or Node.js. Import files, compose card content, and manage export names, MIME types and budgets.", "zh-CN": "用 Rust、Python 或 Node.js 为生成的 Anki 卡片添加图片和音频：导入文件、组合卡片内容，并管理导出名称、MIME 类型与媒体预算。", "es": "Añade imágenes y audio a las tarjetas Anki que generes con Rust, Python o Node.js. Importa archivos, combina contenido y gestiona nombres de exportación, tipos MIME y límites de importación."}
  },
  {
    "source": "docs/image-occlusion.md",
    "route": "/docs/image-occlusion/",
    "name": "image-occlusion",
    "title": {"en": "Generate Anki Image Occlusion cards", "zh-CN": "生成 Anki 图片遮挡卡片", "es": "Genera tarjetas Anki con oclusión de imágenes"},
    "description": {"en": "Create Anki Image Occlusion cards programmatically from an image and the rectangle masks you supply, in either occlusion mode, with Rust, Python or Node.js.", "zh-CN": "用代码批量生成 Anki 图片遮挡卡片：提供图片和矩形遮罩坐标，支持两种遮挡模式，可使用 Rust、Python 或 Node.js。", "es": "Crea tarjetas de oclusión de imágenes de Anki desde código con Rust, Python o Node.js: proporciona la imagen y las máscaras rectangulares y elige uno de los dos modos de oclusión."}
  },
  {
    "source": "docs/updates.md",
    "route": "/docs/updates/",
    "name": "updates",
    "title": {"en": "Update Anki decks with stable note IDs", "zh-CN": "用稳定笔记 ID 更新 Anki 牌组", "es": "Actualiza mazos de Anki con ID de nota estables"},
    "description": {"en": "Release a new version of a generated Anki deck: keep stable note IDs, build from the previous original APKG and review the comparison before publishing.", "zh-CN": "发布生成牌组的新版本：保持稳定笔记 ID，以上一版原始 APKG 为基线构建，并在分发前检查比较结果。", "es": "Publica una nueva versión de un mazo de Anki generado: conserva los ID de nota estables, parte del APKG original anterior y revisa la comparación antes de distribuirlo."}
  },
  {
    "source": "docs/build-guarantees.md",
    "route": "/docs/build-guarantees/",
    "name": "build-guarantees",
    "title": {"en": "Build and output guarantees", "zh-CN": "构建与输出保证", "es": "Garantías de generación y de los archivos APKG"},
    "description": {"en": "Understand temporary artifacts, path protection and candidate publication.", "zh-CN": "理解临时产物、路径保护和候选版本发布。", "es": "Comprende la vida útil de los archivos, los límites de inspección y la publicación de una versión candidata revisada."}
  },
  {
    "source": "docs/troubleshooting.md",
    "route": "/docs/troubleshooting/",
    "name": "troubleshooting",
    "title": {"en": "Troubleshooting", "zh-CN": "排查问题", "es": "Soluciona errores de Anki Forge"},
    "description": {"en": "Resolve installation, media, template, identity and output errors.", "zh-CN": "解决安装、媒体、模板、身份与输出错误。", "es": "Diagnostica errores de instalación, notas, recursos multimedia, plantillas, identidad y publicación de archivos."}
  },
  {
    "source": "docs/compatibility.md",
    "route": "/docs/compatibility/",
    "name": "compatibility",
    "title": {"en": "Compatibility and releases", "zh-CN": "兼容性与发布", "es": "Compatibilidad de Anki Forge y entornos verificados"},
    "description": {"en": "Published packages, verified environments and Anki client conditions.", "zh-CN": "查看公开包、已验证运行环境和 Anki 客户端条件。", "es": "Consulta los paquetes públicos, los entornos probados y las condiciones de importación y reproducción en Anki."}
  },
  {
    "source": "docs/rust-api.md",
    "route": "/docs/rust-api/",
    "name": "rust-api",
    "title": {"en": "Rust API guide", "zh-CN": "Rust API 指南", "es": "Referencia de la API de Rust"},
    "description": {"en": "Find public authoring methods, build defaults, reports and diagnostic behavior.", "zh-CN": "查找内容编写方法、构建默认值、报告和诊断行为。", "es": "Consulta los métodos de creación de contenido, las opciones de generación, los informes y los errores públicos de Rust."}
  },
  {
    "source": "docs/node/quick-start.md",
    "route": "/docs/node-quickstart/",
    "name": "node-quickstart",
    "title": {"en": "Generate Anki flashcards with Node.js", "zh-CN": "用 Node.js 生成 Anki 卡片", "es": "Genera un APKG de Anki con Node.js"},
    "description": {"en": "Install ankiforge from npm, generate Anki flashcards with Node.js or TypeScript, and export an APKG deck you can import into Anki.", "zh-CN": "从 npm 安装 ankiforge，用 Node.js 或 TypeScript 生成 Anki 卡片，并导出可导入 Anki 的 APKG 牌组。", "es": "Instala el paquete publicado de Node.js y exporta tu primer mazo con un programa completo. Continúa con TypeScript, imágenes y actualizaciones."}
  },
  {
    "source": "docs/node/api.md",
    "route": "/docs/node-api/",
    "name": "node-api",
    "title": {"en": "Node and TypeScript API", "zh-CN": "Node 与 TypeScript API", "es": "API de Node y TypeScript"},
    "description": {"en": "Public methods, async behavior, build options and reports.", "zh-CN": "公开方法、异步行为、构建选项和报告。", "es": "Métodos públicos de Node, operaciones asíncronas, opciones de generación, gestión de archivos de salida e informes."}
  },
  {
    "source": "docs/python/quick-start.md",
    "route": "/docs/python-quickstart/",
    "name": "python-quickstart",
    "title": {"en": "Generate Anki flashcards with Python", "zh-CN": "用 Python 生成 Anki 卡片", "es": "Genera un APKG de Anki con Python"},
    "description": {"en": "Install ankiforge from PyPI, generate Anki flashcards with a short Python script, and export an APKG deck you can import into Anki.", "zh-CN": "从 PyPI 安装 ankiforge，用简短的 Python 脚本生成 Anki 问答卡片，并导出可导入 Anki 的 APKG 牌组。", "es": "Instala el wheel publicado de Python y crea tu primer archivo APKG de Anki con un programa completo y claves de nota estables."}
  },
  {
    "source": "docs/python/api.md",
    "route": "/docs/python-api/",
    "name": "python-api",
    "title": {"en": "Python API", "zh-CN": "Python API", "es": "API de Python"},
    "description": {"en": "Project, media, build options, reports and temporary artifact ownership.", "zh-CN": "Project、媒体、构建选项、报告和临时产物的所有权。", "es": "Project, recursos multimedia, opciones de generación, informes y gestión de la vida útil de los archivos temporales."}
  },
  {
    "source": "docs/python/diagnostics.md",
    "route": "/docs/python-diagnostics/",
    "name": "python-diagnostics",
    "title": {"en": "Python diagnostics", "zh-CN": "Python 诊断", "es": "Diagnósticos de Python"},
    "description": {"en": "Understand native authoring errors, build reports and update evidence.", "zh-CN": "理解原生内容编写错误、构建报告和更新证据。", "es": "Interpreta los errores nativos, los informes de generación y las pruebas y políticas de actualización de Anki Forge."}
  },
  {
    "source": "bindings/python/MIGRATION.md",
    "route": "/docs/python-migration/",
    "name": "python-migration",
    "title": {"en": "Python API transition", "zh-CN": "Python API 迁移", "es": "Transición a la nueva API de Python"},
    "description": {"en": "Use the current owned-value SDK and understand the clean-slate boundary.", "zh-CN": "使用当前拥有值所有权的 SDK，理解新版 API 的迁移边界。", "es": "Adapta tu código al SDK actual de Python y conoce los límites de compatibilidad con la API anterior."}
  },
  {
    "source": "docs/python/genanki-migration.md",
    "route": "/docs/genanki-migration/",
    "name": "genanki-migration",
    "title": {"en": "Move from genanki", "zh-CN": "从 genanki 迁移", "es": "Pasar de genanki a Anki Forge"},
    "description": {"en": "Understand content, template and media behavior when moving to Anki Forge.", "zh-CN": "了解迁入 Anki Forge 时内容、模板与媒体行为的变化。", "es": "Adapta notas, modelos, plantillas y recursos multimedia de genanki al modelo de Anki Forge y sus claves estables."}
  }
];

export const docGroups = [
  { labels: { en: 'Start here', 'zh-CN': '从这里开始', es: "Empieza aquí" }, ids: ['docs', 'languages', 'quickstart', 'installation', 'concepts', 'anki-forge-vs-genanki'] },
  { labels: { en: 'Guides', 'zh-CN': '使用指南', es: "Guías" }, ids: ['csv-to-anki', 'json-to-anki', 'ai-anki-cards', 'cards', 'media', 'custom-notetypes', 'templates', 'image-occlusion', 'updates'] },
  { labels: { en: 'Language guides', 'zh-CN': '开发语言指南', es: "Guías por lenguaje" }, ids: ['rust-guide', 'node-quickstart', 'python-quickstart', 'python-migration', 'genanki-migration'] },
  { labels: { en: 'Reference', 'zh-CN': '参考', es: "Referencia" }, collapsed: true, ids: ['api', 'rust-api', 'node-api', 'python-api', 'build-guarantees', 'compatibility'] },
  { labels: { en: 'Help', 'zh-CN': '帮助', es: "Ayuda" }, ids: ['troubleshooting', 'python-diagnostics', 'development'] },
];
/** @type {{ id: string, label: Record<string, string> }[]} */
export const navigation = [
  { id: 'docs', label: { en: 'Guide', 'zh-CN': '指南', es: "Guía" } },
  { id: 'examples', label: { en: 'Examples', 'zh-CN': '示例', es: "Ejemplos" } },
  { id: 'blog', label: { en: 'Blog', 'zh-CN': '博客', es: "Blog" } },
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
