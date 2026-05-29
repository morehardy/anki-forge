# Diagnostics

`Project.write_apkg()` returns a `BuildReport`. Call `report.ensure_success()` when you want invalid, blocked, error, missing-artifact, or error-diagnostic reports to raise `DiagnosticsError`.

## Source Paths

`source_path` values such as `project.notes[3]` are diagnostic addresses, not public Python object access. Note indexes are zero-based: `project.notes[3]` means the fourth note in the exact serialization that produced the report. Index paths may decay after project mutation or note reordering, so use explicit note `stable_id` values for long-lived traceability.

`project.note_types["basic"]` and `project.note_types["cloze"]` may refer to Python-generated stock declarations, even though users never passed those note types to `Project.add_notetype(note_type)`.

## Required Fields And Media

Python validates field keys and identity availability, but Rust owns required-field completeness and media diagnostics. A required media field with a valid media id can fail as a media-source diagnostic if its file source is missing or unreadable; do not filter only for required-field codes when deciding whether a media-heavy build is safe.

## Comparison

`fail_on=None` disables the risk threshold. It does not ignore missing or unreadable `compare_to` baselines; those still produce invalid reports with comparison diagnostics. `compare_to` without `fail_on` still computes diff and risk when the baseline is readable.
