# README export comparison

[Full report](report.md) · [Machine-readable summary](summary.json) · [Predeclared plan](plan.json)

Three complete sessions at clean package revision `ab7d261`, with five Basic
workloads at 100/200/500/1,000 notes. All 2,520 exports passed artifact checks;
120 packages passed the pinned, locally patched Anki checker's import/content/render
checks. Every sample and session is retained, including the approximately tied
1,000-image case.

`round-1/` through `round-3/` contain original manifests, attempts, verified
samples, summaries and compressed artifact/Anki check records. `plan.json` and
`run_three.py` preserve the plan and commands saved before measurement.
`evidence-sha256.json` pins original evidence bytes. APKGs, media fixtures,
executables and build caches are excluded; recorded absolute paths describe
the measurement host.

`media_report.py` generates the aggregate tables and SVGs offline from this
evidence. See the [suite instructions](../../README.md) for regeneration and
reproduction. The report describes native Rust default exports on one host;
it does not measure the bindings or establish a cross-platform release guarantee.
