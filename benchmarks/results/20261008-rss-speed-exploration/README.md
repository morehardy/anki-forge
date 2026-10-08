# RSS / speed exploration, 2026-10-08

Read [the Chinese report](report.md) for decisions, independent confirmation,
uncertainty and validation scope. [All comparisons](tables.md) include first-pass
screening and confirmation; [contrasts.json](contrasts.json) retains marginal
component comparisons. The implementation remains experimental.

The evidence archive contains source snapshots, test/build logs, input JSON,
raw timing/RSS/verification/Anki records, plans and analysis scripts. Its manifest
lists every retained file and the deliberate omissions. Generated APKGs were
validated, hashed and deleted; media/exporter/tool bytes are omitted from the
archive. They are not freshly validated by replay.

To verify archive hashes, source inventories, recomputed statistics and recorded
outcomes, run from this directory:

```sh
python3 replay.py
```

This uses Python's standard library and a temporary directory. It does not run
exporters or Anki. [replay-result.json](replay-result.json) records the completed
reconciliation. [SHA256SUMS](SHA256SUMS) covers the top-level deliverables.

Full inputs, executable artifacts and the working experiment remain in
`benchmarks/.work/rss-speed-20261008/`. The archived `scripts/setup.py`,
`scripts/patch_media.py`, `scripts/patch_text.py`, `scripts/build.py`,
`scripts/measure.py` and `scripts/confirm.py` document experiment construction and
execution; they assume the original repository/workspace layout and local build
and benchmark dependencies. Portable performance reproduction requires restoring
those dependencies and the omitted fixture bytes; archive replay alone is not a
fresh performance measurement.
