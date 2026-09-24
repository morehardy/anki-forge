import path from 'node:path';
import { repository, withBase } from '../site.config.mjs';

export const importedDocs = [
  {"source": "docs/rust-guide.md", "route": "/docs/rust-guide/", "name": "rust-guide", "title": "Rust authoring guide", "description": "Projects, validation, media and updates through the supported Rust API."},
  {"source": "docs/template-bundles.md", "route": "/docs/templates/", "name": "templates", "title": "Template bundles", "description": "Import complete reusable Cloze and normal templates, CSS and media."},
  {"source": "anki_forge/README.md", "route": "/docs/api/", "name": "api", "title": "Public API overview", "description": "Rust compatibility boundaries, artifact ownership and supported behavior."},
  {"source": "docs/installation.md", "route": "/docs/installation/", "name": "installation", "title": "Add to your application", "description": "Create your own Rust application and export its first Anki package."},
  {"source": "docs/concepts.md", "route": "/docs/concepts/", "name": "concepts", "title": "Core concepts", "description": "Projects, owned notes and media, stable keys, text and update identity."},
  {"source": "docs/cards.md", "route": "/docs/cards/", "name": "cards", "title": "Basic and Cloze cards", "description": "Build two notes and three cards with a complete verified Rust example."},
  {"source": "docs/custom-notetypes.md", "route": "/docs/custom-notetypes/", "name": "custom-notetypes", "title": "Custom note types", "description": "Declare fields, card templates and generation rules for your own content."},
  {"source": "docs/media.md", "route": "/docs/media/", "name": "media", "title": "Images and audio", "description": "Generate and package a real waveform image and a one-second audio tone."},
  {"source": "docs/image-occlusion.md", "route": "/docs/image-occlusion/", "name": "image-occlusion", "title": "Image Occlusion", "description": "Create rectangle-based image questions in both occlusion modes."},
  {"source": "docs/updates.md", "route": "/docs/updates/", "name": "updates", "title": "Update and distribute", "description": "Compare and build releases using verified original APKG baselines."},
  {"source": "docs/build-guarantees.md", "route": "/docs/build-guarantees/", "name": "build-guarantees", "title": "Build and output guarantees", "description": "Understand temporary artifacts, path protection and candidate publication."},
  {"source": "docs/troubleshooting.md", "route": "/docs/troubleshooting/", "name": "troubleshooting", "title": "Troubleshooting", "description": "Resolve installation, media, template, identity and output errors."},
  {"source": "docs/compatibility.md", "route": "/docs/compatibility/", "name": "compatibility", "title": "Compatibility and releases", "description": "Separate source versions, verified platforms and package publication."},
  {"source": "docs/rust-api.md", "route": "/docs/rust-api/", "name": "rust-api", "title": "Rust API guide", "description": "Find public authoring methods, build defaults, reports and diagnostic behavior."},
  {"source": "docs/node/quick-start.md", "route": "/docs/node-quickstart/", "name": "node-quickstart", "title": "Node quickstart", "description": "Build the native Node SDK and export two notes and three cards."},
  {"source": "docs/node/api.md", "route": "/docs/node-api/", "name": "node-api", "title": "Node and TypeScript API", "description": "Public methods, async behavior, build options and reports."},
  {"source": "docs/python/quick-start.md", "route": "/docs/python-quickstart/", "name": "python-quickstart", "title": "Python quickstart", "description": "Install the native Python SDK and build real text, image and audio examples."},
  {"source": "docs/python/api.md", "route": "/docs/python-api/", "name": "python-api", "title": "Python API", "description": "Project, media, build options, reports and temporary artifact ownership."},
  {"source": "docs/python/diagnostics.md", "route": "/docs/python-diagnostics/", "name": "python-diagnostics", "title": "Python diagnostics", "description": "Understand native authoring errors, build reports and update evidence."},
  {"source": "bindings/python/MIGRATION.md", "route": "/docs/python-migration/", "name": "python-migration", "title": "Python API transition", "description": "Use the current owned-value SDK and understand the clean-slate boundary."},
  {"source": "docs/python/genanki-migration.md", "route": "/docs/genanki-migration/", "name": "genanki-migration", "title": "Move from genanki", "description": "Understand content, template and media behavior when moving to Anki Forge."},
];

export function rewriteTarget(target, source, base) {
  if (/^(?:[a-z][a-z\d+.-]*:|\/\/|#)/i.test(target)) return target;
  const [, pathname, suffix] = target.match(/^([^?#]*)(.*)$/);
  const resolved = path.posix.normalize(path.posix.join(path.posix.dirname(source), pathname));
  const doc = importedDocs.find(entry => entry.source === resolved);
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
