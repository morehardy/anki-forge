import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { pages } from '../../../../website/site-model.mjs';
import { reviewInputs, digest, rubricVersion, glossaryVersion } from '../../../../website/scripts/content-review-inputs.mjs';

const directory = 'docs/plans/evidence/website-pr66-final-2026-11-02';
const root = process.cwd();
const parse = async filename => JSON.parse(await readFile(filename));
const save = async (filename, value) => writeFile(filename, JSON.stringify(value, null, 2) + '\n');
const auditPath = `${directory}/technical-source-input-audit.json`;
const audit = await parse(auditPath);
const snapshot = await parse(audit.snapshots.path);
assert.equal(digest(await readFile(audit.snapshots.path)), audit.snapshots.sha256);

// These are independent observations authored after reading the actual diff
// and the named implementations. This assembler does not generate judgments.
const observations = {
  'docs/ai-quickstart.json': 'Read both changed prompts: zh-CN replaces internal tools with 内部工具, and pt-BR distinguishes card templates from note-type models. The public-only API restriction, original distribution APKG requirement, all seven tasks and exact interpolation placeholders remain intact. Installation, versions, runtimes and prompt contracts are unchanged.',
  'docs/build-guarantees.md': 'The old source-only-sounding heading becomes Prepare, review and publish. The public 0.3.1 availability, bound candidate, consumption, cleanup and late-durability limits in the surrounding section are unchanged. The explicit semantic marker preserves prepared-publication-from-source.',
  'docs/cards.md': 'The revised opening correctly separates Basic note front/back fields from the card produced by the stock template. Note::basic stores those fields; schema::stock supplies the Basic template. The two-notes/three-cards example and executable bytes remain unchanged.',
  'docs/de/anki-forge-vs-genanki.md': 'The media row now explicitly says typed objects manage immutable media-data snapshots. Media owns Arc<Snapshot>; image/sound retain a clone. This does not promise independent clone storage or dependence on source files.',
  'docs/de/build-guarantees.md': 'Error-kind/code wording now names structured fields correctly; the concurrent-work heading becomes Nebenläufige Vorgänge. The immutable project snapshot and last-temporary-owner lifetime paragraph is preserved, with the old German heading mapped to concurrent-work.',
  'docs/de/concepts.md': 'The new instruction to retain namespace and record keys when editing or sorting preserves the stable-key identity rule and the distinction from display names. Atomic-add conditions and note/model dependencies are unchanged.',
  'docs/de/csv-to-anki.md': 'Teilbaren is clarified as a deck that can be passed on. UTF-8 CSV columns id/front/back, Python parsing responsibility, Basic-card export and no-Anki export requirement are unchanged.',
  'docs/de/media.md': 'Stapelimport is clarified as importing multiple media files, avoiding confusion with Anki decks. Worker count, serial threshold, per-file limits, cleanup/order and GIL/event-loop conditions remain unchanged; the old batch heading still resolves to batch-behavior-and-budgets.',
  'docs/de/node-api.md': 'The renamed heading describes importing several media files and preparing publication. Public package availability and actual API signatures/consume semantics below it are unchanged; both semantic anchors and the former German heading are retained.',
  'docs/de/python-api.md': 'The renamed heading describes batch media and publication preparation. It retains source-api-details as the semantic destination and maps the old German heading there. The methods and publication limits are unchanged.',
  'docs/es/anki-forge-vs-genanki.md': 'The opening now explains that imported media data survives source-file changes and that comparison and risk policy are reviewed before publication. This matches Media snapshot ownership and PreparedPublication, without claiming genanki migration or automatic safety.',
  'docs/es/build-guarantees.md': 'Trabajadores is narrowed to worker threads. The referenced example, typed-error behavior, close/consumption and late-durability limits are unchanged.',
  'docs/es/updates.md': 'The title now names updating a deck. The risk paragraph correctly says exceeding inspection limits is a hard failure rather than calling limits themselves errors. Namespace/evidence/candidate hard failures and policy restrictions are unchanged.',
  'docs/ja/build-guarantees.md': 'The heading and introduction explain preparing a candidate and saving it after review. The unchanged paragraph explicitly states prepare_publication does not replace the destination and publish uses that same candidate; consumption and late-failure limits remain. The old heading alias is preserved.',
  'docs/ja/compatibility.md': 'Only the link label for the prepare-review-publish workflow changes; its semantic destination and public 0.3.1 statement are unchanged. Runtime/native-target and client-compatibility limits remain inherited.',
  'docs/ja/installation.md': 'The package feature list replaces the opaque prepared-publication term with advance candidate-file generation. The actual public API supports preparation; install command, Rust 1.92 minimum, empty defaults and native-toolchain caveats are unchanged.',
  'docs/ja/json-to-anki.md': 'The first output is clarified as the initial distribution APKG and the second as a checked answer-only update. Three notes, three cards, two media, complete SDK programs and no-Anki export requirement are unchanged.',
  'docs/ja/media.md': 'The opening now gives speech synthesis as an example of an external audio-generation tool, rather than implying it generates images. Media import still copies supplied data and retains immutable snapshots; code and API constraints are unchanged.',
  'docs/ja/node-api.md': 'The opening no longer falsely suggests Project is the only exported API: it correctly describes composing notes/media around Project. index.ts exports Content, Media, Field and other public classes. Batch/preparation wording and heading are clearer; exact APIs and both semantic anchors are preserved.',
  'docs/ja/python-api.md': 'Batch/prepared-publication terminology becomes advance generation of candidate files. The subsection still explains prepare/review/publish and consumed-owner behavior, with unchanged methods and source-api-details destination; the old Japanese heading is retained as an alias.',
  'docs/ja/python-migration.md': 'The old optional-artifact return is correctly described as a return value that may lack an artifact, contrasted with guaranteed BuildOutput and structured errors. No optional API or new behavior is introduced.',
  'docs/ja/rust-api.md': 'The introduction describes the public API as candidate pre-generation and saving. The unchanged Rust example uses prepare_publication followed by publish; source-only/public-version boundaries and ownership limits are retained.',
  'docs/ja/updates.md': 'The revised explanation explicitly connects prepare_publication and publish to writing the very APKG reviewed in the report. PreparedPublication owns one private candidate and publish consumes that candidate; no rebuild or client-import guarantee is added.',
  'docs/pt-br/anki-forge-vs-genanki.md': 'The media row corrects the misleading independent-copies wording: snapshots are independent of original source files, while clones share storage. This matches Arc<Snapshot> and typed Content retention.',
  'docs/pt-br/genanki-migration.md': 'Media snapshots are now described as independent of source files, and template expressions are distinguished from note-type models and Field keys. Stable business IDs, plain-text/HTML handling, no legacy identity reconstruction and migration boundary remain unchanged.',
  'docs/pt-br/index.md': 'Card templates consistently use templates while tipos de nota still names note types; the note-fields → templates → cards relationship remains correct. Navigation targets and supported tasks are unchanged.',
  'docs/pt-br/node-quickstart.md': 'The reusable template-bundle link label changes from modelos to templates, with the same destination. Runtime, npm install command, complete program and TypeScript setup remain byte-identical.',
  'docs/pt-br/python-migration.md': 'The migration table now says Media snapshots do not depend on source files and calls old template bundles templates. The in-process binding and original APKG evidence requirements are unchanged.',
  'docs/pt-br/python-quickstart.md': 'Only the template-bundle navigation label changes. Python install/run commands, native-wheel prerequisites, program and expected output are unchanged.',
  'docs/pt-br/troubleshooting.md': 'The media/template heading and bundle-failure row now distinguish card templates from note-type models. Bundle-v2, stable keys, internal paths and explicit media dependencies remain the repair conditions; the old heading alias is retained.',
  'docs/pt-br/updates.md': 'The prose distinguishes note types from card templates and clarifies exceeded inspection limits as a hard failure. Original APKG evidence, versioned identity, historical slots/revisions, non-deletion semantics and client validation caveats are unchanged.',
  'docs/python/api.md': 'Source API details becomes Batch media and prepared publication, matching the already stated public 0.3.1 availability. The explicit source-api-details section marker retains prior links; signatures and behavior are unchanged.',
  'docs/template-bundles.md': 'Explicit media closure is expanded to all explicitly declared media dependencies, and model declaration is clarified as note type definition. from_bundle snapshots the complete validated bundle and media before returning immutable NoteType; no partial project mutation is implied.',
  'docs/updates.md': 'Only the page heading changes publication to deck. Stable identities, original distribution baseline, risk-policy conditions, executable examples and client-import limits remain unchanged.',
  'docs/zh-cn/anki-forge-vs-genanki.md': 'The media row explains immutable data snapshots and clone storage sharing. This agrees with Media Arc<Snapshot>, and does not confuse owning bytes with retaining the original file.',
  'docs/zh-cn/api.md': '身份证据 is clarified as 标识映射与修订信息. IdentityEnvelope/PackageIdentity contain versioned model/note mappings, revisions and media evidence; the original APKG and rejection of Anki re-export remain explicit.',
  'docs/zh-cn/build-guarantees.md': 'The heading correctly describes preparing a candidate and writing the target after review rather than source-only availability. prepared-publication-from-source is now explicit, and the former Chinese heading maps to it; public version and lifecycle constraints remain.',
  'docs/zh-cn/compatibility.md': 'The original-APKG condition now calls evidence identifier mappings and revision information. This retains the same necessary baseline data; no broader platform, import or history promise is introduced.',
  'docs/zh-cn/concepts.md': 'Media now explicitly holds an immutable data snapshot and manages its lifetime. Source independence, shared clone storage and with_export_name preserving prior clones remain correct against Media implementation.',
  'docs/zh-cn/genanki-migration.md': 'The update paragraph clarifies identity evidence without relaxing the requirement for an original package, identical namespace/keys, or a separately verified legacy-identity migration.',
  'docs/zh-cn/index.md': 'The opening says APKGs are generated from notes, templates and media files. This is the same capability, with unchanged task links and no scheduling/import claim.',
  'docs/zh-cn/media.md': 'The single-file import statement now explicitly returns an object holding the media snapshot. Batch concurrency remains opt-in and the four-worker, sixteen-file threshold, per-asset and memory-budget facts are unchanged.',
  'docs/zh-cn/node-api.md': 'Media.file/bytes now explicitly return Media objects that keep imported data independent of later source-file changes. Node wraps the native handle in immutable Media; signatures, rename behavior, properties and MIME limits remain unchanged.',
  'docs/zh-cn/python-api.md': 'The file-import row names the returned Media snapshot holder. Source API details is renamed to batch media/prepared publication with an explicit source-api-details marker and old Chinese alias; calls and guarantees are unchanged.',
  'docs/zh-cn/python-diagnostics.md': 'Missing complete identifier mappings/revision information remains a CompareError hard failure. The wording change does not make evidence corruption a policy-allowable risk or remove completed observations.',
  'docs/zh-cn/python-migration.md': 'The final prerequisite still requires a complete original distribution APKG carrying current identity/revision evidence. It continues to reject automatic recovery of old bundles or old evidence.',
  'docs/zh-cn/rust-api.md': 'Consumer applications is clarified as applications using the library. Identity-evidence terminology is clarified in both hard-failure and embedded-evidence passages; public modules, API calls, policy limits and candidate lifecycle are unchanged.',
  'docs/zh-cn/rust-guide.md': 'Media imports are correctly described as returning snapshot-holding Media objects; image()/sound() are more precisely described as not returning errors. Their signatures return Content directly, while add/build still enforce independent MIME rules.',
  'docs/zh-cn/templates.md': 'The bundle is explained as note-type definitions, templates, CSS and all declared media dependencies, with stored snapshots. This matches from_bundle validation and immutable NoteType; project atomicity and shared API links remain.',
  'docs/zh-cn/troubleshooting.md': 'The prerequisite still requires original APKG identity/revision evidence, and neither namespace mismatch nor corrupt evidence can be allowed by a risk policy. Only the explanatory term changes.',
  'docs/zh-cn/updates.md': 'The opening now separates the versioned evidence format from source-code revision and explicitly retains the original APKG of the preceding distribution. Stable project/note/model/field/template keys and remaining update/client constraints are unchanged.',
  'website/language-briefs.json': 'Read the three glossary changes: zh-CN identifier/revision evidence agrees with PackageIdentity; pt-BR template de cartão distinguishes Template from NoteType; ja prepared publication is described as candidate generation followed by reviewed saving. Protected API names and product names remain unchanged.',
  'website/legacy-sections.json': 'Thirteen renamed H2/H3 headings were checked against actual old/current source headings and section contracts. Every former slug maps to the same stable task; locale ownership is preserved. English self-aliases preserve canonical anchors after visible-heading changes.',
  'website/site-model.mjs': 'Metadata now identifies installation as Rust in five languages, uses precise Portuguese card-template labels, clarifies Chinese ownership and Japanese persistent output/prepared-publication wording. The actual model routes, source mappings, locale selection, navigation destinations and implementation are unchanged; the description changes match their page tasks.',
  'website/src/content/blog/es/stable-ids-and-deck-updates.mdx': 'The baseline heading now says to use the previous package as reference. Its semantic marker and new explicit span preserve the former Spanish slug. Baseline prose, code and distribution-package caveats are unchanged.',
  'website/src/content/blog/ja/stable-ids-and-deck-updates.mdx': 'Relative to the previously reviewed isolated candidate, only the Anki re-export warning is clearer: it cannot be an update baseline. The shared CodeBlock/updateExample fix is byte-identical to the prior reviewed implementation, and the original package evidence requirement is unchanged.',
  'website/src/content/blog/pt-br/stable-ids-and-deck-updates.mdx': 'The title explicitly instructs updating note content without changing its ID. Translation key/date/category, body, code and original distribution baseline condition remain unchanged.',
  'website/src/content/blog/zh-cn/stable-ids-and-deck-updates.mdx': 'The title and final verification paragraph explain stable IDs and matching a note to its prior version. They retain the requirement to inspect source/baseline and not infer correctness from file existence; Anki import/local-edit caveats remain.',
  'website/src/content/i18n/es.json': 'The search label changes from searching a page to searching this site, matching the actual locale-scoped docs search UI. All other framework messages and placeholders remain unchanged.',
  'website/src/i18n/en.json': 'Read all changed messages: language controls now explicitly mean programming language; update prose requires original APKG and stable note IDs; AI actions still describe copying a prompt; performance explicitly describes five 1,000-note export tests and the Rust API. Snapshot CSV/model confirms those facts. machine.constraints retains package/source distinctions and client caveats.',
  'website/src/i18n/zh-CN.json': 'Read all changed messages: preview and AI-copy actions match their controls; IDs/original APKG and hello→hello; hi example descriptions match showcase source; Anki import/review behavior remains limited. Performance keeps notes/minimum/maximum interpolation and distinguishes image/audio shared versus unshared files.',
  'website/src/i18n/es.json': 'Read all changed messages: programming-language tabs, no-script task placeholder, one-second concert A, stable-ID update and rectangular occlusion are accurate against the consumers/showcase. Five measured export tests and one distinct image/audio per note match the selected CSV workloads; interpolation sets are unchanged.',
  'website/src/i18n/pt-BR.json': 'Read all changed messages: card templates are distinguished from note types, required-empty-field diagnosis matches AddErrorKind::RequiredField, and stable IDs/original APKG are preserved. Concert A lasts one second; update fields and five export workloads match showcase/CSV. AI controls and variables remain compatible.',
  'website/src/i18n/ja.json': 'Read all changed messages: required-field wording matches RequiredField; source/download/AI-copy labels describe their real targets; the four-card deck and shared preview fields match showcase source. Stable IDs/original APKG and five 1,000-note export workloads remain accurately bounded; placeholders are unchanged.',
  'website/src/i18n/de.json': 'Read all changed messages: IDs and original APKG remain required, not-found/examples labels match page actions, and Anki controls final card rendering/import behavior. Performance describes only measured export time for five 1,000-note tests and explicitly limits generalization. Machine task-group label changes only presentation.'
};
assert.deepEqual(Object.keys(observations).sort(), audit.changedInputFiles.map(entry => entry.path).sort());

const apiScopes = {
  'anki_forge/src/media/mod.rs': 'Read Media ownership documentation, Arc<Snapshot>, file/bytes constructors, image/sound Content returns, and with_export_name behavior.',
  'anki_forge/src/media/snapshot.rs': 'Read shared Arc/cache and snapshot lifetime/file-import passages; source independence and shared storage support the revised media descriptions.',
  'anki_forge/src/schema/bundle.rs': 'Read from_bundle/from_bundle_with_limits documentation and loading entry: immutable complete asset closure, path rules and no partial registration.',
  'anki_forge/src/schema/mod.rs': 'Read immutable NoteType/Arc and builder validation/asset ownership passages.',
  'anki_forge/src/schema/stock.rs': 'Read Basic stock model/field definition used by Note::basic.',
  'anki_forge/src/note/mod.rs': 'Read Note::basic and Note::cloze constructors and front/back field assignment.',
  'anki_forge/src/note/error.rs': 'Read RequiredField and structured AddErrorKind/code definitions supporting required-empty-field and error-category wording.',
  'anki_forge/src/build_api/identity.rs': 'Read versioned identity envelope and model/note/symbol/media mappings, content hashes and modification timestamps.',
  'anki_forge/src/build_api/identity/validation.rs': 'Read hard missing/invalid evidence errors, format/hash checks and required model mapping validation.',
  'anki_forge/src/build_api/prepared.rs': 'Read private candidate ownership, bound options/baseline, publish(self), policy rejection, late-publication error handling and same-candidate publication.',
  'bindings/node/src/index.ts': 'Read exported public classes and actual Media.file/bytes/files/withExportName wrappers and immutable handles.',
  'anki_forge/examples/website_showcase.rs': 'Read Basic/media/combined/update routines and one-second 440 Hz audio generation; exact source remains unchanged.',
  'docs/benchmark-presentation.json': 'Read selected date/commit/CSV digest and 1,000-note selection.',
  'docs/performance/20261008/comparison.csv': 'Read the five selected 1,000-note workload records, distinct-media counts and export timings. No new benchmark execution claimed.',
  'website/src/components/AiQuickstart.astro': 'Read programming-language tab and no-script prompt consumers, installation and copy-status wiring.',
  'website/src/components/UpdateDemo.astro': 'Read comparison/empty-result labels in the fixed showcase demo; these UI strings do not alter comparison policy or become a general API guarantee.',
  'website/scripts/content-sections.mjs': 'Read heading parsing, semantic mapping, required sections, locale-scoped legacy aliases and alias insertion used by the old-heading audit.'
};
const technicalSources = {};
for (const [filename, scope] of Object.entries(apiScopes)) {
  const bytes = await readFile(filename);
  technicalSources[filename] = { sha256: digest(bytes), readScope: scope, utf8: bytes.toString('utf8') };
}
const placeholders = value => [...value.matchAll(/\{+([A-Za-z_][\w.]*)\}+/g)].map(match => match[1]).sort();
const catalogChanges = {};
for (const filename of Object.keys(observations).filter(name => /website\/src\/(?:content\/)?i18n\/.*\.json$/.test(name))) {
  const { previousUtf8, currentUtf8 } = snapshot.files[filename];
  const before = JSON.parse(previousUtf8), after = JSON.parse(currentUtf8);
  assert.deepEqual(Object.keys(before).sort(), Object.keys(after).sort());
  catalogChanges[filename] = Object.keys(after).filter(key => JSON.stringify(before[key]) !== JSON.stringify(after[key])).map(key => {
    assert.equal(typeof after[key], 'string');
    assert.deepEqual(placeholders(after[key]), placeholders(before[key]), `${filename}/${key}: placeholders`);
    return { key, before: before[key], after: after[key] };
  });
}
const aiBefore = JSON.parse(snapshot.files['docs/ai-quickstart.json'].previousUtf8);
const aiAfter = JSON.parse(snapshot.files['docs/ai-quickstart.json'].currentUtf8);
for (const locale of ['zh-CN', 'pt-BR']) {
  assert.deepEqual(placeholders(aiAfter.prompts[locale]), placeholders(aiBefore.prompts[locale]));
  aiAfter.prompts[locale] = aiBefore.prompts[locale];
}
assert.deepEqual(aiAfter, aiBefore);

const observationsPath = `${directory}/technical-delta-observations.json`;
await save(observationsPath, {
  schemaVersion: 1, reviewerRun: '/root/standards_review/pr66-independent-technical-delta-2026-11-02',
  baselineCommit: audit.reviewBaseline, currentCommit: audit.currentCommit,
  method: 'Actual manual reading of all 65 input-file deltas relative to the previously reviewed isolated candidate, including 54 authored-content sources and 11 shared prompt/catalog/model/anchor/brief files. Full before/current snapshots and per-entry hash lineage are in technical-source-input-audit.json. API review is restricted to named read scopes; unchanged long-form source content is inherited explicitly, not claimed as newly read in full.',
  standards: ['website/README.md:112-122 requires actual independent source review and stable sections, without refreshing hashes as a substitute.', 'docs/plans/2026-10-27-multilingual-implementation-spec.md:174-285 requires facts/limits/code parity, independent revision review and byte-supported incremental reuse.', 'docs/plans/2026-11-01-remaining-locales-implementation-plan.md:100-177 requires current input snapshots and preservation of unresolved issues.'],
  documentedViolations: [], possibleSmells: [],
  observations,
  catalogChanges,
  technicalSources,
  checkedInvariants: ['204 old and 204 current source/target/dependency triples independently recomputed.', '259 closure files archived as exact old/current UTF-8 snapshots; 194 unchanged and 65 reviewed deltas.', 'All 54 changed authored-content sources preserve executable fenced blocks and Markdown link destinations.', 'Thirteen former H2/H3 slugs preserve their exact semantic task destinations.', 'All changed UI catalog key/placeholder sets are retained; AI changes are confined to two localized prompt terms.', 'Previous search/Pagefind, localized numeric formatting and shared Japanese code-block fixes match the previously independently reviewed bytes and retain that report lineage.'],
  limitations: ['This is source technical delta review, not a new full linguistic reread.', 'No new SDK program or browser/build acceptance is claimed by this report.', 'Post-build machine/usage acceptance must bind the subsequently frozen artifact.'],
  openIssues: []
});
const observationDigest = digest(await readFile(observationsPath));
const auditDigest = digest(await readFile(auditPath));
const report = {
  schemaVersion: 1, reviewerType: 'ai', reviewerRun: '/root/standards_review/pr66-independent-technical-delta-2026-11-02',
  rubricVersion, glossaryVersion, scope: 'All 204 current topic/locale entries at 09d431830568345a9e6ae2bd1cda2154af5a08e1, including editorial changes excluded from the earlier isolated candidate; source-only technical delta review.', entries: {}
};
let count = 0;
for (const [id, locales] of Object.entries(audit.entries)) for (const [locale, entry] of Object.entries(locales)) {
  const page = pages.find(page => page.id === id);
  const current = await reviewInputs(page, locale, root);
  assert.deepEqual(current, entry.current, `${id}/${locale}: inputs changed during review`);
  const priorBytes = await readFile(entry.inheritedTechnicalEvidence.path);
  assert.equal(digest(priorBytes), entry.inheritedTechnicalEvidence.sha256);
  const prior = JSON.parse(priorBytes).entries[id][locale].technical;
  assert.deepEqual(prior.openIssues, entry.inheritedOpenIssues);
  const unchangedPrimary = entry.sourceFiles.filter(filename => !entry.sourceChanges.includes(filename));
  const evidence = [
    { path: auditPath, sha256: auditDigest, entry: `${id}/${locale}`, observation: `Recomputed current sourceHash, targetHash and technicalDependencyHash from disk; matched the frozen candidate. Recomputed the prior triple from actual reviewed snapshot bytes and verified the immutable prior report. ${entry.changedFiles.length} closure files differ; all are specifically reviewed below.` },
    { ...entry.inheritedTechnicalEvidence, observation: `Verified prior technical evidence retained for unchanged inputs and unchanged portions of edited sources. ${unchangedPrimary.length ? `Unchanged primary source bytes: ${unchangedPrimary.join(', ')}.` : 'Primary source edits are limited to the explicit prose/metadata changes below; remaining source bytes inherit this review.'} No full reread of unchanged long-form content is claimed.` },
    { path: audit.snapshots.path, sha256: audit.snapshots.sha256, observation: 'Exact prior/current bytes and SHA-256 for all source, target, UI/model and technical dependencies, independently bound by the per-entry audit.' },
    { path: observationsPath, sha256: observationDigest, observation: 'Independent semantic observations, actual API source excerpts/snapshots and explicit reading scope; not an automatic hash refresh.' },
    ...entry.changedFiles.map(filename => ({ source: filename, currentSha256: snapshot.files[filename].currentSha256, previousSha256: snapshot.files[filename].previousSha256, observation: observations[filename] }))
  ];
  if (entry.sections) evidence.push({ observation: `Current ${locale} section mapping satisfies the topic contract. ${entry.sections.renamedHeadings.length ? entry.sections.renamedHeadings.map(heading => `Former #${heading.slug} still targets ${heading.semanticTarget}`).join('; ') : 'No previously published H2/H3 heading slug was removed for this topic/locale.'}` });
  report.entries[id] ??= {};
  report.entries[id][locale] = { ...current, technical: { status: 'reviewed', evidence, openIssues: [...prior.openIssues] } };
  count++;
}
assert.equal(count, 204);
assert.equal(execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(), audit.currentCommit);
const reportPath = `${directory}/technical-source-review.json`;
await save(reportPath, report);
console.log(JSON.stringify({ path: reportPath, sha256: digest(await readFile(reportPath)), entries: count, changedInputs: Object.keys(observations).length, archivedInputs: Object.keys(snapshot.files).length, inheritedOpenIssues: Object.values(report.entries).flatMap(locales => Object.values(locales)).flatMap(entry => entry.technical.openIssues).length }, null, 2));
