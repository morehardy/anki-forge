import path from "node:path";
import {
  native,
  type NativeContent,
  type NativeMedia,
  type NativeNote,
  type NativeNoteType,
  type NativeProject,
} from "./internal/native";
import { call, asyncCall } from "./errors";
import { deepFreeze, string } from "./internal/validation";
import { ApkgArtifact, artifactFromNative } from "./artifact";
import type {
  BuildSnapshot,
  ReportSnapshot,
  ComparisonSnapshot,
  RiskLevel,
  RiskCode,
} from "./snapshots";
export { ApkgArtifact } from "./artifact";
export {
  NativeLoadError,
  ForgeError,
  SchemaError,
  AddError,
  MediaError,
  TemplateBundleError,
  ImageOcclusionError,
  PolicyError,
  ConfigurationError,
  CompareError,
  BuildError,
  PersistError,
  ArtifactClosedError,
} from "./errors";
export type * from "./snapshots";
export type { ErrorSourceDetail } from "./errors";
export { bindingMetadata } from "./internal/native";
export type { BindingMetadata } from "./internal/native";
export type ContentLike = string | Content;
const contentHandles = new WeakMap<Content, NativeContent>();
function content(value: ContentLike): NativeContent {
  if (typeof value === "string") return native().NativeContent.text(value);
  const v = contentHandles.get(value);
  if (!v) throw new TypeError("Expected string or Content");
  return v;
}
let makeContent: (h: NativeContent) => Content;
export class Content {
  private constructor(handle: NativeContent) {
    contentHandles.set(this, handle);
    Object.freeze(this);
  }
  static text(value: string): Content {
    string(value, "text");
    return new Content(call(() => native().NativeContent.text(value)));
  }
  static html(value: string): Content {
    string(value, "html");
    return new Content(call(() => native().NativeContent.html(value)));
  }
  static sequence(values: Iterable<ContentLike>): Content {
    return new Content(
      call(() => native().NativeContent.sequence(Array.from(values, content))),
    );
  }
  static {
    makeContent = (h) => new Content(h);
  }
}
export interface MediaLimits {
  readonly maxBytes?: number | bigint;
}
const mediaHandles = new WeakMap<Media, NativeMedia>();
function media(value: Media): NativeMedia {
  const v = mediaHandles.get(value);
  if (!v) throw new TypeError("Expected Media");
  return v;
}
function limitsJSON(value: object): string {
  for (const [key, n] of Object.entries(value)) {
    if (typeof n === "bigint") {
      if (n < 0n || n > 18446744073709551615n)
        throw new TypeError(`${key} must fit u64`);
    } else if (typeof n !== "number" || !Number.isSafeInteger(n) || n < 0)
      throw new TypeError(
        `${key} must be a nonnegative safe integer or bigint`,
      );
  }
  return JSON.stringify(value, (_key, n) =>
    typeof n === "bigint" ? n.toString() : n,
  );
}
export class Media {
  private constructor(handle: NativeMedia) {
    mediaHandles.set(this, handle);
    Object.freeze(this);
  }
  static async file(
    filename: string,
    limits: MediaLimits = {},
  ): Promise<Media> {
    string(filename, "filename");
    return new Media(
      await asyncCall(() =>
        native().NativeMedia.file(path.resolve(filename), limitsJSON(limits)),
      ),
    );
  }
  static async bytes(
    bytes: Uint8Array,
    mediaType: string,
    limits: MediaLimits = {},
  ): Promise<Media> {
    if (!(bytes instanceof Uint8Array))
      throw new TypeError("Expected Uint8Array");
    string(mediaType, "mediaType");
    return new Media(
      await asyncCall(() =>
        native().NativeMedia.bytes(
          Buffer.from(bytes.buffer, bytes.byteOffset, bytes.byteLength),
          mediaType,
          limitsJSON(limits),
        ),
      ),
    );
  }
  withExportName(name: string): Media {
    string(name, "name");
    return new Media(call(() => media(this).withExportName(name)));
  }
  get filename(): string {
    return media(this).filename;
  }
  get mediaType(): string {
    return media(this).mediaType;
  }
  get byteLength(): number {
    return media(this).byteLength;
  }
  image(): Content {
    return makeContent(media(this).image());
  }
  sound(): Content {
    return makeContent(media(this).sound());
  }
}
export interface FieldOptions {
  readonly name?: string;
  readonly required?: boolean;
  readonly sort?: boolean;
}
export class Field {
  readonly #brand = true;
  readonly key: string;
  readonly options: FieldOptions;
  constructor(key: string, options: FieldOptions = {}) {
    string(key, "field key");
    this.key = key;
    this.options = deepFreeze({ ...options });
    Object.freeze(this);
  }
}
export type GenerationRuleValue =
  | { readonly kind: "anki_default" }
  | { readonly kind: "all" | "any"; readonly fields: readonly string[] };
export class GenerationRule {
  static ankiDefault(): GenerationRuleValue {
    return Object.freeze({ kind: "anki_default" });
  }
  static all(fields: Iterable<string>): GenerationRuleValue {
    return deepFreeze({ kind: "all", fields: [...fields] });
  }
  static any(fields: Iterable<string>): GenerationRuleValue {
    return deepFreeze({ kind: "any", fields: [...fields] });
  }
}
export interface TemplateOptions {
  readonly name?: string;
  readonly front: string;
  readonly back: string;
  readonly browserFront?: string;
  readonly browserBack?: string;
  readonly targetDeck?: string;
  readonly generation?: GenerationRuleValue;
}
export class Template {
  readonly #brand = true;
  readonly key: string;
  readonly options: TemplateOptions;
  constructor(key: string, options: TemplateOptions) {
    string(key, "template key");
    this.key = key;
    this.options = deepFreeze({ ...options });
    Object.freeze(this);
  }
}
interface ModelData {
  key: string;
  name?: string;
  fields: object[];
  templates: object[];
  css?: string;
  clozeField?: string;
}
const modelHandles = new WeakMap<NoteType, NativeNoteType>();
let makeModel: (h: NativeNoteType) => NoteType;
export class NoteType {
  private constructor(handle: NativeNoteType) {
    modelHandles.set(this, handle);
    Object.freeze(this);
  }
  static builder(key: string): NoteTypeBuilder {
    string(key, "model key");
    return new NoteTypeBuilder({ key, fields: [], templates: [] }, []);
  }
  static async fromBundle(
    filename: string,
    limits: MediaLimits = {},
  ): Promise<NoteType> {
    string(filename, "filename");
    return new NoteType(
      await asyncCall(() =>
        native().NativeNoteType.fromBundle(
          path.resolve(filename),
          limitsJSON(limits),
        ),
      ),
    );
  }
  get key(): string {
    return modelHandles.get(this)!.key;
  }
  get displayName(): string {
    return modelHandles.get(this)!.displayName;
  }
  note(): Note {
    return makeNote(modelHandles.get(this)!.note());
  }
  static {
    makeModel = (h) => new NoteType(h);
  }
}
export class NoteTypeBuilder {
  readonly #data: ModelData;
  readonly #assets: readonly Media[];
  /** Use NoteType.builder(key). */
  constructor(data: ModelData, assets: readonly Media[]) {
    this.#data = deepFreeze(data);
    this.#assets = Object.freeze([...assets]);
    Object.freeze(this);
  }
  #with(change: Partial<ModelData>): NoteTypeBuilder {
    return new NoteTypeBuilder({ ...this.#data, ...change }, this.#assets);
  }
  name(value: string): NoteTypeBuilder {
    return this.#with({ name: value });
  }
  field(value: Field): NoteTypeBuilder {
    if (!(value instanceof Field)) throw new TypeError("Expected Field");
    return this.#with({
      fields: [...this.#data.fields, { ...value.options, key: value.key }],
    });
  }
  template(value: Template): NoteTypeBuilder {
    if (!(value instanceof Template)) throw new TypeError("Expected Template");
    return this.#with({
      templates: [
        ...this.#data.templates,
        { ...value.options, key: value.key },
      ],
    });
  }
  css(value: string): NoteTypeBuilder {
    return this.#with({ css: value });
  }
  clozeField(key: string): NoteTypeBuilder {
    return this.#with({ clozeField: key });
  }
  asset(value: Media): NoteTypeBuilder {
    media(value);
    return new NoteTypeBuilder(this.#data, [...this.#assets, value]);
  }
  build(): NoteType {
    return makeModel(
      call(() =>
        native().NativeNoteType.build(
          JSON.stringify(this.#data),
          this.#assets.map(media),
        ),
      ),
    );
  }
}
const noteHandles = new WeakMap<Note, NativeNote>();
let makeNote: (h: NativeNote) => Note;
function note(value: Note): NativeNote {
  const v = noteHandles.get(value);
  if (!v) throw new TypeError("Expected Note");
  return v;
}
export class Note {
  private constructor(handle: NativeNote) {
    noteHandles.set(this, handle);
    Object.freeze(this);
  }
  static basic(front: ContentLike, back: ContentLike): Note {
    return new Note(
      call(() => native().NativeNote.basic(content(front), content(back))),
    );
  }
  static cloze(text: ContentLike): Note {
    return new Note(call(() => native().NativeNote.cloze(content(text))));
  }
  static imageOcclusion(image: Media): ImageOcclusionBuilder {
    media(image);
    return new ImageOcclusionBuilder(image);
  }
  field(key: string, value: ContentLike): Note {
    string(key, "field key");
    return new Note(call(() => note(this).field(key, content(value))));
  }
  deck(name: string): Note {
    string(name, "deck");
    return new Note(call(() => note(this).deck(name)));
  }
  tag(value: string): Note {
    string(value, "tag");
    return new Note(call(() => note(this).tag(value)));
  }
  tags(values: Iterable<string>): Note {
    let result: Note = this;
    for (const v of values) result = result.tag(v);
    return result;
  }
  static {
    makeNote = (h) => new Note(h);
  }
}
export class Mask {
  private constructor(
    readonly key: string,
    readonly x: number,
    readonly y: number,
    readonly width: number,
    readonly height: number,
  ) {
    Object.freeze(this);
  }
  static rect(
    key: string,
    x: number,
    y: number,
    width: number,
    height: number,
  ): Mask {
    string(key, "mask key");
    for (const n of [x, y, width, height])
      if (typeof n !== "number")
        throw new TypeError("Mask coordinates must be numbers");
    return new Mask(key, x, y, width, height);
  }
}
export type OcclusionMode = "hide_all_guess_one" | "hide_one_guess_one";
export class ImageOcclusionBuilder {
  readonly #image: Media;
  readonly #masks: readonly Mask[];
  readonly #mode: OcclusionMode;
  constructor(
    image: Media,
    masks: readonly Mask[] = [],
    mode: OcclusionMode = "hide_all_guess_one",
  ) {
    this.#image = image;
    this.#masks = Object.freeze([...masks]);
    this.#mode = mode;
    Object.freeze(this);
  }
  mask(value: Mask): ImageOcclusionBuilder {
    if (!(value instanceof Mask)) throw new TypeError("Expected Mask");
    return new ImageOcclusionBuilder(
      this.#image,
      [...this.#masks, value],
      this.#mode,
    );
  }
  mode(value: OcclusionMode): ImageOcclusionBuilder {
    if (!["hide_all_guess_one", "hide_one_guess_one"].includes(value))
      throw new TypeError("Unknown occlusion mode");
    return new ImageOcclusionBuilder(this.#image, this.#masks, value);
  }
  build(): Note {
    return makeNote(
      call(() =>
        native().NativeNote.imageOcclusion(
          media(this.#image),
          this.#masks,
          this.#mode,
        ),
      ),
    );
  }
}
export interface InspectLimits {
  readonly maxArchiveBytes?: number | bigint;
  readonly maxEntries?: number | bigint;
  readonly maxCentralDirectoryBytes?: number | bigint;
  readonly maxZipEntryBytes?: number | bigint;
  readonly maxZipTotalBytes?: number | bigint;
  readonly maxMetaBytes?: number | bigint;
  readonly maxMediaMapBytes?: number | bigint;
  readonly maxIdentityBytes?: number | bigint;
  readonly maxCollectionBytes?: number | bigint;
  readonly maxMediaBytes?: number | bigint;
  readonly maxDecodedTotalBytes?: number | bigint;
  readonly maxZstdWindowBytes?: number | bigint;
}
export class UpdatePolicy {
  readonly #level: RiskLevel;
  readonly #allow: readonly RiskCode[];
  constructor(level: RiskLevel = "high", allow: readonly RiskCode[] = []) {
    this.#level = level;
    this.#allow = Object.freeze([...allow]);
    Object.freeze(this);
  }
  failOn(level: RiskLevel): UpdatePolicy {
    return new UpdatePolicy(level, this.#allow);
  }
  allow(code: RiskCode): UpdatePolicy {
    return new UpdatePolicy(this.#level, [...this.#allow, code]);
  }
  /** @internal */ toJSON(): object {
    return { failOn: this.#level, allow: this.#allow };
  }
}
interface BuildData {
  output?: string;
  temporary: boolean;
  updateFrom?: string;
  inspectLimits?: InspectLimits;
  updatePolicy?: UpdatePolicy;
}
export class BuildOptions {
  readonly #data: BuildData;
  private constructor(data: BuildData) {
    this.#data = deepFreeze(data);
    Object.freeze(this);
  }
  static to(filename: string): BuildOptions {
    string(filename, "output");
    return new BuildOptions({
      output: path.resolve(filename),
      temporary: false,
    });
  }
  static temporary(): BuildOptions {
    return new BuildOptions({ temporary: true });
  }
  updateFrom(filename: string): BuildOptions {
    string(filename, "baseline");
    return new BuildOptions({
      ...this.#data,
      updateFrom: path.resolve(filename),
    });
  }
  inspectLimits(limits: InspectLimits): BuildOptions {
    limitsJSON(limits);
    return new BuildOptions({ ...this.#data, inspectLimits: { ...limits } });
  }
  updatePolicy(policy: UpdatePolicy): BuildOptions {
    if (!(policy instanceof UpdatePolicy))
      throw new TypeError("Expected UpdatePolicy");
    return new BuildOptions({ ...this.#data, updatePolicy: policy });
  }
  /** @internal */ toJSON(): object {
    return {
      ...this.#data,
      inspectLimits: this.#data.inspectLimits
        ? JSON.parse(limitsJSON(this.#data.inspectLimits))
        : undefined,
    };
  }
}
interface CompareData {
  baseline: string;
  inspectLimits?: InspectLimits;
  updatePolicy?: UpdatePolicy;
}
export class CompareOptions {
  readonly #data: CompareData;
  private constructor(data: CompareData) {
    this.#data = deepFreeze(data);
    Object.freeze(this);
  }
  static against(filename: string): CompareOptions {
    string(filename, "baseline");
    return new CompareOptions({ baseline: path.resolve(filename) });
  }
  inspectLimits(limits: InspectLimits): CompareOptions {
    limitsJSON(limits);
    return new CompareOptions({ ...this.#data, inspectLimits: { ...limits } });
  }
  updatePolicy(policy: UpdatePolicy): CompareOptions {
    if (!(policy instanceof UpdatePolicy))
      throw new TypeError("Expected UpdatePolicy");
    return new CompareOptions({ ...this.#data, updatePolicy: policy });
  }
  /** @internal */ toJSON(): object {
    return {
      ...this.#data,
      inspectLimits: this.#data.inspectLimits
        ? JSON.parse(limitsJSON(this.#data.inspectLimits))
        : undefined,
    };
  }
}
export class BuildReport {
  readonly #snapshot: ReportSnapshot;
  constructor(snapshot: ReportSnapshot) {
    this.#snapshot = deepFreeze(snapshot);
    Object.freeze(this);
  }
  get counts() {
    return this.#snapshot.counts;
  }
  get baselineCounts() {
    return this.#snapshot.baseline_counts;
  }
  get durationMs() {
    return this.#snapshot.duration_ms;
  }
  get diagnostics() {
    return this.#snapshot.diagnostics;
  }
  get comparison() {
    return this.#snapshot.comparison;
  }
  snapshot(): ReportSnapshot {
    return this.#snapshot;
  }
}
export class ComparisonReport {
  readonly #snapshot: ComparisonSnapshot;
  constructor(snapshot: ComparisonSnapshot) {
    this.#snapshot = deepFreeze(snapshot);
    Object.freeze(this);
  }
  get findings() {
    return this.#snapshot.findings;
  }
  get highestRisk() {
    return this.#snapshot.highest_risk;
  }
  get policy() {
    return this.#snapshot.policy;
  }
  get diagnostics() {
    return this.#snapshot.diagnostics;
  }
  snapshot(): ComparisonSnapshot {
    return this.#snapshot;
  }
}
const outputToken = Symbol("BuildOutput");
let makeOutput: (
  artifact: ApkgArtifact,
  snapshot: BuildSnapshot,
) => BuildOutput;
export class BuildOutput {
  readonly artifact: ApkgArtifact;
  readonly report: BuildReport;
  readonly #snapshot: BuildSnapshot;
  private constructor(
    token: symbol,
    artifact: ApkgArtifact,
    snapshot: BuildSnapshot,
  ) {
    if (token !== outputToken)
      throw new TypeError("BuildOutput is produced only by Project.build");
    this.artifact = artifact;
    this.#snapshot = deepFreeze(snapshot);
    this.report = new BuildReport(snapshot.report);
    Object.freeze(this);
  }
  snapshot(): BuildSnapshot {
    return this.#snapshot;
  }
  static {
    makeOutput = (a, s) => new BuildOutput(outputToken, a, s);
  }
}
export class Project {
  readonly namespace: string;
  #handle: NativeProject;
  constructor(namespace: string) {
    string(namespace, "namespace");
    this.namespace = namespace;
    this.#handle = call(() => new (native().NativeProject)(namespace));
    Object.freeze(this);
  }
  name(value: string): this {
    string(value, "name");
    call(() => this.#handle.name(value));
    return this;
  }
  defaultDeck(value: string): this {
    string(value, "defaultDeck");
    call(() => this.#handle.defaultDeck(value));
    return this;
  }
  get length(): number {
    return this.#handle.length;
  }
  add(key: string, value: Note): this {
    string(key, "note key");
    call(() => this.#handle.add(key, note(value)));
    return this;
  }
  addAsset(value: Media): this {
    call(() => this.#handle.addAsset(media(value)));
    return this;
  }
  clone(): Project {
    const p = new Project(this.namespace);
    p.#handle = this.#handle.cloneState();
    return p;
  }
  async build(options: BuildOptions): Promise<BuildOutput> {
    if (!(options instanceof BuildOptions))
      throw new TypeError("Expected BuildOptions");
    const result = await asyncCall(() =>
      this.#handle.build(JSON.stringify(options)),
    );
    return makeOutput(
      artifactFromNative(result.artifact, process.cwd()),
      JSON.parse(result.snapshot),
    );
  }
  async compare(options: CompareOptions): Promise<ComparisonReport> {
    if (!(options instanceof CompareOptions))
      throw new TypeError("Expected CompareOptions");
    return new ComparisonReport(
      JSON.parse(
        await asyncCall(() => this.#handle.compare(JSON.stringify(options))),
      ),
    );
  }
}
