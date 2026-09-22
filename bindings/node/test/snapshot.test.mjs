import test from "node:test";
import assert from "node:assert/strict";
import {
  Field,
  Template,
  NoteType,
  IdentityRecipe,
  GenerationRule,
} from "../dist/index.mjs";

test("value descriptions expose canonical Rust keys, flags, identity and template definitions as frozen snapshots", () => {
  const field = new Field("Front Side", {
    identity: true,
    sort: true,
    required: true,
  });
  assert.deepEqual(field.describe(), {
    key: "front_side",
    name: "Front Side",
    identity: true,
    sort: true,
    required: true,
    optional: false,
    keyAutoDerived: true,
  });
  assert.equal(
    new Field("Back", { key: "answer", optional: true }).describe()
      .keyAutoDerived,
    false,
  );
  const identity = IdentityRecipe.fields(["b", "a", "b"]);
  assert.deepEqual(identity.describe(), { fieldKeys: ["a", "b"] });
  const rule = GenerationRule.all(["front", "back"]);
  assert.deepEqual(rule.describe(), { kind: "all", fields: ["front", "back"] });
  assert.deepEqual(GenerationRule.any(["front"]).describe(), {
    kind: "any",
    fields: ["front"],
  });
  assert.deepEqual(GenerationRule.cloze("text").describe(), {
    kind: "cloze",
    field: "text",
  });
  assert.deepEqual(GenerationRule.ankiDefault().describe(), {
    kind: "anki_default",
  });
  const template = new Template("Card One", {
    front: "{{Front Side}}",
    back: "{{FrontSide}}",
    browserFront: "browser",
    browserBack: "answer",
    targetDeck: "Other",
    generateWhen: rule,
  });
  assert.deepEqual(template.describe(), {
    key: "card_one",
    name: "Card One",
    front: "{{Front Side}}",
    back: "{{FrontSide}}",
    browserFront: "browser",
    browserBack: "answer",
    targetDeck: "Other",
    generateWhen: rule.describe(),
  });
  const noteType = NoteType.customCloze("custom", "text", {
    name: "Custom",
    fields: [field],
    templates: [template],
    css: ".card {}",
    identity,
  });
  const view = noteType.describe();
  assert.equal(view.kind, "cloze");
  assert.equal(view.clozeField, "text");
  assert.equal(view.id, "custom");
  assert.equal(view.name, "Custom");
  assert.equal(view.css, ".card {}");
  assert.deepEqual(view.fields, [field.describe()]);
  assert.deepEqual(view.templates, [template.describe()]);
  assert.deepEqual(view.identity, identity.describe());
  assert.throws(() => (view.fields[0].key = "changed"), TypeError);
  assert.throws(() => view.identity.fieldKeys.push("changed"), TypeError);
  // Observation must not perform Project.addNoteType validation on this incomplete definition.
  assert.equal(
    NoteType.custom("incomplete", { fields: [], templates: [] }).describe()
      .kind,
    "normal",
  );
});

test("Note descriptions render stock, HTML and media content in Rust without adding the note to a Project", async () => {
  const { Project, Note } = await import("../dist/index.mjs");
  const project = new Project("Observe");
  const media = await project.media.addBytes(
    "icon.svg",
    Buffer.from('<svg xmlns="http://www.w3.org/2000/svg"/>'),
  );
  const note = Note.basic("<b>text &</b>", "back", {
    stableId: "one",
    deckName: "Other",
    tags: ["one", "two"],
    identity: ["Back", "Front", "Back"],
  }).image("Back", media);
  const snapshot = note.describe();
  assert.equal(snapshot.noteTypeId, "basic");
  assert.equal(snapshot.stableId, "one");
  assert.equal(snapshot.deckName, "Other");
  assert.deepEqual(snapshot.tags, ["one", "two"]);
  assert.deepEqual(snapshot.identity, { fieldKeys: ["Back", "Front"] });
  assert.deepEqual(snapshot.renderedFields, {
    Front: "&lt;b&gt;text &amp;&lt;/b&gt;",
    Back: '<img src="icon.svg">',
  });
  assert.throws(() => (snapshot.renderedFields.Front = "changed"), TypeError);
  assert.equal(
    note.html("Front", "<b>raw</b>").describe().renderedFields.Front,
    "<b>raw</b>",
  );
  assert.equal(
    Note.cloze("{{c1::one}}", { backExtra: "hint" }).describe().renderedFields
      .Text,
    "{{c1::one}}",
  );
  assert.equal(
    Note.custom("not-registered").text("a", "<").describe().renderedFields.a,
    "&lt;",
  );
});

test("Deck descriptions expose canonical identities and detached notes while reserving a consistent snapshot", async () => {
  const { Deck, ProjectBusyError } = await import("../dist/index.mjs");
  const deck = new Deck("Descriptions", {
    stableId: "deck",
    basicIdentity: ["back", "front", "back"],
  });
  deck.basic("<b>front</b>", "back", {
    tags: ["tag"],
    identityOverride: { fields: ["back"], reasonCode: "reason" },
  });
  deck.cloze("{{c1::one}}", { stableId: "explicit", extra: "hint" });
  const reading = deck.describe();
  assert.throws(() => deck.basic("busy", "note"), ProjectBusyError);
  const snapshot = await reading;
  assert.equal(snapshot.name, "Descriptions");
  assert.equal(snapshot.stableId, "deck");
  assert.deepEqual(snapshot.identityPolicy, { basic: ["front", "back"] });
  const [basic, cloze] = snapshot.notes;
  assert.equal(basic.kind, "basic");
  assert.equal(basic.front, "<b>front</b>");
  assert.equal(basic.back, "back");
  assert.deepEqual(basic.tags, ["tag"]);
  assert.deepEqual(basic.identityOverride, {
    fields: ["back"],
    reasonCode: "reason",
  });
  assert.equal(basic.resolvedIdentity.usedOverride, true);
  assert.equal(basic.id, basic.resolvedIdentity.stableId);
  assert.equal(cloze.kind, "cloze");
  assert.equal(cloze.extra, "hint");
  assert.equal(cloze.resolvedIdentity.provenance, "explicit_stable_id");
  assert.throws(() => snapshot.notes.push({}), TypeError);
  deck.basic("later", "answer", { stableId: "later" });
  assert.equal(snapshot.notes.length, 2);
  assert.equal((await deck.describe()).notes.length, 3);
});
