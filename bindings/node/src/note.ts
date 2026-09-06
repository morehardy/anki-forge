import type { NoteOptions, ClozeOptions, ImageOcclusionOptions } from './types';
import { options, string, deepFreeze, strings } from './internal/validation';
import { Content, contentDefinition } from './content';
import { MediaRef, mediaFilename, mediaHandle } from './media';
import type { NativeMediaRef } from './internal/native';

type Source =
  | { kind: 'basic'; front: string; back: string }
  | { kind: 'cloze'; text: string }
  | { kind: 'custom'; id: string }
  | {
      kind: 'image_occlusion';
      image: MediaRef;
      mode: string;
      rects: ImageOcclusionOptions['rects'];
      header?: string;
      comments?: string;
    };
type Definition = {
  source: Source;
  options: ClozeOptions;
  fields: Readonly<Record<string, Content>>;
};
const definitions = new WeakMap<Note, Definition>();

function noteOptions(input: NoteOptions | ClozeOptions, cloze = false): ClozeOptions {
  options(
    input,
    ['stableId', 'deckName', 'tags', 'identity', ...(cloze ? ['backExtra'] : [])],
    'note',
  );
  for (const key of ['stableId', 'deckName', 'backExtra'])
    if (input[key] !== undefined) string(input[key], key);
  if (input.tags !== undefined) strings(input.tags, 'tags');
  if (input.identity !== undefined) strings(input.identity, 'identity');
  return deepFreeze({
    ...input,
    ...(input.tags ? { tags: [...input.tags] } : {}),
    ...(input.identity ? { identity: [...input.identity] } : {}),
  });
}

export class Note {
  private constructor(definition: Definition) {
    definitions.set(this, deepFreeze(definition));
    Object.freeze(this);
  }
  static basic(front: string, back: string, config: NoteOptions = {}): Note {
    string(front, 'front');
    string(back, 'back');
    return new Note({
      source: { kind: 'basic', front, back },
      options: noteOptions(config),
      fields: {},
    });
  }
  static cloze(text: string, config: ClozeOptions = {}): Note {
    string(text, 'text');
    return new Note({
      source: { kind: 'cloze', text },
      options: noteOptions(config, true),
      fields: {},
    });
  }
  static custom(id: string, config: NoteOptions = {}): Note {
    string(id, 'noteTypeId');
    return new Note({ source: { kind: 'custom', id }, options: noteOptions(config), fields: {} });
  }
  static imageOcclusion(image: MediaRef, config: ImageOcclusionOptions): Note {
    mediaFilename(image);
    options(
      config,
      [
        'stableId',
        'deckName',
        'tags',
        'identity',
        'backExtra',
        'rects',
        'mode',
        'header',
        'comments',
      ],
      'imageOcclusion',
    );
    const { rects, mode = 'hide-all-guess-one', header, comments, ...rest } = config;
    if (!Array.isArray(rects)) throw new TypeError('rects must be an array');
    for (const rect of rects) {
      options(rect, ['x', 'y', 'width', 'height'], 'rect');
      for (const key of ['x', 'y', 'width', 'height'])
        if (!Number.isInteger(rect[key]) || Number(rect[key]) < 0 || Number(rect[key]) > 0xffffffff)
          throw new TypeError(`rect.${key} must be a uint32`);
    }
    if (!['hide-all-guess-one', 'hide-one-guess-one'].includes(String(mode)))
      throw new TypeError('Invalid image occlusion mode');
    if (header !== undefined) string(header, 'header');
    if (comments !== undefined) string(comments, 'comments');
    return new Note({
      source: {
        kind: 'image_occlusion',
        image,
        mode: String(mode),
        rects: rects.map((rect) => ({ ...rect })),
        header,
        comments,
      },
      options: noteOptions(rest, true),
      fields: {},
    });
  }
  field(field: string, content: Content): Note {
    string(field, 'field');
    contentDefinition(content);
    const definition = definitions.get(this)!;
    return new Note({ ...definition, fields: { ...definition.fields, [field]: content } });
  }
  text(field: string, value: string): Note {
    return this.field(field, Content.text(value));
  }
  html(field: string, value: string): Note {
    return this.field(field, Content.html(value));
  }
  image(field: string, value: MediaRef): Note {
    return this.field(field, Content.image(value));
  }
  sound(field: string, value: MediaRef): Note {
    return this.field(field, Content.sound(value));
  }
}

export function noteDefinition(note: Note): { input: string; references: NativeMediaRef[] } {
  const value = definitions.get(note);
  if (!value) throw new TypeError('addNote requires a Note created by this SDK');
  const references: NativeMediaRef[] = [];
  if (value.source.kind === 'image_occlusion') references.push(mediaHandle(value.source.image));
  const source =
    value.source.kind === 'image_occlusion'
      ? { ...value.source, image: mediaFilename(value.source.image) }
      : value.source;
  const input = JSON.stringify({
    ...value,
    source,
    fields: Object.fromEntries(
      Object.entries(value.fields).map(([field, content]) => [
        field,
        contentDefinition(content, references),
      ]),
    ),
  });
  return { input, references };
}
