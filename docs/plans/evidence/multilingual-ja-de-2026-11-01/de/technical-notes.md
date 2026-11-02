# German independent technical review — staging read

Reviewer: `/root/de_technical_review/staging-2026-11-01`.

Status: complete independent staging reading; **formal source approval is pending** the six-locale source freeze and candidate report. This reviewer did not author the German translations, modify product/shared/staging files, build a distribution, or create a commit.

## Material actually read

Read all 29 German document bodies and both complete German blog articles against the actual current English sources, including code blocks, conditions, limitations and frontmatter. Read all 210 German UI values, 11 framework values, 29 metadata records, navigation/group labels, language brief, complete coding-assistant prompt and all authored search terms/concepts. The article examples and common home/examples/blog-list consumers were checked against the actual Rust showcase and workflow source.

`technical-reading-records.json` contains 34 topic-specific semantic findings. `technical-input-hashes.json` records 139 exact input files and distinguishes complete reading, targeted SDK-contract reading and exact source-fence verification. Copies of those inputs are held under `target/remaining-locales/de-technical/read-snapshot/` for a real later delta comparison. Hash checks support the semantic reading; they are not its substitute.

## Source contract findings

- **Original APKG:** updates consistently require the previous original distribution with complete `ankiforge-identity-v1` evidence. Anki re-export is not a baseline. Namespace and note/model/field/template keys remain stable; omissions are not deletion synchronization. The copy does not promise overwriting newer local edits, restoring deleted history or preserving every learner state.
- **Policy and publication:** a completed comparison may return blocked findings. Default High/Critical blocking, whole-category allowances and unallowable hard errors agree with `UpdatePolicy` and the preparation pipeline. Prepared publication retains one inspected file, binds options/paths, consumes the owner on every attempt, excludes review waiting from duration and preserves late publication/durability facts.
- **Ownership:** typed content retains imported immutable media. Original input files may change after successful import. Models, media and temporary artifacts have distinct ownership; reports/JSON/path strings do not keep files alive. Node base-directory persistence, invocation snapshots and protocol 6; Python copies/context management, GIL release, busy/fork errors, protocol 1 and contract 2.1.0; Rust `Arc` artifact lifetime and call-time path resolution were checked in their actual implementations.
- **Budgets and schemas:** per-asset 256 MiB, 64 MiB shared snapshot memory, 1 MiB streaming threshold, per-preparation 16 MiB encoding pool and batch ordering/failure semantics retain their qualified scope. Inspection limits are independent per APKG; zero is a zero budget. Template-bundle v2 and tool-input v2 are not package-identity v1. The 255 UTF-8-byte export-name budget is additional to the schema character limit. Key validation rejects empty/surrounding whitespace without inventing a blanket ban on interior spaces.
- **Public packages:** basic authoring, batch media and prepared publication are public 0.3.1 APIs in the documented SDK contracts. Source checkout/Maturin/local facade/native packaging remain separate. Caller parsing of CSV/JSON is not an SDK converter; OCR, TTS and automatic diagram label detection are not claimed.
- **Shared copy:** count placeholders and plural forms preserve their variables. AI text requires the published API, exact install/run/verification, persistent output and original baseline. Historical benchmark copy remains a dated Rust Project API/genanki measurement and does not claim current Python/Node or import/review speed.

## Finding corrected before freeze

`DE-TECH-001` (minor inherited English ambiguity): the media page directly recommended `tools::load_project` next to the supported bundle loader without explaining its repository-only boundary. `anki_forge/src/lib.rs` gates this module behind `internal-tools` and explicitly excludes it from the supported consumer API. The parent added this boundary to the German staging paragraph. I re-read the complete paragraph and verified it against that source; the issue is resolved in the recorded input hash. No claim is made that unchanged older-language prose was rewritten.

I also read the corrected genanki sequencing sentence, explicit zero-budget wording and custom-key whitespace rule. They agree with the actual contracts. The troubleshooting persistence failure wording now points to saving/publishing, not RAM allocation.

## Mechanical corroboration and limits

Independently compared every fenced block in the 29 documents with its English counterpart. The Rust quickstart's MDX-injected program was compared directly with `target_api_basic.rs`; all `source:` fences were compared with the actual file or named source region. All checks passed. Catalog key/parameter coverage is 210 UI keys and 11 framework keys; metadata covers all 29 documents.

There are no unresolved German technical issues in these staging inputs. This is not a website source approval, package execution, browser, search-ranking or machine-fetch acceptance. The frozen six-locale candidates are not yet available. No withheld query file or `website/search-queries.json` was read. Vocabulary was reviewed from authored terms alone; later negative-query semantics must be reviewed against actual frozen observations.

For the eventual 204-entry report, preserve the existing five-locale independent review lineage and inspect every actual changed input. Bind only the German content read here plus any freshly reviewed deltas to the final source hashes. Prior real SDK execution evidence can be reused only with explicit unchanged-input and retained-output checks, never presented as a new execution.
