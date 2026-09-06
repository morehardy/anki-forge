export type Severity = 'info' | 'warning' | 'error';
export type BuildStatus = 'success' | 'blocked' | 'invalid' | 'error';
export type RiskLevel = 'info' | 'low' | 'medium' | 'high' | 'critical';
export type UpdateSafetyMode = 'strict' | 'report-only' | 'disabled';
export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

export interface Diagnostic {
  readonly code: string;
  readonly severity: Severity;
  readonly domain: string;
  readonly stage: string;
  readonly path: string | null;
  readonly span: { readonly byte_start: number; readonly byte_end: number } | null;
  readonly message: string;
  readonly suggested_fix: string | null;
}

export interface ProjectOptions {
  stableId?: string;
  defaultDeck?: string;
  baseDir?: string;
}

export interface NoteOptions {
  stableId?: string;
  deckName?: string;
  tags?: readonly string[];
  identity?: readonly string[];
}

export interface ClozeOptions extends NoteOptions {
  backExtra?: string;
}
export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}
export type IoMode = 'hide-all-guess-one' | 'hide-one-guess-one';
export interface ImageOcclusionOptions extends ClozeOptions {
  stableId: string;
  rects: readonly Rect[];
  mode?: IoMode;
  header?: string;
  comments?: string;
}

export interface BuildOptions {
  output: string;
  artifactsDir?: string;
  inspect?: boolean;
  compareTo?: string;
  failOn?: RiskLevel;
  reportJson?: string;
  identityLockfile?: string;
  writeIdentityLockfile?: boolean;
  updateSafety?: UpdateSafetyMode;
  selfContained?: boolean;
  inspectLimits?: InspectLimits;
  mediaMode?: 'path-backed' | 'self-contained';
  mediaStoreDir?: string;
  mediaPolicy?: MediaPolicy;
}

export interface InspectLimits {
  maxArchiveBytes?: number;
  maxEntries?: number;
  maxCentralDirectoryBytes?: number;
  maxZipEntryBytes?: number;
  maxZipTotalBytes?: number;
  maxMetaBytes?: number;
  maxMediaMapBytes?: number;
  maxCollectionBytes?: number;
  maxMediaBytes?: number;
  maxDecodedTotalBytes?: number;
  maxZstdWindowBytes?: number;
}
export interface MediaPolicy {
  unusedBinding?: 'ignore' | 'info' | 'warning' | 'error';
  unknownMime?: 'ignore' | 'info' | 'warning' | 'error';
  declaredMimeMismatch?: 'warning' | 'error';
}
export interface InspectSummary {
  readonly source_kind: string;
  readonly observation_status: string;
  readonly notes: number;
  readonly cards: number;
  readonly notetypes: number;
  readonly templates: number;
  readonly fields: number;
  readonly media: number;
}
export interface MediaSummary {
  readonly objects: number;
  readonly bindings: number;
  readonly references: number;
  readonly missing_references: number;
  readonly unsafe_references: number;
  readonly unused_bindings: number;
  readonly unique_bytes: number | string;
  readonly entries: readonly {
    readonly id: string;
    readonly filename: string;
    readonly source_mode: 'inline' | 'path_backed';
    readonly size_bytes: number | string;
  }[];
}
export interface BuildPolicyResult {
  readonly status: 'passed' | 'blocked' | 'not_evaluated';
  readonly threshold: RiskLevel | null;
  readonly highest_risk: RiskLevel | null;
  readonly blocking_findings: readonly string[];
}
export interface BaselineSourceSummary {
  readonly source_kind: string;
  readonly source_ref: string;
  readonly display_path: string | null;
  readonly status: string;
  readonly used_for_reconcile: boolean;
  readonly limitations: readonly string[];
  readonly diagnostic_codes: readonly string[];
}
export interface UpdateSafetySummary {
  readonly mode: string;
  readonly baseline_sources: readonly BaselineSourceSummary[];
  readonly notes_preserved: number;
  readonly notes_derived: number;
  readonly notes_failed: number;
  readonly baseline_conflicts: number;
  readonly blocking_diagnostics: readonly string[];
  readonly lockfile_written: boolean;
}
export interface EvidenceRef {
  readonly kind: 'diagnostic' | 'diff_change' | 'inspect_observation' | 'update_safety' | 'oracle';
  readonly ref_id: string;
}
export interface BuildDiffSummary {
  readonly artifact_diff: {
    readonly changes: readonly {
      readonly category: string;
      readonly domain: string;
      readonly severity: string;
      readonly selector: string;
      readonly message: string;
      readonly evidence_refs: readonly EvidenceRef[];
    }[];
    readonly limitations: readonly string[];
  } | null;
  readonly semantic_changes: readonly {
    readonly category:
      | 'notetype'
      | 'field'
      | 'template'
      | 'note_identity'
      | 'card_count'
      | 'media'
      | 'baseline';
    readonly selector: string;
    readonly change_kind: 'added' | 'removed' | 'modified' | 'reordered' | 'unavailable';
    readonly risk_codes: readonly string[];
    readonly message: string;
    readonly source: string | null;
  }[];
  readonly summary_counts: {
    readonly added: number;
    readonly removed: number;
    readonly modified: number;
    readonly reordered: number;
    readonly uncompared_domains: number;
  };
  readonly limitations: readonly string[];
}
export interface ImportRiskReport {
  readonly highest_level: RiskLevel | null;
  readonly limitations: readonly string[];
  readonly findings: readonly {
    readonly code: string;
    readonly level: RiskLevel;
    readonly category: string;
    readonly message: string;
    readonly source: string | null;
    readonly evidence_refs: readonly EvidenceRef[];
    readonly suggested_action: string | null;
  }[];
}
export interface CoreDiffReport {
  readonly status: BuildStatus;
  readonly comparison: CoreBuildReport['comparison'];
  readonly diagnostics: readonly Diagnostic[];
  readonly current_inspect: InspectSummary | null;
  readonly previous_inspect: InspectSummary | null;
  readonly update_safety: UpdateSafetySummary | null;
  readonly diff: BuildDiffSummary | null;
  readonly risk: ImportRiskReport | null;
  readonly metrics: { readonly duration_ms: number | string };
}

export interface BindingMetadata {
  readonly bindingVersion: string;
  readonly coreVersion: string;
  readonly contractVersion: string;
  readonly target: string;
  readonly nodeApiVersion: number;
}

export interface CoreBuildReport {
  readonly kind: 'anki-forge-build-report';
  readonly schema_version: string;
  readonly tool_version: string;
  readonly status: BuildStatus;
  readonly comparison: 'not_requested' | 'complete' | 'partial' | 'unavailable';
  readonly artifact: { readonly path: string } | null;
  readonly counts: { readonly notes: number; readonly cards: number; readonly media: number };
  readonly diagnostics: readonly Diagnostic[];
  readonly media: MediaSummary;
  readonly metrics: { readonly duration_ms: number | string };
  readonly policy: BuildPolicyResult;
  readonly inspect: InspectSummary | null;
  readonly previous_inspect: InspectSummary | null;
  readonly diff: BuildDiffSummary | null;
  readonly risk: ImportRiskReport | null;
  readonly update_safety: UpdateSafetySummary | null;
}
