import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { siteConfig, withBase } from '../site.config.mjs';

export async function serveBuiltSite() {
  const dist = fileURLToPath(new URL('../dist/', import.meta.url));
  const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.json': 'application/json', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.png': 'image/png', '.wav': 'audio/wav', '.woff2': 'font/woff2', '.xml': 'application/xml' };
  const server = createServer(async (request, response) => {
    try {
      const url = new URL(request.url, 'http://localhost');
      const prefix = siteConfig.base === '/' ? '' : siteConfig.base;
      if (!url.pathname.startsWith(prefix + '/')) throw new Error('Outside site base');
      let relative = decodeURIComponent(url.pathname.slice(prefix.length));
      if (relative.endsWith('/')) relative += 'index.html';
      const file = path.resolve(dist, '.' + relative);
      if (!file.startsWith(dist)) throw new Error('Outside artifact');
      const bytes = await readFile(file);
      response.writeHead(200, { 'Content-Type': types[path.extname(file)] ?? 'application/octet-stream' });
      response.end(bytes);
    } catch {
      response.writeHead(404, { 'Content-Type': 'text/html' });
      response.end(await readFile(path.join(dist, '404.html')));
    }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const origin = `http://127.0.0.1:${server.address().port}`;
  return { origin, url: route => origin + withBase(route, siteConfig.base), close: () => new Promise(resolve => server.close(resolve)) };
}
