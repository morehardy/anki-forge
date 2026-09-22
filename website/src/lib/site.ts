export const repository = 'https://github.com/morehardy/anki-forge';
export const description = 'Generate and validate Anki decks with Rust. Keep note identities stable as your content evolves.';

export function sitePath(path: string): string {
  if (!path.startsWith('/') || path.startsWith('//')) throw new Error('Expected a site-relative path.');
  return `${import.meta.env.BASE_URL.replace(/\/$/, '')}${path}`;
}

export function formatDate(date: Date): string {
  return new Intl.DateTimeFormat('en', { year: 'numeric', month: 'long', day: 'numeric', timeZone: 'UTC' }).format(date);
}
