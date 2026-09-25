/** JSON data mirrors Rust snapshots; saving a snapshot never owns an artifact. */
export type PathSnapshot =
  | string
  | { readonly encoding: "unix_bytes"; readonly bytes: readonly number[] }
  | { readonly encoding: "windows_wide"; readonly units: readonly number[] };
export type RiskLevel = "info" | "low" | "medium" | "high" | "critical";
export type RiskCode =
  | "RISK.NOTE_ADDED"
  | "RISK.NOTE_CHANGED"
  | "RISK.NOTE_REMOVED"
  | "RISK.CARD_ADDED"
  | "RISK.CARD_REMOVED"
  | "RISK.MODEL_ADDED"
  | "RISK.MODEL_CHANGED"
  | "RISK.SORT_FIELD_CHANGED"
  | "RISK.MODEL_REMOVED"
  | "RISK.FIELD_ADDED"
  | "RISK.FIELD_REMOVED"
  | "RISK.TEMPLATE_ADDED"
  | "RISK.TEMPLATE_REMOVED"
  | "RISK.MASK_ADDED"
  | "RISK.MASK_REMOVED"
  | "RISK.MEDIA_ADDED"
  | "RISK.MEDIA_CHANGED"
  | "RISK.MEDIA_REMOVED";
export interface Diagnostic {
  readonly severity: string;
  readonly code: string;
  readonly message: string;
  readonly [key: string]: unknown;
}
export interface BuildCounts {
  readonly notes: number;
  readonly cards: number;
  readonly media: number;
}
export interface ComparisonEvidence {
  readonly selector: string;
  readonly before: unknown;
  readonly after: unknown;
}
export interface RiskFinding {
  readonly code: RiskCode;
  readonly level: RiskLevel;
  readonly message: string;
  readonly evidence: readonly ComparisonEvidence[];
}
export interface PolicySnapshot {
  readonly allows_publication: boolean;
  readonly threshold: RiskLevel;
  readonly allowed_codes: readonly RiskCode[];
  readonly unmatched_allowances: readonly RiskCode[];
  readonly blocking_findings: readonly RiskFinding[];
}
export interface ComparisonSnapshot {
  readonly schema_version: string;
  readonly findings: readonly RiskFinding[];
  readonly highest_risk: RiskLevel | null;
  readonly policy: PolicySnapshot;
  readonly diagnostics: readonly Diagnostic[];
  readonly baseline_counts: BuildCounts;
  readonly candidate_counts: BuildCounts;
}
export interface ReportSnapshot {
  readonly schema_version: string;
  readonly counts: BuildCounts;
  readonly baseline_counts: BuildCounts | null;
  readonly diagnostics: readonly Diagnostic[];
  readonly duration_ms: number;
  readonly comparison: ComparisonSnapshot | null;
}
export interface PublicationSnapshot {
  readonly path: PathSnapshot;
  readonly stage: "not_published" | "published";
  readonly temporary: boolean;
  readonly durability: "confirmed" | "unconfirmed";
}
export type BuildResultSnapshot =
  | {
      readonly status: "success";
      readonly artifact: PathSnapshot;
      readonly temporary: boolean;
    }
  | {
      readonly status: "failure";
      readonly kind: string;
      readonly code: string;
      readonly message: string;
      readonly causes: readonly string[];
      readonly publications: readonly PublicationSnapshot[];
    };
export interface BuildSnapshot {
  readonly schema_version: string;
  readonly tool_version: string;
  readonly result: BuildResultSnapshot;
  readonly report: ReportSnapshot;
}
