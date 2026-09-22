/** Immutable observations of Rust product values; these are not editable authoring IR. */
export interface FieldSnapshot {
  readonly key: string;
  readonly name: string;
  readonly identity: boolean;
  readonly sort: boolean;
  readonly required: boolean;
  readonly optional: boolean;
  readonly keyAutoDerived: boolean;
}
export interface IdentityRecipeSnapshot {
  readonly fieldKeys: readonly string[];
}
export type GenerationRuleSnapshot =
  | { readonly kind: 'anki_default' }
  | { readonly kind: 'all' | 'any'; readonly fields: readonly string[] }
  | { readonly kind: 'cloze'; readonly field: string };
export interface TemplateSnapshot {
  readonly key: string;
  readonly name: string;
  readonly front: string;
  readonly back: string;
  readonly browserFront: string | null;
  readonly browserBack: string | null;
  readonly targetDeck: string | null;
  readonly generateWhen: GenerationRuleSnapshot;
}
export interface NoteTypeSnapshot {
  readonly id: string;
  readonly kind: 'normal' | 'cloze';
  readonly clozeField: string | null;
  readonly name: string | null;
  readonly fields: readonly FieldSnapshot[];
  readonly templates: readonly TemplateSnapshot[];
  readonly css: string | null;
  readonly identity: IdentityRecipeSnapshot | null;
}
export interface NoteSnapshot {
  readonly noteTypeId: string;
  readonly stableId: string | null;
  readonly deckName: string | null;
  readonly tags: readonly string[];
  readonly identity: IdentityRecipeSnapshot | null;
  readonly renderedFields: Readonly<Record<string, string>>;
}
export interface ResolvedDeckIdentitySnapshot {
  readonly stableId: string;
  readonly recipeId: string | null;
  readonly provenance:
    | 'explicit_stable_id'
    | 'inferred_from_note_fields'
    | 'inferred_from_notetype_fields'
    | 'inferred_from_stock_recipe';
  readonly canonicalPayload: string | null;
  readonly usedOverride: boolean;
}
interface DeckNoteSnapshotBase {
  readonly id: string;
  readonly stableId: string | null;
  readonly tags: readonly string[];
  readonly generated: boolean;
  readonly resolvedIdentity: ResolvedDeckIdentitySnapshot | null;
}
export type DeckNoteSnapshot = DeckNoteSnapshotBase &
  (
    | {
        readonly kind: 'basic';
        readonly front: string;
        readonly back: string;
        readonly identityOverride: {
          readonly fields: readonly ('front' | 'back')[];
          readonly reasonCode: string;
        } | null;
      }
    | { readonly kind: 'cloze'; readonly text: string; readonly extra: string }
    | {
        readonly kind: 'image_occlusion';
        readonly image: string;
        readonly mode: import('./types').IoMode;
        readonly rects: readonly Readonly<import('./types').Rect>[];
        readonly header: string;
        readonly backExtra: string;
        readonly comments: string;
      }
  );
export interface DeckSnapshot {
  readonly name: string;
  readonly stableId: string | null;
  readonly identityPolicy: { readonly basic: readonly ('front' | 'back')[] | null };
  readonly notes: readonly DeckNoteSnapshot[];
}
