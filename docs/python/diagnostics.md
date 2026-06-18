# Diagnostics

`Project.write_apkg()` returns a `BuildReport`. Call `report.ensure_success()` when you want invalid, blocked, error, missing-artifact, or error-diagnostic reports to raise `DiagnosticsError`.

## Source Paths

`source_path` values such as `project.notes[3]` are diagnostic addresses, not public Python object access. Note indexes are zero-based: `project.notes[3]` means the fourth note in the exact serialization that produced the report. Index paths may decay after project mutation or note reordering, so use explicit note `stable_id` values for long-lived traceability.

`project.note_types["basic"]` and `project.note_types["cloze"]` may refer to Python-generated stock declarations, even though users never passed those note types to `Project.add_notetype(note_type)`.

## Required Fields And Media

Python validates field keys and identity availability, but Rust owns required-field completeness and media diagnostics. A required media field with a valid media id can fail as a media-source diagnostic if its file source is missing or unreadable; do not filter only for required-field codes when deciding whether a media-heavy build is safe.

## Comparison

`fail_on=None` disables the risk threshold. It does not ignore missing or unreadable `compare_to` baselines; those still produce invalid reports with comparison diagnostics. `compare_to` without `fail_on` still computes diff and risk when the baseline is readable.

## Update-Safe Builds

`Project.write_apkg()` accepts `identity_lockfile`,
`write_identity_lockfile`, and `update_safety`.

```python
project = Project("Japanese Core", stable_id="jp-core")
project.add_note(Note.basic("食べる", "to eat", stable_id="jp:taberu"))
project.write_apkg(
    "dist/jp-core.apkg",
    identity_lockfile="anki-forge.lock.json",
    write_identity_lockfile=True,
    update_safety="strict",
).ensure_success()
```

Strict update-safe Python builds, default baseline-driven update-safe builds,
and any build that writes an identity lockfile require `Project.stable_id`.
`update_safety="disabled"` ignores baseline inputs; `update_safety="report_only"`
keeps update-safety diagnostics visible as warnings without blocking writer
execution.
