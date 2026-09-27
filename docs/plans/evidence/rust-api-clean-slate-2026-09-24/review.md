# Final design review

Review baseline: `115ed2761dd6bad66fb6cf4fb72919a49a0313d4`.
Scope: tracked diff and all new implementation, consumer, contract and SDK files.
Two GPT-6 Astra agents independently reviewed standards and specification;
implementation fixes were followed by focused read-only re-review.

## Standards

- **P1: fork media ownership — resolved.** Python inherited Media/Content/Note/
  NoteType/Project handles cannot invoke or destroy parent-owned resources.
  Snapshot ownership and cache access are process-aware; a child never waits on
  an inherited cache lock. Fresh child imports own and clean their own snapshots.
  Five owner classes have real fork subprocess regressions with timeouts.
- **P2: bundle budget facts — resolved.** Python preserves typed source details
  for manifest/text/media budget failures, including limit and observed values.
  Genuine over-budget fixtures and adjusted-budget success are covered.

No further actionable standards findings were reported on re-review.

## Spec

- **P2: normalization I/O causes — resolved.** Real filesystem/read/write/sync
  errors flow through normalization into BuildError with Io classification,
  diagnostics and completed baseline observations. Tests downcast the original
  nested error rather than accepting a reconstructed message.
- **P2: Python bundle budget control — resolved.** Public MediaLimits reaches
  Rust from_bundle_with_limits and affects actual asset loading.
- **P2: non-finite IO coordinates — resolved.** Node/Python pass IEEE floats
  directly through their native bridges. NaN and infinities reach the same Rust
  builder validation and return NOTE.IO_RECT_INVALID at build completion.

No further actionable specification findings were reported on re-review.

This closes the two Standards and three Spec findings. It does not replace final
quality/distribution gates or claim execution of other platforms. The real Anki
oracle documents client import limitations separately; same-second Always skip
was source-checked but not naturally reproduced in the recorded run.

## Focused acceptance-harness follow-up

Concurrent verification reproduced a shared runnable-artifact race. Public
consumer binaries now use a process ID and per-process sequence. Documentation
main/observer binaries and media probes use process-specific names; all build and
launch paths were checked independently. Two concurrent public suites passed
21/21 each, two documentation runs passed 16 executions/22 APKG checks each, and
media lifecycle 4/4 plus the measurement launcher smoke passed. This follow-up
changes repository verification tools only, not production behavior.
