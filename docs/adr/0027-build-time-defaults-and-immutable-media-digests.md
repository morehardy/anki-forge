# ADR 0027: Build-time defaults and immutable media digests

## Decision

The embedded contract bundle remains the source of truth. Cargo's build script
validates its manifest, asset inventory, archive integrity, bundle version and
typed writer policy/build context, then writes the runtime JSON into `OUT_DIR`.
The build script, bundle validation tests and installed-bundle reader share the
same validation code and configuration models. Changing the bundle or generator
sources reruns generation; malformed embedded resources fail compilation.

The generated data is included in the crate binary and cached on first use.
Consumers need no repository checkout, resource-generation command or external
tool. External manifests retain their existing runtime validation. The contract
bundle itself, public API and package formats do not change. Host dependencies
may increase a clean build's work; runtime builds avoid archive decoding and
manifest/schema/YAML initialization. Long-lived processes benefit only on first
initialization because subsequent calls already reuse cached defaults.

Media preparation may reuse the BLAKE3 digest retained by a private immutable
memory snapshot. The snapshot storage owner determines eligibility; callers
cannot assert that arbitrary bytes or files are trusted. Individual temporary
files and shared spill segments are rehashed on every preparation, including
under the shared 64 MiB budget. All sources still undergo reading, SHA-1, size
checks, MIME sampling, encoding and final package inspection.

## Validation

Generated defaults are compared with both the shared decoder and the full
path-based bundle loader. Malformed archives, manifests, asset inventories and
default values exercise the same code used by the build script. Package checks
must build and run the crate outside the checkout.

Public consumer regressions corrupt an owned file and an owned spill segment,
then require `MEDIA.SOURCE_CHANGED`, no publication, and complete temporary
cleanup. Existing memory, source-deletion, fork, failure, budget and parallel
ownership tests remain applicable. Performance comparisons use untouched and
changed release binaries with identical inputs and independent RSS samples.
