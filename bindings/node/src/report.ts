import { deepFreeze } from "./internal/validation";
import type { ReportSnapshot, ComparisonSnapshot } from "./snapshots";

/** Observations shared by successful builds and structured operation errors. */
export class BuildReport {
  readonly #snapshot: ReportSnapshot;
  readonly #comparison: ComparisonReport | null;
  constructor(snapshot: ReportSnapshot) {
    this.#snapshot = deepFreeze(snapshot);
    this.#comparison =
      snapshot.comparison === null
        ? null
        : new ComparisonReport(snapshot.comparison);
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
  get comparison(): ComparisonReport | null {
    return this.#comparison;
  }
  snapshot(): ReportSnapshot {
    return this.#snapshot;
  }
}

/** Completed comparison, independent of whether publication is allowed. */
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
