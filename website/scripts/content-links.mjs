import path from 'node:path';
import { translatedDocs } from './core-pages.mjs';
import { repository, withBase } from '../site.config.mjs';

export const importedDocs = [
  { source: 'docs/json-to-anki.md', route: '/docs/json-to-anki/', name: 'json-to-anki', title: 'Generate and update Anki decks from JSON', description: 'Read JSON with Rust, Python or TypeScript, package images and audio, then review an answer-only APKG update.' },
  { source: 'docs/anki-forge-vs-genanki.md', route: '/docs/anki-forge-vs-genanki/', name: 'anki-forge-vs-genanki', title: 'Anki Forge vs genanki: choose for your workflow', description: 'Compare Anki APKG authoring workflows, runtime requirements, update review and migration costs.' },
  {source: 'docs/source-builds.md', route: '/docs/development/', name: 'development', title: 'Build SDKs from source', description: 'Build SDKs and examples from a source checkout.'},
  {"source": "docs/rust-guide.md", "route": "/docs/rust-guide/", "name": "rust-guide", "title": "Rust authoring guide", "description": "Projects, validation, media and updates through the supported Rust API."},
  {"source": "docs/template-bundles.md", "route": "/docs/templates/", "name": "templates", "title": "Template bundles", "description": "Import complete reusable Cloze and normal templates, CSS and media."},
  {"source": "docs/api-overview.md", "route": "/docs/api/", "name": "api", "title": "Public API overview", "description": "Find Rust, Python and Node APIs for authoring, media, diagnostics and updates."},
  {"source": "docs/installation.md", "route": "/docs/installation/", "name": "installation", "title": "Add to your application", "description": "Integrate the Rust crate, configure Cargo dependencies and choose the next guide."},
  {"source": "docs/concepts.md", "route": "/docs/concepts/", "name": "concepts", "title": "Core concepts", "description": "Projects, owned notes and media, stable keys, text and update identity."},
  {"source": "docs/cards.md", "route": "/docs/cards/", "name": "cards", "title": "Basic and Cloze cards", "description": "Build two notes and three cards with a complete verified Rust example."},
  {"source": "docs/custom-notetypes.md", "route": "/docs/custom-notetypes/", "name": "custom-notetypes", "title": "Custom note types", "description": "Declare fields, card templates and generation rules for your own content."},
  {"source": "docs/media.md", "route": "/docs/media/", "name": "media", "title": "Images and audio", "description": "Generate and package a real waveform image and a one-second audio tone."},
  {"source": "docs/image-occlusion.md", "route": "/docs/image-occlusion/", "name": "image-occlusion", "title": "Image Occlusion", "description": "Create rectangle-based image questions in both occlusion modes."},
  {"source": "docs/updates.md", "route": "/docs/updates/", "name": "updates", "title": "Update and distribute", "description": "Compare and build releases using verified original APKG baselines."},
  {"source": "docs/build-guarantees.md", "route": "/docs/build-guarantees/", "name": "build-guarantees", "title": "Build and output guarantees", "description": "Understand temporary artifacts, path protection and candidate publication."},
  {"source": "docs/troubleshooting.md", "route": "/docs/troubleshooting/", "name": "troubleshooting", "title": "Troubleshooting", "description": "Resolve installation, media, template, identity and output errors."},
  {"source": "docs/compatibility.md", "route": "/docs/compatibility/", "name": "compatibility", "title": "Compatibility and releases", "description": "Published packages, verified environments and Anki client conditions."},
  {"source": "docs/rust-api.md", "route": "/docs/rust-api/", "name": "rust-api", "title": "Rust API guide", "description": "Find public authoring methods, build defaults, reports and diagnostic behavior."},
  {"source": "docs/node/quick-start.md", "route": "/docs/node-quickstart/", "name": "node-quickstart", "title": "Generate an Anki APKG with Node.js", "description": "Install the public Node package and export your first deck."},
  {"source": "docs/node/api.md", "route": "/docs/node-api/", "name": "node-api", "title": "Node and TypeScript API", "description": "Public methods, async behavior, build options and reports."},
  {"source": "docs/python/quick-start.md", "route": "/docs/python-quickstart/", "name": "python-quickstart", "title": "Generate an Anki APKG with Python", "description": "Install the public Python wheel and generate your first Anki APKG."},
  {"source": "docs/python/api.md", "route": "/docs/python-api/", "name": "python-api", "title": "Python API", "description": "Project, media, build options, reports and temporary artifact ownership."},
  {"source": "docs/python/diagnostics.md", "route": "/docs/python-diagnostics/", "name": "python-diagnostics", "title": "Python diagnostics", "description": "Understand native authoring errors, build reports and update evidence."},
  {"source": "bindings/python/MIGRATION.md", "route": "/docs/python-migration/", "name": "python-migration", "title": "Python API transition", "description": "Use the current owned-value SDK and understand the clean-slate boundary."},
  {"source": "docs/python/genanki-migration.md", "route": "/docs/genanki-migration/", "name": "genanki-migration", "title": "Move from genanki", "description": "Understand content, template and media behavior when moving to Anki Forge."},
];

export function rewriteTarget(target, source, base) {
  if (/^(?:[a-z][a-z\d+.-]*:|\/\/|#)/i.test(target)) return target;
  const [, pathname, suffix] = target.match(/^([^?#]*)(.*)$/);
  const resolved = path.posix.normalize(path.posix.join(path.posix.dirname(source), pathname));
  if (resolved.startsWith('docs/recipes/json-to-anki/')) return `${withBase('/generated/recipes/json-to-anki/', base)}${path.posix.basename(resolved)}${suffix}`;
  const doc = [...importedDocs, ...translatedDocs].find(entry => entry.source === resolved);
  if (doc) return `${withBase(doc.route, base)}${suffix}`;
  return `${repository}/blob/main/${resolved.split('/').map(encodeURIComponent).join('/')}${suffix}`;
}

export function rewriteMarkdown(markdown, source, base) {
  // Source examples remain verbatim. Only prose links are made site-aware.
  return markdown.split(/(```[\s\S]*?```|~~~[\s\S]*?~~~)/g).map((part, i) => i % 2 ? part : part
    .replace(/(!?\[[^\]\n]*\]\()([^\s)]+)(\))/g, (_, start, target, end) => `${start}${rewriteTarget(target, source, base)}${end}`)
    .replace(/^(\[[^\]\n]+\]:\s*)(\S+)/gm, (_, start, target) => `${start}${rewriteTarget(target, source, base)}`)
  ).join('');
}

// Starlight supplies the page title; the crate README uses an HTML title on GitHub.
export function stripDocumentTitle(markdown) {
  const content = markdown.replace(/^# .+\r?\n+/, '');
  const boundary = content.search(/^```|^~~~|^## /m);
  const end = boundary < 0 ? content.length : boundary;
  return content.slice(0, end).replace(/<h1\b[^>]*>[\s\S]*?<\/h1>\s*/i, '') + content.slice(end);
}
