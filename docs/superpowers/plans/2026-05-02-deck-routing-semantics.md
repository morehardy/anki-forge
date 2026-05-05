# Deck Routing Semantics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make deck routing match Anki semantics: each generated card uses `template.target_deck_name` when present, otherwise the note's `deck_name`, while product-level default deck metadata no longer overwrites explicit note decks.

**Architecture:** Keep `note.deck_name` and `template.target_deck_name` as separate layers all the way through authoring, normalization, staging, APKG materialization, and inspection. Replace the writer's template-only deck-id registry with a package deck registry that includes note decks and template override decks, then use separate helpers for template config deck ids and card row deck ids. Preserve Anki's native template config convention: no template override serializes as target deck id `0`.

**Tech Stack:** Rust workspace, `authoring_core` normalized IR models, `writer_core` staging/APKG/inspect pipeline, `anki_forge` product lowering, SQLite assertions through `rusqlite`, cargo integration tests.

---

## Source Anchors

- Anki card generation resolves the card deck as template target deck first, then extracted/import deck: `docs/source/anki/rslib/src/notetype/cardgen.rs:137-140`.
- Anki add-card code also falls back from generated card deck to the caller target deck: `docs/source/anki/rslib/src/notetype/cardgen.rs:288-289`.
- Text import passes the selected or per-row deck id as `ctx.deck_id` when creating and updating cards: `docs/source/anki/rslib/src/import_export/text/import.rs:372-379` and `docs/source/anki/rslib/src/import_export/text/import.rs:420-424`.
- Template config stores no deck override as `None`/`0`, not as the note deck: `docs/source/anki/rslib/src/notetype/schema11.rs:407-411`.
- Anki text export derives a note deck column from the first existing card's deck: `docs/source/anki/rslib/src/storage/deck/all_decks_of_search_notes.sql`.

## Desired Semantics

The writer should apply this rule:

```text
template_config.target_deck_id = deck_id(template.target_deck_name) if template.target_deck_name is present, else 0
card.deck_name = template.target_deck_name if present, else note.deck_name
card.did = deck_id(card.deck_name)
```

The current Rust product API has `ProductNote.deck_name` as a required field. Therefore `ProductDocument.default_deck_name` must not replace explicit product note decks during lowering. It remains product-level default metadata for callers that construct notes from a package/deck-level default before adding them.

## File Structure

- Modify `writer_core/tests/build_tests.rs` - add failing integration tests that inspect generated SQLite `cards.did` and `decks.name`.
- Modify `writer_core/src/staging.rs` - replace template-only deck id collection with package-level deck id resolution and keep template-target observation behavior.
- Modify `writer_core/src/apkg.rs` - create all note/template decks and route card rows through `template override ?? note deck`.
- Modify `anki_forge/tests/product_lowering_tests.rs` - add a failing product lowering test for explicit note deck preservation.
- Modify `anki_forge/src/product/lowering.rs` - stop letting `default_deck_name` overwrite `ProductNote.deck_name`.
- Modify `writer_core/tests/inspect_tests.rs` - add APKG inspection coverage for note/card deck observations.
- Modify `writer_core/src/inspect.rs` - report note/card deck names and recover APKG note deck from the first card's deck instead of hard-coding `Default`.
- Modify `contracts/semantics/build.md` - document the three-layer routing rule and template config distinction.
- Modify `contracts/semantics/normalization.md` - document that normalization preserves note deck and template target deck as independent fields.
- Modify `contracts/semantics/inspect.md` - document note/card deck observations and APKG first-card reconstruction.

---

### Task 1: Add Writer Build Tests For Card Deck Routing

**Files:**
- Modify: `writer_core/tests/build_tests.rs`

- [ ] **Step 1: Add failing test for note deck without template override**

Insert this test after `latest_collection_uses_explicit_normalized_note_mtime_when_present` in `writer_core/tests/build_tests.rs`:

```rust
#[test]
fn latest_collection_places_cards_in_note_deck_when_template_has_no_target_deck() {
    let root = unique_artifact_root("note-deck-routing");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/note-deck-routing");
    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0].deck_name = "Biology::Cells".into();

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let conn = latest_collection_from_built_apkg(&root);
    let card_deck_name: String = conn
        .query_row(
            "select decks.name
             from cards
             join decks on decks.id = cards.did
             join notes on notes.id = cards.nid
             where notes.guid = 'note-1' and cards.ord = 0",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(card_deck_name, "Biology::Cells");
}
```

- [ ] **Step 2: Run the new note-deck test to verify it fails**

Run:

```bash
cargo test -p writer_core --test build_tests latest_collection_places_cards_in_note_deck_when_template_has_no_target_deck -v
```

Expected: FAIL. The assertion should show the card still resolves to `"Default"` because `writer_core/src/apkg.rs` currently falls back to deck id `1` when `template.target_deck_name` is absent.

- [ ] **Step 3: Add failing test for template override precedence**

Insert this test immediately after `latest_collection_places_cards_in_note_deck_when_template_has_no_target_deck`:

```rust
#[test]
fn latest_collection_template_target_deck_overrides_note_deck_for_cards() {
    let root = unique_artifact_root("template-deck-routing");
    let target =
        BuildArtifactTarget::new(root.clone(), "artifacts/phase3/template-deck-routing");
    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0].deck_name = "Biology::Cells".into();
    normalized.notetypes[0].templates[0].target_deck_name = Some("Biology::Overrides".into());

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let conn = latest_collection_from_built_apkg(&root);
    let card_deck_name: String = conn
        .query_row(
            "select decks.name
             from cards
             join decks on decks.id = cards.did
             join notes on notes.id = cards.nid
             where notes.guid = 'note-1' and cards.ord = 0",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(card_deck_name, "Biology::Overrides");

    let deck_names: std::collections::BTreeSet<String> = conn
        .prepare("select name from decks order by name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|row| row.unwrap())
        .collect();

    assert!(deck_names.contains("Biology::Cells"));
    assert!(deck_names.contains("Biology::Overrides"));
}
```

- [ ] **Step 4: Run the template override test to verify it fails**

Run:

```bash
cargo test -p writer_core --test build_tests latest_collection_template_target_deck_overrides_note_deck_for_cards -v
```

Expected: FAIL. The card may already land in `"Biology::Overrides"`, but the note deck `"Biology::Cells"` should be missing from the `decks` table because the writer only creates template target decks today.

- [ ] **Step 5: Commit the failing tests**

```bash
git add writer_core/tests/build_tests.rs
git commit -m "test: capture deck routing semantics"
```

---

### Task 2: Resolve All Package Deck IDs In Staging

**Files:**
- Modify: `writer_core/src/staging.rs`

- [ ] **Step 1: Replace the template-only resolver with a package deck resolver**

In `writer_core/src/staging.rs`, replace the entire `resolve_template_target_deck_ids()` function with this function:

```rust
pub(crate) fn resolve_deck_ids(normalized_ir: &NormalizedIr) -> BTreeMap<String, i64> {
    let mut names: BTreeSet<String> = normalized_ir
        .notes
        .iter()
        .map(|note| note.deck_name.clone())
        .collect();

    names.extend(normalized_ir.notetypes.iter().flat_map(|notetype| {
        notetype
            .templates
            .iter()
            .filter_map(|template| template.target_deck_name.clone())
    }));

    let mut resolved = BTreeMap::from([("Default".into(), 1_i64)]);
    let mut occupied_ids: BTreeSet<i64> = BTreeSet::from([1_i64]);

    names.remove("Default");

    for name in names {
        let mut next_id = 2_i64;
        while occupied_ids.contains(&next_id) {
            next_id += 1;
        }
        resolved.insert(name, next_id);
        occupied_ids.insert(next_id);
    }

    resolved
}
```

- [ ] **Step 2: Update template target deck observation resolution**

In `writer_core/src/staging.rs`, change `resolve_template_target_decks()` so it calls the new resolver:

```rust
pub(crate) fn resolve_template_target_decks(
    normalized_ir: &NormalizedIr,
) -> Vec<ResolvedTemplateTargetDeck> {
    let deck_ids = resolve_deck_ids(normalized_ir);
    let mut resolved = vec![];

    for notetype in &normalized_ir.notetypes {
        for template in &notetype.templates {
            let Some(target_deck_name) = template.target_deck_name.as_ref() else {
                continue;
            };
            let resolved_target_deck_id = deck_ids.get(target_deck_name).copied().unwrap_or(1);
            resolved.push(ResolvedTemplateTargetDeck {
                notetype_id: notetype.id.clone(),
                template_name: template.name.clone(),
                target_deck_name: target_deck_name.clone(),
                resolved_target_deck_id,
            });
        }
    }

    resolved
}
```

- [ ] **Step 3: Run the writer tests to expose the remaining APKG call-site errors**

Run:

```bash
cargo test -p writer_core --test build_tests latest_collection_places_cards_in_note_deck_when_template_has_no_target_deck -v
```

Expected: FAIL to compile with an unresolved import or unresolved function name in `writer_core/src/apkg.rs`, because `resolve_template_target_deck_ids` was replaced.

- [ ] **Step 4: Commit the staging resolver change**

```bash
git add writer_core/src/staging.rs
git commit -m "refactor: resolve package deck ids"
```

---

### Task 3: Route APKG Cards Through Template Override Then Note Deck

**Files:**
- Modify: `writer_core/src/apkg.rs`
- Test: `writer_core/tests/build_tests.rs`

- [ ] **Step 1: Update the staging import**

At the top of `writer_core/src/apkg.rs`, replace this import:

```rust
use crate::staging::{
    load_normalized_ir_from_staging_manifest, resolve_template_target_deck_ids,
};
```

with:

```rust
use crate::staging::{load_normalized_ir_from_staging_manifest, resolve_deck_ids};
```

- [ ] **Step 2: Use the package deck registry in `populate_latest_collection()`**

In `writer_core/src/apkg.rs`, replace:

```rust
let template_target_deck_ids = resolve_template_target_deck_ids(normalized_ir);
```

with:

```rust
let deck_ids = resolve_deck_ids(normalized_ir);
```

Then replace every remaining `template_target_deck_ids` variable use inside `populate_latest_collection()` with `deck_ids`.

- [ ] **Step 3: Keep deck table creation package-wide**

In `writer_core/src/apkg.rs`, make the deck insertion loop read exactly:

```rust
for (deck_name, deck_id) in &deck_ids {
    if deck_name == "Default" {
        continue;
    }
    conn.execute(
        "insert into decks (id, name, mtime_secs, usn, common, kind) values (?1, ?2, 0, 0, ?3, ?4)",
        rusqlite::params![
            deck_id,
            deck_name,
            default_deck_common_bytes(),
            default_deck_kind_bytes(default_deck_config_id)
        ],
    )?;
}
```

- [ ] **Step 4: Preserve template config target deck semantics**

In the template insertion loop in `writer_core/src/apkg.rs`, keep the default id as `0_i64`:

```rust
for (template_ord, template) in notetype.templates.iter().enumerate() {
    let target_deck_id = resolve_template_target_deck_id(template, &deck_ids, 0_i64);
    conn.execute(
        "insert into templates (ntid, ord, name, mtime_secs, usn, config) values (?1, ?2, ?3, 0, 0, ?4)",
        rusqlite::params![
            ntid,
            template.ord.unwrap_or(template_ord as u32) as i64,
            template.name,
            encode_template_config(template, target_deck_id)
        ],
    )?;
}
```

- [ ] **Step 5: Route cards with a dedicated helper**

In the card insertion loop in `writer_core/src/apkg.rs`, replace:

```rust
let target_deck_id =
    resolve_template_target_deck_id(template, &deck_ids, 1_i64);
```

with:

```rust
let target_deck_id = resolve_card_deck_id(note, template, &deck_ids);
```

- [ ] **Step 6: Replace the deck helper block**

In `writer_core/src/apkg.rs`, replace the existing `resolve_template_target_deck_id()` helper with these helpers:

```rust
fn resolve_card_deck_id(
    note: &NormalizedNote,
    template: &authoring_core::NormalizedTemplate,
    deck_ids: &std::collections::BTreeMap<String, i64>,
) -> i64 {
    let deck_name = template
        .target_deck_name
        .as_deref()
        .unwrap_or(note.deck_name.as_str());
    resolve_deck_id(deck_name, deck_ids, 1_i64)
}

fn resolve_template_target_deck_id(
    template: &authoring_core::NormalizedTemplate,
    deck_ids: &std::collections::BTreeMap<String, i64>,
    default_id: i64,
) -> i64 {
    template
        .target_deck_name
        .as_deref()
        .map(|deck_name| resolve_deck_id(deck_name, deck_ids, default_id))
        .unwrap_or(default_id)
}

fn resolve_deck_id(
    deck_name: &str,
    deck_ids: &std::collections::BTreeMap<String, i64>,
    default_id: i64,
) -> i64 {
    deck_ids.get(deck_name).copied().unwrap_or(default_id)
}
```

- [ ] **Step 7: Run the note-deck test**

Run:

```bash
cargo test -p writer_core --test build_tests latest_collection_places_cards_in_note_deck_when_template_has_no_target_deck -v
```

Expected: PASS.

- [ ] **Step 8: Run the template override test**

Run:

```bash
cargo test -p writer_core --test build_tests latest_collection_template_target_deck_overrides_note_deck_for_cards -v
```

Expected: PASS.

- [ ] **Step 9: Run the full build test target**

Run:

```bash
cargo test -p writer_core --test build_tests -v
```

Expected: PASS.

- [ ] **Step 10: Commit the writer routing implementation**

```bash
git add writer_core/src/staging.rs writer_core/src/apkg.rs writer_core/tests/build_tests.rs
git commit -m "fix: route cards through note decks"
```

---

### Task 4: Preserve Explicit Product Note Decks During Lowering

**Files:**
- Modify: `anki_forge/tests/product_lowering_tests.rs`
- Modify: `anki_forge/src/product/lowering.rs`

- [ ] **Step 1: Add a failing product lowering test**

Append this test to `anki_forge/tests/product_lowering_tests.rs`:

```rust
#[test]
fn product_default_deck_does_not_overwrite_explicit_note_deck() {
    let plan = ProductDocument::new("multi-deck-doc")
        .with_default_deck("Package::Default")
        .with_basic("basic-main")
        .add_basic_note(
            "basic-main",
            "note-1",
            "Per Note::Deck",
            "front",
            "back",
        )
        .lower()
        .expect("lower should succeed");

    let note = plan
        .authoring_document
        .notes
        .first()
        .expect("lower should produce one note");

    assert_eq!(note.deck_name, "Per Note::Deck");
}
```

- [ ] **Step 2: Run the product lowering test to verify it fails**

Run:

```bash
cargo test -p anki_forge --test product_lowering_tests product_default_deck_does_not_overwrite_explicit_note_deck -v
```

Expected: FAIL. The assertion should show `"Package::Default"` on the left and `"Per Note::Deck"` on the right.

- [ ] **Step 3: Replace product note deck selection**

In `anki_forge/src/product/lowering.rs`, replace the `let deck_name = document.default_deck_name()...` block inside `for note in document.notes()` with this match:

```rust
let deck_name = match note {
    ProductNote::Basic(basic) => basic.deck_name.clone(),
    ProductNote::Cloze(cloze) => cloze.deck_name.clone(),
    ProductNote::ImageOcclusion(io) => io.deck_name.clone(),
    ProductNote::Custom(custom) => custom.deck_name.clone(),
};
```

- [ ] **Step 4: Run the focused product lowering test**

Run:

```bash
cargo test -p anki_forge --test product_lowering_tests product_default_deck_does_not_overwrite_explicit_note_deck -v
```

Expected: PASS.

- [ ] **Step 5: Run the full product lowering test target**

Run:

```bash
cargo test -p anki_forge --test product_lowering_tests -v
```

Expected: PASS.

- [ ] **Step 6: Commit the product lowering fix**

```bash
git add anki_forge/src/product/lowering.rs anki_forge/tests/product_lowering_tests.rs
git commit -m "fix: preserve product note decks"
```

---

### Task 5: Report Deck Routing In APKG Inspection

**Files:**
- Modify: `writer_core/tests/inspect_tests.rs`
- Modify: `writer_core/src/inspect.rs`

- [ ] **Step 1: Add an inspect test for note and card deck names**

Insert this test after `inspect_apkg_reports_complete_observations_and_counts` in `writer_core/tests/inspect_tests.rs`:

```rust
#[test]
fn inspect_apkg_reports_note_and_card_deck_names() {
    let root = unique_artifact_root("inspect-apkg-deck-routing");
    let target =
        BuildArtifactTarget::new(root.clone(), "artifacts/phase3/inspect-apkg-deck-routing");
    let mut normalized_ir = sample_basic_normalized_ir();
    normalized_ir.notes[0].deck_name = "Biology::Cells".into();

    build(
        &normalized_ir,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let report = inspect_apkg(root.join("package.apkg")).unwrap();

    let note = report
        .observations
        .references
        .iter()
        .find(|value| value["selector"] == "note[id='note-1']")
        .expect("note observation");
    assert_eq!(note["deck_name"], "Biology::Cells");

    let card = report
        .observations
        .references
        .iter()
        .find(|value| value["selector"] == "card[note_id='note-1'][ord=0]")
        .expect("card observation");
    assert_eq!(card["deck_name"], "Biology::Cells");
}
```

- [ ] **Step 2: Run the inspect test to verify it fails**

Run:

```bash
cargo test -p writer_core --test inspect_tests inspect_apkg_reports_note_and_card_deck_names -v
```

Expected: FAIL. The report currently omits `deck_name` from note/card observations and APKG inspection reconstructs normalized notes with `"Default"`.

- [ ] **Step 3: Add `deck_name` to note observations**

In `writer_core/src/inspect.rs`, change the note observation JSON block in `build_observations()` to include `deck_name`:

```rust
note_entries.push(json!({
    "selector": format!("note[id='{}']", note_id),
    "id": note_id,
    "notetype_id": notetype_id,
    "deck_name": note.deck_name.as_str(),
    "tags": &note.tags,
    "fields": &note.fields,
    "evidence_refs": [format!("note:{}", note_id)],
}));
```

- [ ] **Step 4: Add final `deck_name` to card observations**

In `writer_core/src/inspect.rs`, replace the card observation loop body with:

```rust
for (ord, template) in notetype.templates.iter().enumerate() {
    let template_name = template.name.as_str();
    let card_deck_name = template
        .target_deck_name
        .as_deref()
        .unwrap_or(note.deck_name.as_str());
    card_entries.push(json!({
        "selector": format!("card[note_id='{}'][ord={}]", note_id, ord),
        "note_id": note_id,
        "ord": ord,
        "template_name": template_name,
        "deck_name": card_deck_name,
        "evidence_refs": [format!("card:{}:{}", note_id, ord)],
    }));
}
```

- [ ] **Step 5: Recover APKG note decks from first card deck**

In `writer_core/src/inspect.rs`, inside latest-collection loading, insert this block immediately before the existing `let mut note_rows = conn.prepare("select id, guid, mid, mod, tags, flds from notes order by id")?;` line:

```rust
let mut note_decks_by_row_id = BTreeMap::<i64, String>::new();
let mut note_deck_rows = conn.prepare(
    "select cards.nid, decks.name
     from cards
     left join decks on decks.id = cards.did
     where cards.ord = (
         select min(inner_cards.ord)
         from cards inner_cards
         where inner_cards.nid = cards.nid
     )
     order by cards.nid",
)?;
for row in note_deck_rows.query_map([], |row| {
    Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
})? {
    let (note_id, deck_name) = row?;
    note_decks_by_row_id.insert(note_id, deck_name.unwrap_or_else(|| "Default".into()));
}
```

- [ ] **Step 6: Use the recovered deck when reconstructing notes**

In the `query_map()` closure for note rows in `writer_core/src/inspect.rs`, replace:

```rust
let _id: i64 = row.get(0)?;
```

with:

```rust
let id: i64 = row.get(0)?;
```

Then replace:

```rust
deck_name: "Default".into(),
```

with:

```rust
deck_name: note_decks_by_row_id
    .get(&id)
    .cloned()
    .unwrap_or_else(|| "Default".into()),
```

- [ ] **Step 7: Run the focused inspect test**

Run:

```bash
cargo test -p writer_core --test inspect_tests inspect_apkg_reports_note_and_card_deck_names -v
```

Expected: PASS.

- [ ] **Step 8: Run the full inspect test target**

Run:

```bash
cargo test -p writer_core --test inspect_tests -v
```

Expected: PASS.

- [ ] **Step 9: Commit inspect routing observations**

```bash
git add writer_core/src/inspect.rs writer_core/tests/inspect_tests.rs
git commit -m "fix: inspect deck routing"
```

---

### Task 6: Document Deck Routing Contracts

**Files:**
- Modify: `contracts/semantics/build.md`
- Modify: `contracts/semantics/normalization.md`
- Modify: `contracts/semantics/inspect.md`

- [ ] **Step 1: Update build semantics**

In `contracts/semantics/build.md`, replace the final `Phase 5A` bullet:

```markdown
- template target deck names are resolved to stable deck ids during staging and
  reused when writing template configs and card rows
```

with:

```markdown
- note deck names and template target deck names are resolved into one stable
  package deck registry during staging and APKG materialization
- template configs only receive `target_deck_id` when
  `template.target_deck_name` is present; templates without deck override keep
  Anki's native `0`/none target-deck representation
- card rows use Anki's routing order:
  `template.target_deck_name ?? note.deck_name`; this keeps per-note deck
  import semantics separate from template deck override semantics
```

- [ ] **Step 2: Update normalization semantics**

In `contracts/semantics/normalization.md`, insert this paragraph after the list ending with ``field_metadata` entries including `field_name`, `label`, and `role_hint``:

```markdown
Normalization preserves `notes[].deck_name` independently from
`notetypes[].templates[].target_deck_name`. The note deck represents the deck
selected by authoring/import input for new cards from that note, while the
template target deck represents Anki's per-template Deck Override. Normalization
must not copy a note deck into template target deck fields or copy a template
target deck into note deck fields.
```

- [ ] **Step 3: Update inspect semantics**

In `contracts/semantics/inspect.md`, append this paragraph after the existing `template_target_decks` bullet list:

```markdown
Deck routing observations expose `deck_name` on note and card reference entries.
For staging sources, note deck names come directly from normalized IR and card
deck names are computed as `template.target_deck_name ?? note.deck_name`. For
APKG sources, the original note-level import deck is not stored separately in
Anki's collection schema, so inspection reconstructs `notes[].deck_name` from
the first existing card deck, matching Anki's text export behavior.
```

- [ ] **Step 4: Verify the contract text exists**

Run:

```bash
rg -n "template\\.target_deck_name \\?\\? note\\.deck_name|first existing card deck|must not copy a note deck" contracts/semantics
```

Expected: PASS with matches in `contracts/semantics/build.md`, `contracts/semantics/normalization.md`, and `contracts/semantics/inspect.md`.

- [ ] **Step 5: Commit contract docs**

```bash
git add contracts/semantics/build.md contracts/semantics/normalization.md contracts/semantics/inspect.md
git commit -m "docs: clarify deck routing semantics"
```

---

### Task 7: Final Verification

**Files:**
- Verify: `writer_core/src/staging.rs`
- Verify: `writer_core/src/apkg.rs`
- Verify: `writer_core/src/inspect.rs`
- Verify: `anki_forge/src/product/lowering.rs`
- Verify: `contracts/semantics/build.md`
- Verify: `contracts/semantics/normalization.md`
- Verify: `contracts/semantics/inspect.md`

- [ ] **Step 1: Run writer build tests**

Run:

```bash
cargo test -p writer_core --test build_tests -v
```

Expected: PASS.

- [ ] **Step 2: Run writer inspect tests**

Run:

```bash
cargo test -p writer_core --test inspect_tests -v
```

Expected: PASS.

- [ ] **Step 3: Run product lowering tests**

Run:

```bash
cargo test -p anki_forge --test product_lowering_tests -v
```

Expected: PASS.

- [ ] **Step 4: Search for stale template-only resolver names**

Run:

```bash
rg -n "resolve_template_target_deck_ids|template_target_deck_ids" writer_core/src writer_core/tests
```

Expected: exit code `1` with no matches.

- [ ] **Step 5: Search for product default deck overwrites**

Run:

```bash
rg -n "default_deck_name\\(\\).*unwrap_or_else|default_deck_name\\(\\).*deck_name" anki_forge/src/product anki_forge/tests
```

Expected: exit code `1` with no matches.

- [ ] **Step 6: Inspect the git diff**

Run:

```bash
git diff --stat
git diff -- writer_core/src/staging.rs writer_core/src/apkg.rs writer_core/src/inspect.rs anki_forge/src/product/lowering.rs
```

Expected: The diff should only include deck routing resolver changes, card deck id selection, inspection deck observation changes, and product lowering deck preservation.

- [ ] **Step 7: Commit final verification notes if any files changed after Task 6**

If Step 6 shows changes not already committed by prior tasks, run:

```bash
git add writer_core/src/staging.rs writer_core/src/apkg.rs writer_core/src/inspect.rs writer_core/tests/build_tests.rs writer_core/tests/inspect_tests.rs anki_forge/src/product/lowering.rs anki_forge/tests/product_lowering_tests.rs contracts/semantics/build.md contracts/semantics/normalization.md contracts/semantics/inspect.md
git commit -m "test: verify deck routing semantics"
```

Expected: A commit is created only when Step 6 shows uncommitted changes.

---

## Self-Review

**Spec coverage:** The plan covers the three-layer rule with writer build tests, package deck id resolution, APKG card row routing, template config preservation, product lowering semantics, APKG inspection, and contract docs.

**Placeholder scan:** The plan contains concrete paths, code snippets, commands, and expected outcomes. It does not rely on unspecified future code.

**Type consistency:** The plan consistently uses existing fields `NormalizedNote.deck_name`, `NormalizedTemplate.target_deck_name`, `ProductNote::* deck_name`, `ResolvedTemplateTargetDeck.resolved_target_deck_id`, `cards.did`, and `decks.name`.
