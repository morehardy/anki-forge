import { canonicalUrl, entityIds, markdownUrl } from './machine-urls.mjs';

export function buildStructuredData(page, context) {
  const { siteConfig, repository, publicInstall, homeDescription } = context;
  const ids = entityIds(page, siteConfig);
  const root = ids.website.slice(0, -'#website'.length);
  const software = { '@id': ids.software, '@type': 'SoftwareApplication', name: 'Anki Forge' };
  const website = { '@id': ids.website, '@type': 'WebSite', name: 'Anki Forge', url: root, inLanguage: ['en', 'zh-CN'] };
  const collection = page.type === 'blog-list' || page.type === 'examples' || (page.type === 'docs' && page.id === 'docs');
  const webpage = {
    '@id': ids.webpage, '@type': collection ? 'CollectionPage' : 'WebPage', name: page.title,
    description: page.description, url: page.canonical, inLanguage: page.locale,
    isPartOf: { '@id': ids.website }, about: { '@id': ids.software },
  };
  const graph = [website, webpage];
  const isArticle = page.type === 'article' || (page.type === 'docs' && page.id !== 'docs');
  if (page.type === 'home') {
    webpage.mainEntity = { '@id': ids.software };
    Object.assign(software, {
      url: root, applicationCategory: 'DeveloperApplication', softwareVersion: publicInstall.publicVersion,
      license: `${repository}/blob/main/LICENSE`, softwareHelp: canonicalUrl('docs', page.locale, siteConfig),
      description: homeDescription,
    });
    graph.push(software, { '@id': ids.source, '@type': 'SoftwareSourceCode', name: 'Anki Forge', codeRepository: repository, license: `${repository}/blob/main/LICENSE`, targetProduct: { '@id': ids.software } });
  } else if (isArticle) {
    webpage.mainEntity = { '@id': ids.article };
    const article = {
      '@id': ids.article, '@type': page.type === 'article' ? 'BlogPosting' : 'TechArticle',
      headline: page.title, description: page.description, url: page.canonical, inLanguage: page.locale,
      mainEntityOfPage: { '@id': ids.webpage }, about: { '@id': ids.software },
      encoding: { '@type': 'MediaObject', contentUrl: markdownUrl(page), encodingFormat: 'text/markdown', inLanguage: page.locale },
    };
    if (page.type === 'article') {
      if (!page.publishedAt || !page.category) throw new Error(`${page.id}/${page.locale}: missing blog metadata`);
      article.datePublished = page.publishedAt;
      article.articleSection = page.category;
    }
    graph.push(article, software);
  } else graph.push(software);
  return { '@context': 'https://schema.org', '@graph': graph };
}

export function serializeJsonLd(graph) {
  return JSON.stringify(graph).replace(/</g, '\\u003c').replace(/\u2028/g, '\\u2028').replace(/\u2029/g, '\\u2029');
}
