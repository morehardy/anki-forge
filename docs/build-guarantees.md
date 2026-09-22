# Build and output guarantees

Use this reference when retaining artifacts or publishing updates. For a first
export use the [installation guide](installation.md); for a release sequence use
[update and distribute a deck](updates.md).

## Temporary and persistent artifacts

A build without an explicit output or artifact directory returns a temporary
`ApkgArtifact`. Keep its report or an owning artifact handle alive while using
the path. Copying the path does not retain the file. Use `persist_to(path)`,
`write_apkg(path)` or an explicit output for a permanent package.

Explicit destinations are caller-owned and survive handle cleanup. Report JSON
is a snapshot and cannot own an artifact, so automatic `report_json` requires
an output or artifact directory. See the full
[artifact ownership contract](../anki_forge/README.md#artifact-ownership).

## Baselines and publication

When using `compare_to(previous_apkg)`, keep the baseline separate from the
output, `artifacts_dir/package.apkg`, report JSON, and any writable identity
lockfile. Builds reject existing same-file aliases (including relative paths,
symlinks, and hard links) with `PROJECT.PATH_COLLISION` before writing. This
includes the actual `staging/manifest.json` destination, whose links may point
outside the artifact directory. Baselines, outputs, retained packages, and
identity lockfiles (including read-only ones) must stay outside the writable
`staging/` tree and its media directory, including directory aliases. Staging
materialization must not overwrite these files before a risk rejection.

New destinations are rechecked after creation and before lockfile/report writes,
so filesystem-specific case folding cannot turn an APKG into JSON. A collision
detected after publication returns an error but keeps the valid published APKG.

The baseline is inspected once before building; GUID reconciliation and diff
use that same snapshot. The candidate APKG is compared and checked against
`fail_on(...)` before publishing the APKG or updating the identity lockfile.
On a blocked build, existing outputs and lockfiles remain unchanged, and the
report retains diff/risk evidence with `artifact: null`. A separate report JSON
can still be written; intermediate staging/media files may remain in an explicit
artifact directory. Successful publication uses atomic replacement per file,
not a transaction spanning the APKG, lockfile, and report.
Output-only builds copy directly from the private candidate to the requested
output; no extra package copy is made in the disposable artifact workspace.
Private candidates live inside the artifact workspace, so an explicit
`artifacts_dir(...)` also selects their filesystem; they are removed after the
build. Lockfiles use an exclusively reserved temporary file beside the target,
so temporary names cannot overwrite existing baselines or outputs.

## Inspection and failures

Inspection applies finite archive, entry-count, expansion and zstd-window budgets
to the candidate and baseline. Start with `InspectLimits::default()` and raise
only the relevant budget for a trusted larger deck. These limits are not a
process-wide memory or CPU sandbox.

Check structured errors and reports. A late persistence failure can retain a
valid published artifact in the error report. See
[errors and concurrency](../anki_forge/README.md#errors-and-concurrency) and
[troubleshooting](troubleshooting.md#updates-and-outputs).
