export const repository = 'https://github.com/morehardy/anki-forge';
export const description = 'Build Basic, Cloze, Image Occlusion and custom Anki cards with Rust, TypeScript or Python. Package media and review updates.';

export function sitePath(path: string): string {
  if (!path.startsWith('/') || path.startsWith('//')) throw new Error('Expected a site-relative path.');
  return `${import.meta.env.BASE_URL.replace(/\/$/, '')}${path}`;
}

export function formatDate(date: Date): string {
  return new Intl.DateTimeFormat('en', { year: 'numeric', month: 'long', day: 'numeric', timeZone: 'UTC' }).format(date);
}
