# Native public-API oracle results — 2026-09-24

Command: `bash scripts/run_roundtrip_oracle.sh`. Local upstream Anki was compiled and used directly. The producer used `ankiforge` with default features disabled and no `internal-tools` access. Raw evidence is archived as [anki-import-oracle.json](../../docs/plans/evidence/rust-api-clean-slate-2026-09-24/anki-import-oracle.json). Reproduction writes `target/native-roundtrip-oracle/report.json` and the original APKGs beside it.

The executed matrix contains 7 version chains × 4 import settings = 28 scenarios and 96 actual APKG imports. The four settings are model merge disabled/enabled crossed with IfNewer/Always for note and model updates. Existing notes retained their actual Anki note IDs and GUIDs; existing cards retained their actual IDs, ordinals and review scheduling (23-day interval, 17 repetitions, 3 lapses, ease factor 2710). No note/model timestamp was edited by the producer or importer harness, and no sleeps were used to alter update ordering.

| Chain | Stages per setting | Observed behavior |
| --- | --- | --- |
| Basic content / project display title | 3 | All content updates applied under all four settings |
| Custom model/field/template rename and key declaration reorder | 3 | All content/name bindings correct; existing identity and scheduling retained |
| Field/template addition, removal, restoration | 4 | Without merge, addition/restoration logged conflicting notes and retained old body; model count grew from 7 to 8. With merge + IfNewer, schema timestamps prevented later body updates. With merge + Always, all four bodies applied in this run; model count remained 7 |
| Sort-field change, then another body edit | 3 | IfNewer retained the old body after the schema timestamp changed, including the subsequent older distribution; observed with either merge setting. Always applied all bodies in this run |
| Cloze equal-count replacement then restoration | 3 | All bodies applied; existing scheduled cards remained and restored ordinal reused its original card |
| IO HideAllGuessOne reorder/move/add/remove/restore | 4 | All bodies and independently expected normalized mask encodings matched; actual card counts 2 → 2 → 3 → 3 |
| IO HideOneGuessOne same chain | 4 | Same identity/card preservation; exact mode-specific encoding matched |

85 of 96 imported stages matched all active candidate field values. The remaining 11 are explicitly recorded limitations: 4 schema conflicts without merge, 3 schema/sort-field timestamp advances blocking the body update, and 4 later updates blocked by the already advanced target timestamp. These are not reported as successful content replacement.

The same-second Always skip is supported by the checked upstream `should_update` implementation and is handled by the oracle's exact equality condition, but **this run did not naturally encounter an equal-second Always failure**. It is not claimed as an executed behavioral case. The oracle does not manipulate timestamps to manufacture or conceal it.

This is a local source-based Anki import experiment. It does not replace the release platform matrix, distribution gates, or interactive image-occlusion rendering in every client.
