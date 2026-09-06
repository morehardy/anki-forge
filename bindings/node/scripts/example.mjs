import path from 'node:path';
import { root, targets } from './platforms.mjs';
const platform = targets.find((item) => item.os === process.platform && item.cpu === process.arch);
if (!platform) throw new Error('Unsupported example platform');
process.env.ANKI_FORGE_NATIVE_PATH = path.join(root, 'npm', platform.suffix, 'anki-forge.node');
await import('../examples/basic.mjs');
