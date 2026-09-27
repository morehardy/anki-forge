# Update and distribute a publication

Keep the project namespace and note/model/field/template keys stable. Build each
new release from the previous **original distribution APKG**, which carries
complete versioned identity evidence. Save that package with its source revision.

## Run two versions and compare

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

This writes `spanish-v1.apkg` and `spanish-v2.apkg`, compares the changed answer,
and checks the updated package counts. The executable source is:

<!-- source: anki_forge/examples/docs_workflow.rs#updates -->
```rust
fn vocabulary(answer: &str) -> anyhow::Result<Project> {
    let mut project = Project::new("docs-spanish-updates")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", answer))?;
    Ok(project)
}

fn updates(output: &Path) -> anyhow::Result<()> {
    let previous = output.join("spanish-v1.apkg");
    let next = output.join("spanish-v2.apkg");
    vocabulary("hello")?.build(BuildOptions::to(&previous))?;
    let project = vocabulary("hello; hi")?;
    let comparison = project.compare(CompareOptions::against(&previous))?;
    ensure!(comparison.policy().allows_publication());
    let built = project.build(BuildOptions::to(&next).update_from(&previous))?;
    verify(&built, 1, 1, 0)?;
    ensure!(built.report().comparison().is_some());
    Ok(())
}
```
<!-- /source -->

`compare` completes successfully even when findings would block publication.
Read `comparison.policy().allows_publication()` separately from the original
risk level and evidence. `build(...update_from(...))` runs the same analysis and
returns `BuildError` if its policy blocks the candidate, before publication.

## Policy and inspection limits

This full program selects a stricter threshold and uses the same configuration
for comparison and update:

```rust
use ankiforge::{BuildOptions, Note, Project};
use ankiforge::build::InspectLimits;
use ankiforge::update::{CompareOptions, RiskLevel, UpdatePolicy};

fn main() -> anyhow::Result<()> {
    let mut first = Project::new("policy-example")?;
    first.add("cell", Note::basic("Cell?", "Unit of life"))?;
    first.build(BuildOptions::to("policy-v1.apkg"))?;
    let mut next = Project::new("policy-example")?;
    next.add("cell", Note::basic("Cell?", "The basic unit of life"))?;
    let mut limits = InspectLimits::default();
    limits.max_collection_bytes = 1 << 30;
    let policy = UpdatePolicy::default().fail_on(RiskLevel::Medium);
    let comparison = next.compare(CompareOptions::against("policy-v1.apkg")
        .inspect_limits(limits.clone()).update_policy(policy.clone()))?;
    assert!(comparison.policy().allows_publication());
    next.build(BuildOptions::to("policy-v2.apkg").update_from("policy-v1.apkg")
        .inspect_limits(limits).update_policy(policy))?;
    Ok(())
}
```

The default policy blocks High and Critical findings. After reviewing a specific
category, a caller can deliberately choose
`UpdatePolicy::default().allow(RiskCode::NoteRemoved)`. It accepts **all findings
in that category**, retaining their original level and evidence. It does not
claim to approve selected notes individually. An allowance with no matching
finding produces a warning; unknown risk codes are rejected. Do not automatically
allow every code returned by a comparison.

Missing or corrupt evidence, namespace mismatch, invalid candidate data and
inspection limits are hard errors, not allowable risk categories. Comparison
returns an error with completed observations when analysis cannot finish.
Explicit policy on a create request without `update_from` is a configuration
error regardless of setter order.

## Evidence and identity

The package includes `ankiforge-identity.json` with the
`ankiforge-identity-v1` format. It binds model/config IDs, note GUIDs, historical
field/template slots, card/mask ordinals, revisions and media to the actual
collection and payloads. Anki re-export does not preserve this evidence and is
not a usable update baseline. Missing evidence is never silently reconstructed.

Display-name and content edits preserve identity keys. Unchanged content keeps
its baseline revision; changed content advances it. Reordering fields/templates
preserves historical mappings. Retired identities remain reserved so that
restoring the same key can reuse them. A note key cannot move to another model,
and a model key cannot change between normal and Cloze kinds.

## Client import behavior and release review

An APKG is a distribution package, not deletion synchronization. Omitting notes,
fields, templates or masks does not delete existing learner data. Field/template
schema changes and sort-field changes are high-risk because Anki merge and import
settings affect their behavior. Schema changes can copy models or update target
note timestamps before note-content comparison.

Test first import and updates into an existing collection on the clients you
support. Check fields, GUIDs, card sets, local edits and existing scheduling with
the intended merge and note-update settings. Stable identity alone does not
promise that newer local edits are overwritten or deleted history is restored.

Write candidates to a path separate from the archived baseline. After review,
distribute the accepted package and retain it as the next release's evidence.
See [build guarantees](build-guarantees.md) for ownership and publication facts.
