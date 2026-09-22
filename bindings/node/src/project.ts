import path from 'node:path';
import type { ProjectOptions } from './types';
import { Buildable } from './buildable';
import { native, type NativeBuildable, type NativeProject } from './internal/native';
import { options, string } from './internal/validation';
import { Note, noteDefinition } from './note';
import { nativeError } from './errors';
import { outcome } from './internal/outcome';
import { NoteType, noteTypeDefinition } from './notetype';
import { MediaRegistry } from './media';
import { Deck, deckBackend } from './deck';

const adoptions = new WeakMap<ProjectOptions, NativeProject>();

export class Project extends Buildable {
  readonly baseDir: string;
  readonly name: string;
  readonly media: MediaRegistry;
  #project: InstanceType<ReturnType<typeof native>['NativeProject']>;

  constructor(name: string, config: ProjectOptions = {}) {
    super();
    string(name, 'name');
    options(config, ['stableId', 'defaultDeck', 'baseDir'], 'project');
    for (const key of ['stableId', 'defaultDeck', 'baseDir'])
      if (config[key] !== undefined) string(config[key], key);
    const baseDir = config.baseDir ?? process.cwd();
    string(baseDir, 'baseDir');
    this.name = name;
    this.baseDir = path.resolve(baseDir);
    this.#project =
      adoptions.get(config) ??
      new (native().NativeProject)(
        name,
        JSON.stringify({ stableId: config.stableId, defaultDeck: config.defaultDeck }),
      );
    this.media = new MediaRegistry(this.#project, this.baseDir);
    Object.freeze(this);
  }

  static async fromDeck(deck: Deck): Promise<Project> {
    try {
      const backend = await deckBackend(deck).toProject();
      const config: ProjectOptions = { baseDir: deck.baseDir };
      adoptions.set(config, backend);
      return new Project(deck.name, config);
    } catch (error) {
      nativeError(error);
    }
  }

  async clone(): Promise<Project> {
    try {
      const backend = await this.#project.cloneState();
      const config: ProjectOptions = { baseDir: this.baseDir };
      adoptions.set(config, backend);
      return new Project(this.name, config);
    } catch (error) {
      nativeError(error);
    }
  }

  addNote(note: Note): void {
    const { input, references } = noteDefinition(note);
    try {
      outcome(this.#project.addNote(input, references));
    } catch (error) {
      nativeError(error);
    }
  }

  addNoteType(noteType: NoteType): void {
    try {
      outcome(this.#project.addNoteType(noteTypeDefinition(noteType)));
    } catch (error) {
      nativeError(error);
    }
  }

  async importTemplateBundle(directory: string): Promise<void> {
    string(directory, 'directory');
    try {
      outcome(await this.#project.importTemplateBundle(path.resolve(this.baseDir, directory)));
    } catch (error) {
      nativeError(error);
    }
  }

  protected nativeProject(): NativeBuildable {
    return this.#project;
  }
}
