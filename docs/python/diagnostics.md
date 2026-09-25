# Python diagnostics and update policy

A successful build returns BuildOutput with a guaranteed artifact. A failed build raises BuildError with observations and publication facts. Diagnostics alone never determine success.

```python
import json
from anki_forge import Project, Note, BuildOptions, BuildError

project = Project('course').add('term', Note.basic('Question', 'Answer'))
try:
    output = project.build(BuildOptions.to('course.apkg'))
except BuildError as error:
    print(error.kind, error.code)
    print(json.dumps(error.snapshot(), ensure_ascii=False))
else:
    print(output.artifact.path)
    print(json.dumps(output.report.snapshot(), ensure_ascii=False))
```

Use machine `kind` and `code` for decisions, not message parsing. All public domain exceptions retain source-chain text in `causes` and chain their native exception. `details` carries operation-specific evidence: template source locations, media paths and budget measurements, incomplete comparison observations, or publication facts. An I/O source becomes a chained OSError where available. `source_details` preserves typed nested errors: bundle text-budget causes include `limit` and `observed`, while media causes include their `code`, `path` and `limit_exceeded`.

BuildReport only contains observations. Its snapshot has counts, diagnostics, elapsed time and any completed comparison. Output and BuildError snapshots contain the actual Success/Failure outcome. Failure snapshots distinguish a destination that was never published from one published before a durability failure. JSON paths do not own files or extend temporary artifact lifetimes.

## Compare before publication

```python
from anki_forge import CompareOptions, UpdatePolicy

comparison = project.compare(CompareOptions.against('course-v1.apkg'))
print(comparison.findings)
print(comparison.policy)
```

A completed comparison returns normally even if it discovers High risk. Check `comparison.allows_publication` to see the policy decision. An unreadable baseline, missing complete identity evidence, invalid candidate or exceeded budget raises CompareError; its `.report` contains observations completed before failure.

Default policy blocks High and Critical findings. After reviewing a specific risk category, acceptance is explicit:

```python
policy = UpdatePolicy().allow('RISK.NOTE_REMOVED')
comparison = project.compare(CompareOptions.against('course-v1.apkg').update_policy(policy))
output = project.build(BuildOptions.to('course-v2.apkg')
    .update_from('course-v1.apkg').update_policy(policy))
```

Acceptance covers every finding in the named category. The findings keep their original risk and evidence. Unmatched allowances produce warnings; warnings do not turn successful publication into failure. Unknown codes, missing evidence and other hard errors cannot be accepted. Do not automatically accept every category returned by a comparison.

`update_from` requires the previous original distribution package, including its complete embedded evidence. Anki re-exports do not preserve that evidence. Omitted notes/cards in an APKG do not automatically delete them from a learner's collection. Review structural changes against the intended Anki import settings.

## Budgets and artifact ownership

```python
from anki_forge import InspectLimits

limits = InspectLimits(max_archive_bytes=3 << 30)
options = BuildOptions.to('course-v2.apkg').update_from('course-v1.apkg').inspect_limits(limits)
```

Budgets apply separately to each inspected baseline and candidate. Media import has its own earlier MediaLimits budget. Budget failures preserve resource, limit and observed values under `details['limit_exceeded']`; increase the relevant limit deliberately and retry. For bundles, pass `NoteType.from_bundle(path, limits=MediaLimits(...))` to adjust each asset’s read budget; manifest and template text retain their separate fixed limits. Required inspections cannot be disabled.

Use `BuildOptions.temporary()` when no persistent path is wanted. Retain the output or an artifact handle while consuming the file; `artifact.persist_to(path)` makes a persistent copy. If persistence fails, PersistError details record the target's publication stage and the original handle remains usable.

Snapshot and error-detail paths are strings when valid Unicode. Unix byte names
that Python represents with `surrogateescape` appear in JSON as
`{"encoding": "unix_bytes", "bytes": [...]}`; restore them with
`Path(os.fsdecode(bytes(value["bytes"])))`. Windows names containing unpaired
UTF-16 surrogates use `{"encoding": "windows_wide", "units": [...]}`, with each
array entry holding one original 16-bit code unit. Encoded paths describe their
original platform. Runtime `artifact.path` remains a `pathlib.Path`, and
`BuildOptions.to`, `update_from`, and `CompareOptions.against` preserve native
path values through the binding.

A missing or mismatched native extension raises an import error identifying `BINDING.EXTENSION_UNAVAILABLE` or `BINDING.VERSION_MISMATCH`. Install a matching wheel, or rebuild a source checkout with maturin. Concurrent operations on one Project are rejected; use independent projects for concurrent work.
