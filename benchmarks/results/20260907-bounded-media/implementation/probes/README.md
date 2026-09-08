# Candidate source evidence

Each valid `build-*.json` records the exact crate/adapter source hashes and binary
hash. `source-content.tar.gz` contains each unique file once, named by SHA-256.
Reconstruct a variant by writing each referenced blob to its recorded relative
path; the private workspace root is `workspace.Cargo.toml`. Use the adapter's
locked Cargo manifest to rebuild. Patches compare each variant with the frozen
pre-change crate/adapter, including tests and unused source files where present.

`current` is the prior accepted production binary, pinned in each run plan.
`current-rebuilt` rebuilds its runtime source in the private path. `pool` and
`pool-root` test shared storage; `pool-borrowed` adds a rejected identity-borrowing
experiment. `accepted` is the real workspace build. The `*-memory` builds add
diagnostic allocation accounting only; their timings are not performance claims.

Two diagnostic builds with `invalid-copy-timestamps` in their names reused a stale
crate because copied source mtimes were older. They were detected by identical
binary hashes before any measurement, excluded, and rebuilt after forcing private
source mtimes fresh. All measured candidate logs show the intended crate compiled.
The original invalid build records remain in the archive.
