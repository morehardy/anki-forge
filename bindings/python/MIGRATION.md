# Clean-slate Python API

No legacy compatibility layer is retained. Replace the old Deck/registry/runtime APIs with the owned public model:

| Previous workflow | Current workflow |
| --- | --- |
| Deck or Project with optional stable ID | `Project(namespace, name=..., default_deck=...)` |
| `add_note(note)` and content-derived identity | `add(stable_key, note)` |
| Mutable NoteType and add_notetype | `NoteType.builder(key)...build()`, then `model.note()` |
| String model references | Notes own their validated model |
| Project media registry | Independent `Media.file` / `Media.bytes` snapshots |
| Inline rendered media and field-specific setters | `Content.sequence`, `media.image()`, `media.sound()`, `Note.field` |
| Implicit HTML | Strings are Text; use `Content.html` |
| Masks without stable identity | `Mask.rect(key, x, y, width, height)` |
| Optional artifact plus ensure_success | `BuildOutput` with guaranteed artifact; failures raise structured errors |
| Report outcome guessed from diagnostics | Output/Error snapshots provide the actual result; Report only contains observations |
| Diff, lockfile and update-safety switches | `compare(CompareOptions.against(...))`, `BuildOptions.update_from(...)` |
| Runtime subprocess package | In-process PyO3 binding to the default Rust API |

Only complete original distribution APKGs carrying current identity evidence are update baselines. Legacy bundles and old APKG evidence are not recovered automatically. See README and executable examples for the new workflow.
