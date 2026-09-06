import { Project, Note } from 'anki-forge-node';
import os from 'node:os';
import path from 'node:path';
import fs from 'node:fs/promises';

const baseDir = await fs.mkdtemp(path.join(os.tmpdir(), 'anki-forge-example-'));
const project = new Project('Spanish', {
  stableId: 'spanish-a1',
  defaultDeck: 'Spanish::A1',
  baseDir,
});
project.addNote(Note.basic('hola', 'hello', { stableId: 'es:hola' }));
project.addNote(Note.cloze('{{c1::uno}}, {{c2::dos}}', { stableId: 'es:numbers' }));
(await project.validate()).ensureSuccess();
const report = await project.writeApkg('spanish.apkg');
report.ensureSuccess();
console.log(report.prettyReport());
console.log(`APKG: ${report.artifact.path}`);
