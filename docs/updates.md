# Update and distribute a deck

Correct a note from **hello** to **hello; hi** while keeping its stable ID
`es:hola`. The key ingredients are stable project/note identities and evidence
from the last distributed APKG or an identity lockfile.

## Run a two-version example

From the repository root:

```sh
cargo run --locked -p anki_forge --example docs_workflow -- target/docs-examples
```

The program writes `spanish-v1.apkg` and `spanish-v2.apkg` in `target/docs-examples`.
It checks that the second build has one note, one card, a comparison report and
one preserved note identity. These are package/build checks, not a review-history guarantee.

## Compare against the previous package

This excerpt from the [complete program](../anki_forge/examples/docs_workflow.rs)
constructs both source versions. Its `verify` helper checks inspected counts.

<!-- source: anki_forge/examples/docs_workflow.rs#updates -->
```rust
fn vocabulary(answer: &str) -> anyhow::Result<Project> {
    let mut project = Project::new("Spanish")
        .stable_id("docs-spanish-updates")
        .default_deck("Spanish");
    project.add_note(Note::basic("hola", answer).stable_id("es:hola"))?;
    Ok(project)
}

fn updates(output: &Path) -> anyhow::Result<()> {
    let previous = output.join("spanish-v1.apkg");
    let next = output.join("spanish-v2.apkg");
    vocabulary("hello")?
        .write_apkg(&previous)?
        .ensure_success()?;
    let report =
        vocabulary("hello; hi")?.build(BuildOptions::new().output(&next).compare_to(&previous))?;
    report.ensure_success()?;
    verify(&report, 1, 1, 0)?;
    ensure!(report.diff.is_some());
    let safety = report.update_safety.as_ref().expect("baseline evidence");
    ensure!(safety.notes_preserved == 1 && safety.notes_failed == 0);
    Ok(())
}
```
<!-- /source -->

Keep the project ID, note IDs and existing field/template keys stable. Give a new
logical note a new ID. Keep the previous package separate from the new output.
Changed content advances baseline revision evidence; unchanged content preserves it.

## Use an identity lockfile

A lockfile is another way to carry release evidence. The following function uses
`vocabulary(...)` from the preceding example:

<!-- source: anki_forge/examples/docs_workflow.rs#lockfile -->
```rust
fn lockfile_updates(output: &Path) -> anyhow::Result<()> {
    let lockfile = output.join("identity.lock.json");
    vocabulary("hello")?
        .build(
            BuildOptions::new()
                .output(output.join("locked-v1.apkg"))
                .first_update_safe_build(&lockfile),
        )?
        .ensure_success()?;
    let report = vocabulary("hello; hi")?.build(
        BuildOptions::new()
            .output(output.join("locked-v2.apkg"))
            .update_safe(&lockfile)
            .write_identity_lockfile(true),
    )?;
    report.ensure_success()?;
    verify(&report, 1, 1, 0)?;
    let safety = report.update_safety.as_ref().expect("lockfile evidence");
    ensure!(safety.notes_preserved == 1 && safety.notes_failed == 0);
    ensure!(lockfile.is_file());
    Ok(())
}
```
<!-- /source -->

`first_update_safe_build(...)` establishes the first lockfile. Later
`update_safe(...)` reads it. Reading does not advance it: request
`write_identity_lockfile(true)` when the candidate should write new evidence.
These strict builds need a stable project identity and sufficient baseline evidence.

The example writes candidate outputs to its own directory. In your release
process, keep the previous release's APKG and lockfile immutable, work with a copy,
and archive the accepted candidate evidence only after validation. A legacy lockfile
without revision evidence may require the previous APKG for migration.

## Review and distribute

1. Save the last distributed APKG and identity lockfile with their source revision.
2. Build to separate candidate paths and inspect the diagnostics, diff and update-safety report.
3. Test both first import and an update into an existing test collection in your target Anki version.
4. Check edited fields, card structure, media, local edits and the review state relevant to your users.
5. Distribute the accepted candidate and keep its APKG/lockfile as the next release's evidence.

Anki's import settings, local edits and client behavior still govern imported
updates. Stable IDs do not override them. A baseline-free `write_apkg()` is a
first-export path and does not guarantee later content updates.

For path collisions, temporary artifacts, blocked builds and atomic replacement,
see [build and output guarantees](build-guarantees.md). For language-specific
options, see the [Rust API](rust-api.md#build-options),
[Node API](node/api.md#build-compare-and-output), or
[Python diagnostics](python/diagnostics.md#update-safe-builds).
