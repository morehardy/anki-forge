import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { docGroups, docPages } from './site-model.mjs';
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
      locales: { root: { label: 'English', lang: 'en' }, 'zh-cn': { label: '简体中文', lang: 'zh-CN' } },
      defaultLocale: 'root',
      routeMiddleware: './src/route-middleware.ts',
      components: { Header: './src/components/DocsHeader.astro', Footer: './src/components/DocsFooter.astro', LanguageSelect: './src/components/LanguageSwitcher.astro', ThemeSelect: './src/components/ThemeControl.astro' },
      logo: { light: '../docs/assets/brand/ankiforge.svg', dark: '../docs/assets/brand/ankiforge-dark.svg', alt: '', replacesTitle: false },
      description: 'Build Basic, Cloze, Image Occlusion and custom Anki cards with Rust, TypeScript or Python.',
      favicon: '/favicon.svg',
      disable404Route: true,
      social: [{ icon: 'github', label: 'GitHub', href: repository }],
      editLink: { baseUrl: `${repository}/edit/main/website/` },
      customCss: ['./src/styles/docs.css'],
      head: [
        { tag: 'meta', attrs: { property: 'og:image', content: new URL(withBase('/generated/og.png', siteConfig.base), siteConfig.site).href } },
        { tag: 'meta', attrs: { property: 'og:image:alt', content: 'Anki Forge. Anki decks, built from code.' } },
        { tag: 'meta', attrs: { name: 'twitter:card', content: 'summary_large_image' } },
        {
          tag: 'script',
          attrs: {
            type: 'module',
            src: 'https://static.cloudflareinsights.com/beacon.min.js',
            'data-cf-beacon': JSON.stringify({ token: 'ab8936f615cc40e7a31bb7db4366b73b' }),
          },
        },
      ],
      sidebar: docGroups.map(group => ({
        label: group.label, translations: { 'zh-CN': group.zh }, collapsed: group.collapsed ?? false,
        items: group.ids.map(id => {
          const page = docPages.find(page => page.id === id);
          return { slug: page.en.slice(1, -1), label: page.title.en, translations: { 'zh-CN': page.title['zh-CN'] } };
        }),
      })),
    }),
  ],
});
