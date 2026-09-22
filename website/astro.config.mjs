import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { repository, siteConfig, withBase } from './site.config.mjs';

export default defineConfig({
  site: siteConfig.site,
  base: siteConfig.base,
  trailingSlash: 'always',
  output: 'static',
  devToolbar: { enabled: false },
  integrations: [
    starlight({
      title: 'Anki Forge',
      description: 'Generate and validate Anki decks with Rust. Keep note identities stable as your content evolves.',
      favicon: '/favicon.svg',
      disable404Route: true,
      social: [{ icon: 'github', label: 'GitHub', href: repository }],
      editLink: { baseUrl: `${repository}/edit/main/website/` },
      customCss: ['./src/styles/docs.css'],
      head: [
        { tag: 'meta', attrs: { property: 'og:image', content: new URL(withBase('/generated/og.png', siteConfig.base), siteConfig.site).href } },
        { tag: 'meta', attrs: { property: 'og:image:alt', content: 'Anki Forge. Anki decks, built from code.' } },
        { tag: 'meta', attrs: { name: 'twitter:card', content: 'summary_large_image' } },
      ],
      sidebar: [
        { label: 'Start here', items: [
          { label: 'Introduction', slug: 'docs' },
          { label: 'Choose your language', slug: 'docs/languages' },
          { label: 'First Rust deck', slug: 'docs/quickstart' },
          { label: 'Add to your application', slug: 'docs/installation' },
          { label: 'Core concepts', slug: 'docs/concepts' },
        ] },
        { label: 'Guides', items: [
          { label: 'Basic and Cloze', slug: 'docs/cards' },
          { label: 'Images and audio', slug: 'docs/media' },
          { label: 'Custom note types', slug: 'docs/custom-notetypes' },
          { label: 'Template bundles', slug: 'docs/templates' },
          { label: 'Image Occlusion', slug: 'docs/image-occlusion' },
          { label: 'Update and distribute', slug: 'docs/updates' },
        ] },
        { label: 'Language guides', items: [
          { label: 'Rust authoring', slug: 'docs/rust-guide' },
          { label: 'Node quickstart', slug: 'docs/node-quickstart' },
          { label: 'Python quickstart', slug: 'docs/python-quickstart' },
          { label: 'Python 0.1 to 0.2', slug: 'docs/python-migration' },
          { label: 'Move from genanki', slug: 'docs/genanki-migration' },
        ] },
        { label: 'Reference', collapsed: true, items: [
          { label: 'Public API overview', slug: 'docs/api' },
          { label: 'Rust API guide', slug: 'docs/rust-api' },
          { label: 'Node / TypeScript API', slug: 'docs/node-api' },
          { label: 'Python API', slug: 'docs/python-api' },
          { label: 'Build and output guarantees', slug: 'docs/build-guarantees' },
          { label: 'Compatibility and releases', slug: 'docs/compatibility' },
        ] },
        { label: 'Help', items: [
          { label: 'Troubleshooting', slug: 'docs/troubleshooting' },
          { label: 'Python diagnostics', slug: 'docs/python-diagnostics' },
          { label: 'Contributing', link: `${repository}/blob/main/docs/development.md` },
        ] },
      ],
    }),
  ],
});
