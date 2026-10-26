# Historical performance snapshot

This unmodified CSV is the only experiment input required to build the website
and README charts. It was copied from the public repository history, not from a
new benchmark run.

- Repository: https://github.com/morehardy/anki-forge
- Evidence commit: `bef4aeb653fc875f216614e73d8a617be039b9cf`
- Measurement commit: `11991964b06896b2e07ba09cbb3e6465f7585099`
- Original path: `benchmarks/results/20261008-latest-commit-genanki/comparison.csv`
- Measured: 2026-10-08
- SHA-256: `252eae6c6f7f947b84b7430b62a6401d8af93840595b56abb0e79b7697668999`

[Original CSV](https://github.com/morehardy/anki-forge/blob/bef4aeb653fc875f216614e73d8a617be039b9cf/benchmarks/results/20261008-latest-commit-genanki/comparison.csv)
· [Report and limitations](https://github.com/morehardy/anki-forge/blob/bef4aeb653fc875f216614e73d8a617be039b9cf/benchmarks/results/20261008-latest-commit-genanki/report.md)

The selection in `docs/benchmark-presentation.json` separates the local `dataFile`
from the historical `dataset`. Keep both evidence commits and the checksum when
moving the local file. Website loading and README generation verify the checksum.
The five displayed 1,000-note rows measure the Rust Project API against genanki
0.13.1 on an M1 Pro; they do not measure the Python or Node hosts or today's
registry packages. No new experiment or general cross-language speed claim is implied.

The former automatic benchmark smoke workflow is now manual reproduction against
this immutable evidence commit. It does not run missing tools in this checkout.
The pre-existing benchmark deletions are deliberately outside the GEO commits.
