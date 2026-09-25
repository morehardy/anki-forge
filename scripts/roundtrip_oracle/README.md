# Real Anki import/update oracle

Run `bash scripts/run_roundtrip_oracle.sh [evidence-directory]`. This requires the repository's local upstream Anki source, protoc, and cached Rust dependencies. The default evidence directory is `target/native-roundtrip-oracle`; original APKGs, prepared input, observations and the oracle log are retained.

The producer (`anki_forge/examples/native_roundtrip_oracle_prepare.rs`) uses only the default Rust public API. It builds an original baseline and successive native updates with complete identity evidence. For this experiment, it deliberately permits High findings with an explicit policy, while recording default policy decisions and checking the expected risk codes. This is not tutorial publication policy.

The oracle calls upstream Anki's `Collection::import_apkg`, importing each scenario's complete chain into the same learner collection. Every scenario runs with `merge_notetypes` false and true, and with IfNewer and Always, for both note and note-type updates. Existing cards are placed into review with a 23-day interval and nonzero repetition/lapse/ease values through Anki's public scheduling/card APIs. Observations are read from the actual Anki collection and public CardsService.

Covered chains:

- Basic body changes and project display-title changes.
- Model/field/template display-name changes and field/template key declaration reordering.
- Field/template additions, removals and restoration.
- Sort-field changes followed by another body update.
- Cloze changes replacing one card ordinal with another at equal candidate card count, then restoration.
- Both IO modes: mask reordering and movement, equal-count addition/removal, then restoration; exact normalized mask HTML is compared against independently authored expected c1/c2/c3 associations.

Assertions compare actual Anki note IDs, GUIDs, card IDs, ordinals and scheduling across imports. They also check candidate content, field-name binding and presence of active card ordinals. Candidate package evidence is never used as a substitute for observing imported identity or scheduling. Omitted cards must remain in the learner collection; APKG import is not deletion synchronization.

No source or target note/model timestamp is edited to force an update, and the runner does not sleep to make timestamps pass. Structural updates can produce documented client constraints: without model merge, Anki can copy the incoming type and log a conflicting note; with model merge or a sort-field change, the schema operation can advance the learner note's timestamp before the body update decision. IfNewer compares timestamps strictly; Anki's Always mode also skips equal-second note timestamps. Such observations are recorded as explicit limitations, not counted as successful content replacement. Unexpected body mismatches or loss of existing identity/scheduling fail the oracle.

`report.json` records the source and observed note times, import settings, conflict counts, actual content, card scheduling, candidate findings and limitations for each stage. A `verified` status means the invariants and declared client-behavior constraints matched; it does not promise that every high-risk scenario updated its body under every import setting. Inspect `body_matches_candidate` and `limitations`.
