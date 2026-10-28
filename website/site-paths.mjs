// URL path handling is pure so it can also be used by schema construction.
export function withBase(path, base) {
  if (!path.startsWith('/') || path.startsWith('//')) throw new Error('Expected a site-relative path.');
  return `${base === '/' ? '' : base}${path}`;
}
