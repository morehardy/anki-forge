import { native } from './internal/native';
import { string } from './internal/validation';
import { MediaRef, mediaFilename, mediaHandle } from './media';
import type { NativeMediaRef } from './internal/native';

type ContentDefinition =
  | { kind: 'text' | 'html'; value: string }
  | { kind: 'image' | 'sound'; value: MediaRef };
const definitions = new WeakMap<Content, ContentDefinition>();
export class Content {
  private constructor(definition: ContentDefinition) {
    definitions.set(this, Object.freeze(definition));
    Object.freeze(this);
  }
  static text(value: string): Content {
    string(value, 'text');
    return new Content({ kind: 'text', value });
  }
  static html(value: string): Content {
    string(value, 'html');
    return new Content({ kind: 'html', value });
  }
  static image(value: MediaRef): Content {
    mediaFilename(value);
    return new Content({ kind: 'image', value });
  }
  static sound(value: MediaRef): Content {
    mediaFilename(value);
    return new Content({ kind: 'sound', value });
  }
  render(): string {
    const definition = definitions.get(this)!;
    if (definition.kind === 'image') return mediaHandle(definition.value).renderImage();
    if (definition.kind === 'sound') return mediaHandle(definition.value).renderSound();
    return native().renderContent(String(definition.value), definition.kind === 'html');
  }
}
export function contentDefinition(
  content: Content,
  references?: NativeMediaRef[],
): { kind: string; value: string } {
  const definition = definitions.get(content);
  if (!definition) throw new TypeError('Expected Content created by this SDK');
  if (typeof definition.value !== 'string') references?.push(mediaHandle(definition.value));
  return {
    kind: definition.kind,
    value:
      typeof definition.value === 'string' ? definition.value : mediaFilename(definition.value),
  };
}
