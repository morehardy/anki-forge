# Python diagnostics

`Project.add_note/add_notetype` validate in Rust and raise `ProjectAddError`
synchronously, without partially changing the project. Media registration raises
`MediaError`; template import raises `TemplateBundleError`; Deck additions raise
`DeckError`. These are `ValidationError` subclasses with stable core `code` and
`message`. Project add errors include a full `diagnostic`, `path` and optional
UTF-8 byte `span`. Bundle errors retain the file `path` and `byte_offset`.

`validate()` returns a `ValidationReport` without building an APKG or re-reading
media. Normalization, media verification and writing can still fail afterward.

`build()` / `write_apkg()` return valid core failure reports. Call
`report.ensure_success()` to raise `BuildError` (a `DiagnosticsError` subclass),
which retains the complete `report`, `diagnostics`, `code` and `failure_cause`.
A late persistence failure can retain a recoverable Artifact. `diff_against_apkg`
uses the same report-first convention with `ProjectDiffReport` and
`ProjectDiffError`. Raw reports preserve metrics, policy, versions, extensions
and Python integer precision; JSON deserialization never owns a temporary file.

## Diagnostic paths

Paths such as `project.notes[3]` are addresses in the core authoring snapshot,
not public Python attribute expressions. Indexes are zero-based. Stock type paths
can refer to declarations introduced by Rust. Use stable IDs for long-lived
traceability and inspect both `path` and `span` for template errors.

## Media and comparison

File registration reads and fingerprints the source immediately. Later deletion
or modification fails during build with core media diagnostics. Handle all error
diagnostics rather than only required-field codes.

`compare_to` computes package differences and risk. `fail_on` applies the selected
risk threshold and can also be used with lockfile-only evidence. The core decides
baseline validity and publication; Python does not require an APKG baseline for
every risk threshold. `InspectLimits` bounds both candidate and baseline reading.
`inspect=False` suppresses the summary, not the final package checks.

## Update-safe builds

```python
from anki_forge import BuildOptions, Note, Project

project = Project("Japanese Core", stable_id="jp-core")
project.add_note(Note.basic("食べる", "to eat", stable_id="jp:taberu"))
project.build(BuildOptions(output="jp-core.apkg")
    .first_update_safe_build("anki-forge.lock.json")).ensure_success()
```

Strict builds require the stable project identity and sufficient baseline
identity/revision evidence. `report_only` / `report-only` downgrades update-safety
failures to warnings; `disabled` disables identity preservation checks. Package
comparison and its inspection limits still operate when requested. Keep original
APKGs to recover missing revision evidence from legacy 0.1 lockfiles.

Import failures are separate from domain errors: reinstall a compatible wheel
for `BINDING.EXTENSION_UNAVAILABLE` or mixed files for `BINDING.VERSION_MISMATCH`.
`versions()` shows the three version axes. Busy/forked/retired native objects
raise RuntimeError with a `BINDING.*` code, rather than a fabricated core report.
