# Python 0.2 capability and verification index

Scope: the Rust Supported Consumer Interface, plus the explicitly named SDK
extensions in [the accepted plan](../../docs/plans/2026-09-21-python-api-parity-implementation-plan.md).
This is an implementation/verification index, not a package publication notice.

| Gap | Public behavior / implementation | Evidence |
| --- | --- | --- |
| PY-01 | Project.validate, no build or media reread | test_native_authoring |
| PY-02 | Structured atomic add errors | test_native_authoring, test_product_validation_parity |
| PY-03 | Core bundle loader, source byte offsets, atomic assets | test_native_bundle; independent Rust `bundle` producer |
| PY-04 | Field fallback/type recipe/note override/explicit stable ID | test_native_parity; independent `identity` producer; actual 0.1 fixtures |
| PY-05 | Exact template/CSS whitespace and Unicode | test_product_e2e, test_native_bundle; Rust producer |
| PY-06 | Registration-time fingerprints verified again at build | test_native_project, test_native_media, test_native_deck |
| PY-07 | Project empty/64 KiB/dedup semantics | test_native_media |
| PY-08 | Cross-project references resolve destination filename | test_native_parity `media`; checks real media hashes |
| PY-09 | BuildOptions, InspectLimits, paths, advanced media policy | test_native_build_options; every budget independently exercised |
| PY-10 | Owned Artifact, clone, close, persistence, late failures | test_native_artifacts; fork ownership in test_native_concurrency |
| PY-11 | bytes and bounded completed-file copying | test_native_output; independent Rust `large` producer |
| PY-12 | Independent diff with no publication/lock advance | test_native_diff; complete Rust diff report comparison |
| PY-13 | Complete lossless reports, unknown fields and large integers | test_report, test_native_diff, test_native_artifacts |
| PY-14 | Risk thresholds delegated to core, including lock-only inputs | test_product_e2e, test_native_build_options |
| PY-15 | Add snapshots, immutable settings, detached observations | test_native_authoring, test_product_validation_parity; MIGRATION.md |
| PY-16 | Core default keys and stock aliases | test_native_authoring/parity/migration |
| PY-17 | Actual Rust Deck, inferred IO, identity override, Project conversion | test_native_deck; independent `deck` and `deck_project` producers |
| PY-18 | optional declaration/readback, required conflict | test_native_authoring; no extra card-generation promise |
| PY-19 | Aliases, staging overlap, permission failure and atomic publication | test_native_build_options/artifacts, test_product_e2e |
| PY-20 | Native wheel, source build, typing, versions, concurrency, docs | test_native_distribution/concurrency, check_native_wheel/sdist, CI |

## Independent comparison boundary

`native/examples/python_parity.rs` constructs Rust Projects/Decks directly; it
never consumes Python-generated ProductDocument or adapter input JSON. Both
producers use the same complete APKG inspector and identity reader. Comparisons
include fields/templates/browser templates, model/config IDs, GUIDs, provenance,
revision evidence, cards, media hashes and missing/degraded observation status.
Only the observer's verified input filename is normalized. Complete diff-report
comparison normalizes only wall-clock duration.

Scenarios: Basic, custom multi-template, type/note/explicit identity precedence,
stock/custom Cloze, Project IO, Deck inferred IO, Deck conversion plus custom
notes, cross-project media, large file-object output, template bundle assets,
field reorder, derived identity-field rename and explicit-ID field rename.

`tests/fixtures/python01` was produced by actual Python 0.1 at `51a44ad`.
Migration checks unchanged/answer/tag/repeated/reverted updates and a lockfile
without revision evidence. Two serialized migration differences are explicit:
GUID source becomes previous_apkg and inferred template generation requirements
are now stored by the core Project. Other observed evidence remains compared.

## Existing core limits and compatibility changes

- Grouped `hide_one_guess_one` IO currently fails with
  `PRODUCT.CLOZE_MARKER_MALFORMED` in both Product and Deck paths.
- Custom derived identity includes selected **display names**. Reordering fields
  preserves GUIDs; renaming identity fields can change them. Explicit stable IDs
  remain stable. All three cases match independent Rust products.
- Python 0.2 captures additions and registers media eagerly. Project settings are
  read-only. Core default keys differ from some Python 0.1 punctuation rules.
- RuntimeOverride, ProductDocument serialization, CLI subprocess controls,
  manually constructed sequence MediaRefs and the old MediaItem serializer view
  remain 0.1-specific. Public 0.2 has one native implementation; the dev-only raw
  CLI wrapper remains separate and is not shipped.

Old JSON-serializer tests were retired with that seam; their supported authoring,
content, templates, identity and media behavior is covered by actual APKG/parity
and migration tests above. Report-shape/forward-compatibility tests were retained
in test_report. Core update/path tests now perform real operations instead of
mocking subprocess argument lists. Legacy raw/structured CLI tests remain.

## Verification commands and platform scope

```sh
cargo build -p anki_forge_python_native --example python_parity --locked
python -m pytest bindings/python/tests -q
python -m mypy --config-file bindings/python/pyproject.toml bindings/python/src/anki_forge
cargo clippy -p anki_forge_python_native --all-targets --locked -- -D warnings
python bindings/python/scripts/check_native_wheel.py <wheel> --observer <python_parity-binary>
python bindings/python/scripts/check_native_sdist.py <sdist>
```

The wheel checker installs outside the checkout, removes Cargo/CLI/PYTHONPATH
from runtime discovery, runs the complete example and positive/negative mypy
consumers, then optionally the public suite with independent Rust observations.
The CI workflow builds four real platform wheels and installs each on ordinary
CPython 3.11 and 3.12. Linux uses manylinux2014; Apple Silicon uses macOS 11+.
The actual wheel filename/metadata records the Intel macOS deployment target.
Windows lacks fork/POSIX permission-bit tests; those skips are named explicitly.

T2 four-platform trial passed at `8a44a94` in
[run 35687448346](https://github.com/morehardy/anki-forge/actions/runs/35687448346).
**That run is not final API evidence.** Final candidate package, review and CI
results are recorded in the [implementation log](../../docs/plans/2026-09-22-python-api-parity-progress.md).
No support claim is made for free-threaded interpreters, subinterpreters,
Python 3.13/3.14, cancellation or multi-file transactional rollback.
