# Phase 3 Notes Data Carrier Probe

Record one row per Anki build tested.

Required fields:

- Date
- Platform
- Anki version
- anki-forge commit
- Input APKG path and SHA-256
- Imported note count
- Exported APKG path and SHA-256
- Whether `notes.data.anki_forge_identity` survived import/export
- Observed fallback path if metadata did not survive

Probe steps:

1. Build a one-note APKG with explicit `stable_id`.
2. Inspect the APKG SQLite `notes.data` and confirm `anki_forge_identity` exists.
3. Import into Anki.
4. Export the deck back to APKG.
5. Inspect exported APKG SQLite `notes.data`.
6. Mark the carrier as preserved only when the JSON object and `stable_id` survived.
