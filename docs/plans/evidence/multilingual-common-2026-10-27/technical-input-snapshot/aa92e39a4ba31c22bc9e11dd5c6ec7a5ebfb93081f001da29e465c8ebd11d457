import showcase from '../generated/showcase.json';
import basic from '../generated/basic.rs?raw';
import cloze from '../generated/cloze.rs?raw';
import media from '../generated/media.rs?raw';
import occlusion from '../generated/occlusion.rs?raw';
import updates from '../generated/updates.rs?raw';
import quickstartSource from '../../../anki_forge/examples/target_api_basic.rs?raw';

const snippets = new Map([['basic', basic], ['cloze', cloze], ['media', media], ['occlusion', occlusion]]);
export const examples = showcase.examples.map(example => {
  const source = snippets.get(example.id);
  if (!source) throw new Error(`Missing compiled example source: ${example.id}`);
  const clozeParts = example.id === 'cloze' ? example.front.match(/^(.*?)\{\{c1::(.*?)\}\}(.*)$/) : null;
  if (example.id === 'cloze' && !clozeParts) throw new Error(`Missing cloze deletion: ${example.id}`);
  const { mask, imageWidth, imageHeight } = example;
  let maskStyle: string | undefined;
  if (mask) {
    if (!imageWidth || !imageHeight) throw new Error(`Missing image dimensions: ${example.id}`);
    maskStyle = `left:${mask.x / imageWidth * 100}%;top:${mask.y / imageHeight * 100}%;width:${mask.width / imageWidth * 100}%;height:${mask.height / imageHeight * 100}%`;
  }
  return { ...example, source, maskStyle, clozeParts };
});
export type CardExample = (typeof examples)[number];
export const updateExample = { ...showcase.update, source: updates };
export { quickstartSource, showcase };
