# Node SDK verification coverage

This is an implementation and evidence index, not a release certification.
The independent producer uses Rust product constructors in
`native/examples/sdk_parity.rs`. Node constructs matching inputs separately in
`test/parity.test.mjs`. Both APKGs are observed by Rust's existing inspector and
baseline identity reader. Tests compare complete observations, actual GUIDs,
model/config IDs, revision evidence and complete build JSON reports; only elapsed
time and the chosen test directory are normalized.

| Plan | Public behavior | Automated evidence |
| --- | --- | --- |
| C01 | Deck creation and lanes | Product Deck tests; independent `deck` scenario |
| C02 | Project stable ID and default deck | Independent `stock` scenario |
| C03 | Basic, Cloze, fields, tags, deck assignment | Independent `stock`, five `revision-*` scenarios |
| C04 | Custom Normal/Cloze | Independent `normal`, `custom-cloze` scenarios |
| C05 | Field keys, identity, sort and optional/required flags | Independent `normal`, `renamed`; product authoring tests |
| C06 | Template keys, browser, target deck and CSS | Independent `normal`, `renamed`, `reordered` |
| C07 | Default, all, any and Cloze generation | Independent `stock`, `normal`, `custom-cloze` |
| C08 | Explicit/inferred identities and Deck overrides | Product Deck/identity tests; independent full identity-index comparison |
| C09 | Text, HTML, image and sound content | Independent `stock`, `normal`, `media` |
| C10 | Files, bytes, buffers, duplicates and references | Product media/file/snapshot tests; independent `media` |
| C11 | Image occlusion | Independent `io`; product Project/Deck bounds tests. **Hide-one core error remains open.** |
| C12 | Template directory and assets | Product atomic-import test; independent `bundle` |
| C13 | Synchronous errors and atomic additions | Product addition, Deck, type and bundle tests; existing Rust addition regressions |
| C14 | Independent project/template validation | Product validation, no-publication and byte-offset tests |
| C15 | APKG file, Buffer and Writable | Independent inspected APKGs; product bytes/backpressure/stream-error tests |
| C16 | Baselines, locks, risk and safety modes | Five independent revision scenarios; legacy lock migration, risk-blocking and alias tests |
| C17 | Side-effect-free comparison | Product comparison and unchanged lockfile assertions |
| C18 | Options, reports, errors and numeric boundaries | Full independent report comparison; all 11 limit failures; MIME policies; corrupt baseline partial report; 64-bit model-ID/native number tests |

`npm test` discovers every `test/*.test.mjs` except the explicitly separate
parity suite. The runner validates references in
[`test/capability-matrix.json`](test/capability-matrix.json), which maps C01–C18
and N01–N08 to named tests. `npm run test:parity` builds the independent
Rust helper and runs parity/update regressions. `cargo test -p
anki_forge_node_native --test json_numbers --test inspect_limits --locked` checks
signed/unsigned report numbers and exact budget inputs at the JSON boundary.

`npm run test:installed` installs real main and host-platform tarballs from a
temporary registry. It verifies npm ci with a fresh cache, automatic optional
dependency selection, ESM/CJS class identity, positive/negative TypeScript cases,
all complete README JavaScript snippets, Unicode/space paths, read-only package
directories (Unix), and missing/mismatched native runtimes. `-- --all` requires
all four real platform binaries and serves all four tarballs. The CI consumer
jobs use this mode after collecting the four native-build artifacts.

`npm run prepare:desktop` retains 16 SDK-generated scenario packages, their
SHA-256 hashes, Rust/Node observations and reports, and a pending Desktop
checklist under `artifacts/desktop`. It does not launch Anki or claim that GUI
rendering, review history or upgrade imports were manually verified.

Still required before the complete release: resolve and validate the core
hide-one image-occlusion behavior; run the actual four-platform CI matrix and
minimum-system/dynamic-library checks; finish Desktop import/rendering checks;
verify npm package ownership and publishing credentials; perform prerelease,
partial-publication recovery and public-registry installation checks. Windows
symlink/ACL behavior needs its own verification; Unix mode-bit tests do not prove
Windows permissions. The capability table links current evidence and does not
claim every edge case in the implementation plan has been exhausted.

## Node parity additions

| Plan | Public behavior | Evidence |
| --- | --- | --- |
| N01 / C15 / C18 | Temporary/output/artifacts-only modes, native artifact ownership, late errors, aliases, GC and Worker teardown | artifact.test.mjs |
| N02 | Independent editable Deck import, mixed HTML/Basic/Cloze/IO, inferred IDs and media fingerprints | conversion.test.mjs; independent deck-import scenario |
| N03 / C15 | Bounded chunks, callbacks/drain, failures, listener and tempfile cleanup | stream.test.mjs; resource probes |
| N04 | Frozen canonical field/template/identity/Note/Deck snapshots | snapshot.test.mjs |
| N05 | Media lookup without reading or registering a file; Busy lookup | conversion.test.mjs; deck-mixed/deck-import scenarios |
| N06 | Independently editable copies, concurrent builds, shared large-buffer lifetime | clone.test.mjs |
| N07 / C18 | 11 exact u64 budgets for build/diff; unsafe values rejected; limits enforced | limits.test.mjs; native inspect_limits test |
| N08 | Direct Rust Deck builds, validation warning mapping, repeatable state | parity.test.mjs; direct deck/deck-mixed producers; native allocation probe |
| S01 | Source-visible lower() remains outside product compatibility | ADR 0012; Node README |

The installed-consumer suite checks new class identity, artifact cleanup,
conversion/cloning, readonly declarations and exact budget types. Same-version
outdated native protocols are rejected. CI includes the exact minimum 22.13.0 in
addition to rolling 22/24/26; configuration does not prove a remote job passed.

[Implementation and test evidence](../../docs/plans/2026-09-22-node-api-parity-progress.md)
and [resource measurements](../../docs/plans/2026-09-22-node-api-parity-resources.md)
record what was actually verified. Fault injection for native panic/failed states
and real Windows alias/ACL behavior are not claimed by the macOS result.
