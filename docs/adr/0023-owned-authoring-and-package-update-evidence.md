# ADR 0023: Owned Authoring and Complete Package Update Evidence

Decision date: 2026-09-24. Specification: [clean-slate design](../plans/2026-09-23-rust-api-clean-slate-design.md).

One `Project` collects immutable owned models, notes, and media. Typed content
retains assets until rendering; file imports take snapshots before returning.
Field/template/mask keys and explicit namespace/note keys are independent of
human-readable names. Template bundles use v2 and compile stable field keys to
Anki display names only at output.

Every APKG carries verified complete identity evidence. Updating requires the
original distributed package, preserves historical assignments and advances
changed revisions. Separate comparison returns full evidence even when policy
blocks publication. `BuildOutput` owns a successful artifact; observation-only
reports and JSON snapshots cannot keep temporary files alive. Typed errors retain
source chains and truthful publication/durability facts.

Native filesystem paths in snapshots preserve their exact bytes/code units:
Unicode paths are strings; other Unix/Windows paths use tagged numeric arrays.
The build-report schema covers both forms. Input schemas describe structural
constraints; loaders additionally enforce UTF-8 byte budgets, filesystem/media
validity and references. In particular, portable export names retain the 255-byte
runtime limit without restricting the accepted Unicode alphabet to fit a
character-count approximation.

This replaces the former public Deck facade, mutable registry, inferred identity
recipes, lockfile update protocol and SDK transport to ProductDocument. It
supersedes contrary interface/update decisions in ADRs 0003, 0004, 0014, 0015,
0017–0019, 0021 and 0022. Their implementation experiments remain historical
evidence; they do not define current consumer behavior.

Node and Python adapt the same Rust owned values and operation pipeline.
Published low-level conformance fixtures remain test inputs for private engines,
not an alternate native authoring model. Actual Anki import settings and local
revision times can prevent content updates; tested limitations are recorded in
[the real import oracle results](../../scripts/roundtrip_oracle/RESULTS.md).
