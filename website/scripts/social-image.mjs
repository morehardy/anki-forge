import { copyFile, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import sharp from 'sharp';

const escape = value => String(value).replace(/[<>&"']/g, character => ({ '<': '&lt;', '>': '&gt;', '&': '&amp;', '"': '&quot;', "'": '&apos;' })[character]);

export async function prepareBrandAssets(website, example) {
  const icon = await readFile(path.join(website, 'node_modules/@phosphor-icons/core/assets/regular/cards.svg'), 'utf8');
  const favicon = icon.replaceAll('currentColor', '#116e60');
  await writeFile(path.join(website, 'public/favicon.svg'), favicon);
  await copyFile(path.join(website, 'node_modules/@phosphor-icons/core/LICENSE'), path.join(website, 'public/generated/phosphor-license.txt'));
  // A share graphic drawn from the same fields as the compiled Basic deck.
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630">
    <rect width="1200" height="630" fill="#f8faf9"/>
    <g font-family="Arial, DejaVu Sans, sans-serif">
      <text x="64" y="84" fill="#116e60" font-size="30" font-weight="700">ankiforge</text>
      <text x="64" y="245" fill="#142d29" font-size="70" font-weight="700" letter-spacing="-3">Anki decks,</text>
      <text x="64" y="329" fill="#116e60" font-size="70" font-weight="700" letter-spacing="-3">built from code.</text>
      <text x="68" y="412" fill="#536b65" font-size="25">Write notes. Build decks. Keep their identity.</text>
      <rect x="764" y="154" width="370" height="300" rx="16" fill="#edf3f1" stroke="#d9e4df"/>
      <rect x="790" y="182" width="318" height="244" rx="10" fill="#fdfefd" stroke="#d9e4df"/>
      <text x="949" y="229" text-anchor="middle" fill="#536b65" font-size="17">Spanish → English</text>
      <text x="949" y="310" text-anchor="middle" fill="#142d29" font-size="55">${escape(example.front)}</text>
      <text x="949" y="365" text-anchor="middle" fill="#116e60" font-size="30">${escape(example.back)}</text>
      <path d="M64 530H1136" stroke="#d9e4df"/>
      <text x="68" y="576" fill="#536b65" font-size="19">Open source · Rust · Anki</text>
    </g>
  </svg>`;
  await sharp(Buffer.from(svg)).png().toFile(path.join(website, 'public/generated/og.png'));
}
