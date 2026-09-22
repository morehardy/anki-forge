import showcase from '../generated/showcase.json';
import basic from '../generated/basic.rs?raw';
import cloze from '../generated/cloze.rs?raw';
import media from '../generated/media.rs?raw';
import updates from '../generated/updates.rs?raw';
import quickstartSource from '../../../anki_forge/examples/target_api_basic.rs?raw';

const snippets = new Map([['basic', basic], ['cloze', cloze], ['media', media]]);
export const examples = showcase.examples.map(example => {
  const source = snippets.get(example.id);
  if (!source) throw new Error(`Missing compiled example source: ${example.id}`);
  return { ...example, source };
});
export type CardExample = (typeof examples)[number];
export const updateExample = { ...showcase.update, source: updates };
export { quickstartSource, showcase };
