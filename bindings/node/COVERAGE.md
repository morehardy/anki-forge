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

`npm test` runs the product suite. `npm run test:parity` builds the independent
Rust helper and runs parity/update regressions. `cargo test -p
anki_forge_node_native --test json_numbers --locked` checks signed/unsigned
numeric boundaries before JSON reaches JavaScript.

`npm run test:installed` installs real main and host-platform tarballs from a
temporary registry. It verifies npm ci with a fresh cache, automatic optional
dependency selection, ESM/CJS class identity, positive/negative TypeScript cases,
all complete README JavaScript snippets, Unicode/space paths, read-only package
directories (Unix), and missing/mismatched native runtimes. `-- --all` requires
all four real platform binaries and serves all four tarballs. The CI consumer
jobs use this mode after collecting the four native-build artifacts.

`npm run prepare:desktop` retains 14 SDK-generated scenario packages, their
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
