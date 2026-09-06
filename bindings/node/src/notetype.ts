import { deepFreeze, options, string, strings } from './internal/validation';
import { native } from './internal/native';
import { outcome } from './internal/outcome';
import { ValidationReport } from './report';

type Definition = Record<string, unknown>;
const definitions = new WeakMap<object, Definition>();
function definition(value: object, expected: Function): Definition {
  if (!(value instanceof expected) || !definitions.has(value))
    throw new TypeError(`Expected ${expected.name} created by this SDK`);
  return definitions.get(value)!;
}
function save(value: object, data: Definition): void {
  definitions.set(value, deepFreeze(data));
  Object.freeze(value);
}

export interface FieldOptions {
  key?: string;
  identity?: boolean;
  sort?: boolean;
  required?: boolean;
  optional?: boolean;
}
export class Field {
  #brand = undefined;
  constructor(name: string, config: FieldOptions = {}) {
    string(name, 'field name');
    options(config, ['key', 'identity', 'sort', 'required', 'optional'], 'field');
    if (config.key !== undefined) string(config.key, 'key');
    for (const key of ['identity', 'sort', 'required', 'optional'])
      if (config[key] !== undefined && typeof config[key] !== 'boolean')
        throw new TypeError(`${key} must be boolean`);
    if (config.required && config.optional)
      throw new TypeError('A field cannot be both required and optional');
    save(this, { name, ...config });
  }
}
export class IdentityRecipe {
  #brand = undefined;
  private constructor(fields: readonly string[]) {
    save(this, { fields: [...fields] });
  }
  static fields(fields: readonly string[]): IdentityRecipe {
    strings(fields, 'identity fields');
    return new IdentityRecipe(fields);
  }
}
export class GenerationRule {
  #brand = undefined;
  private constructor(data: Definition) {
    save(this, data);
  }
  static ankiDefault(): GenerationRule {
    return new GenerationRule({ kind: 'anki_default' });
  }
  static all(fields: readonly string[]): GenerationRule {
    strings(fields, 'fields');
    return new GenerationRule({ kind: 'all', fields: [...fields] });
  }
  static any(fields: readonly string[]): GenerationRule {
    strings(fields, 'fields');
    return new GenerationRule({ kind: 'any', fields: [...fields] });
  }
  static cloze(field: string): GenerationRule {
    string(field, 'field');
    return new GenerationRule({ kind: 'cloze', field });
  }
}
export interface TemplateOptions {
  key?: string;
  front: string;
  back: string;
  browserFront?: string;
  browserBack?: string;
  targetDeck?: string;
  generateWhen?: GenerationRule;
}
export class Template {
  #brand = undefined;
  constructor(name: string, config: TemplateOptions) {
    string(name, 'template name');
    options(
      config,
      ['key', 'front', 'back', 'browserFront', 'browserBack', 'targetDeck', 'generateWhen'],
      'template',
    );
    string(config.front, 'front');
    string(config.back, 'back');
    for (const key of ['key', 'browserFront', 'browserBack', 'targetDeck'])
      if (config[key] !== undefined) string(config[key], key);
    save(this, {
      name,
      ...config,
      generateWhen:
        config.generateWhen === undefined
          ? undefined
          : definition(config.generateWhen, GenerationRule),
    });
  }
}
export interface NoteTypeOptions {
  name?: string;
  fields: readonly Field[];
  templates: readonly Template[];
  css?: string;
  identity?: IdentityRecipe;
}
export class NoteType {
  #brand = undefined;
  private constructor(id: string, config: NoteTypeOptions, clozeField?: string) {
    string(id, 'note type id');
    options(config, ['name', 'fields', 'templates', 'css', 'identity'], 'note type');
    for (const key of ['name', 'css']) if (config[key] !== undefined) string(config[key], key);
    if (!Array.isArray(config.fields) || !Array.isArray(config.templates))
      throw new TypeError('fields and templates must be arrays');
    save(this, {
      id,
      clozeField,
      ...config,
      fields: config.fields.map((value) => definition(value, Field)),
      templates: config.templates.map((value) => definition(value, Template)),
      identity:
        config.identity === undefined
          ? undefined
          : definition(config.identity, IdentityRecipe).fields,
    });
  }
  static custom(id: string, config: NoteTypeOptions): NoteType {
    return new NoteType(id, config);
  }
  static customCloze(id: string, field: string, config: NoteTypeOptions): NoteType {
    string(field, 'cloze field');
    return new NoteType(id, config, field);
  }
}
export function noteTypeDefinition(noteType: NoteType): string {
  return JSON.stringify(definition(noteType, NoteType));
}
export function validateTemplate(source: string, fields: readonly string[]): ValidationReport {
  string(source, 'source');
  strings(fields, 'fields');
  return new ValidationReport(outcome(native().validateTemplate(source, [...fields])).diagnostics);
}
