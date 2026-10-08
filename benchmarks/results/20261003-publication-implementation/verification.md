# Functional verification and review

The public interfaces and real APKG contents are the principal test seams. Existing
build tests now traverse the same preparation/publication core as the explicit
owner API. Tests use the existing outer failure seams when public inputs cannot
reliably reproduce a durability or worker-termination failure.

| Spec gate | Evidence |
|---|---|
| F1 candidate/report equivalence | Rust `prepared_publication_tests`, consumer update identity/cards/revival suites; full comparison equality and raw-content verification in every core/SDK measurement batch |
| F2 policy and hard failures | Prepared blocked-owner test; existing update policy/evidence/inspection-limit consumers through the shared core; blocked workflow benchmark verifies full failure evidence and no public output |
| F3 ownership/lifecycle | Rust prepared and artifact lifecycle tests; Node single-use/close/concurrent calls; Python prepared owner and inherited-owner fork tests; benchmark temporary roots must be empty after child completion |
| F4 paths and publication facts | Prepared cwd/review-wait/original-file/hardlink/changed-symlink tests; existing native artifact failure-injection, non-Unicode and path-alias tests |
| F5 Node snapshots | Public build/compare/prepare test verifies contents after add/addAsset/defaultDeck, clone edits and rejected add; benchmark clone counts and real APKG verification |
| F6 Node task teardown | Shared task ownership cleanup; existing artifact lifecycle; expanded public teardown test terminates build, compare and prepare workers three times each after invocation on 1k-note projects |
| F7 staging paths | Native manifest obstruction regression, writer conformance/canonical tests and full-staging controls; controls assert real manifest contents/references and retain matching SHA-1 fingerprints |
| F8 byte validation/sharing | Public cache-hit test with unavailable temporary directory, strict budgets/MIME failure checks, independent names, 2/8 MiB repeated and unique import controls |
| F9 races/cleanup/fork | Public concurrent 2/8 MiB ownership tests, existing weak-cache/last-owner/internal fork-lock tests, Python inherited-media/prepared-owner tests, concurrent-miss measurement controls |
| F10 consumer distribution | Packaged Rust consumer, installed Node ESM/CJS/TypeScript/README consumers, Python native wheel and positive/negative typing; stale native protocol rejection |
| F11 Anki | 28-scenario roundtrip oracle plus selected measurement exports imported by the real Anki oracle; raw package verification is separate from Anki import |

Final logs are in `raw-evidence.tar.gz`. The Rust workspace run records 708 passed,
0 failed and 23 ignored. Node records 29 passed and one platform skip; Python
records 67 passed and one skip. The later worker teardown extension passed its
own targeted run. Its first invocation omitted the workspace native binary path
and failed loading the module; that log is retained beside the successful explicit
native-path run. No production source changed after the full suites.

mypy, TypeScript, clippy, Rust formatting, contract governance, embedded bundle and
payload checks passed. The actual installed-package checks and native wheel
consumers passed. These are macOS arm64 results; other supported platforms still
require their CI completion and release gates.

## Standards

The parallel standards review found misplaced changelog entries and duplicate
Node error/result conversion; both were corrected. Benchmark review found missing
input identities, unchecked comparison reports, a missing binary output directory,
and incomplete collector-state validation. Shared inventory/collector validators,
full-report equality and binary-directory creation address these findings. All
saved valid collector attempts are also checked offline, including interruption,
reaping and leftover descendants. No unresolved code-standard finding remains.

## Spec

The parallel spec review found missing performance controls and mixed Node edit/
operation measurements. Standalone compare, full staging, concurrent misses, real
Node/Python P1 workflows, a Node operation-only matrix and the existing full
20-cell standard matrix were added. Their measurements and failure records remain
separate from the original batches. The staging control's in-process validation
cost is explicitly disclosed instead of presenting its RSS as writer-only memory.

Two acceptance requirements remain unresolved: repeated wide-field RSS increases
exceed the specification's regression gate, and the supported-platform CI matrix
has not run. P1–P4 timing targets passing does not close either requirement. The
report records the concrete RSS tradeoff for a spec decision. Cross-platform CI
is a completion requirement as well as a release prerequisite.

Review summary: Standards 0 unresolved; Spec 2 acceptance requirements, confirmed wide-field RSS regression and supported-platform CI.
