# Maintaining user documentation

User guides are imported into the static website from repository Markdown.
The topic model in `website/site-model.mjs` defines routes, titles,
descriptions and each page's editable source. Generated pages are ignored by Git.
Edit the source document, not the generated copy.

## Write a task guide

Give the goal, requirements and working directory first. Include a complete
runnable example or explicitly identify the excerpt's complete program and its
inputs. Explain the expected file, card/note counts and how to inspect the result.
Then cover the relevant errors and link to a next task and API reference.

Keep field keys, display names, note IDs and card counts distinct. Describe
uniform text escaping and explicit Content::html accurately. Record each language's
own version and supported behavior; core support does not imply SDK parity.

## Keep code synchronized

Checked source blocks use this form around a fenced code block:

```text
<!-- source: repository/relative/file.rs#region -->
... fenced example ...
<!-- /source -->
```

Regions are delimited by `// docs:<region>:start` and `// docs:<region>:end`.
Existing website snippets use `// website:<region>:start/end` and the reference
suffix `#website:<region>`. Omit a suffix to include a complete file.

Run `npm run check:docs` from `website` to compare embedded code to its source,
check repository link targets, and check versioned public install commands. Use
`npm run sync:docs` after intentional example edits, then review the diff.
It updates marked blocks only; it does not prove the program works.

The website build runs the Rust showcase and documentation workflow. CI also
runs every complete imported Rust main plus the Basic/custom/media/workflow
examples in isolated directories, observes their APKG contents, and checks Python
and Node installed examples. New guides must
extend the relevant execution check when they add another complete program.

## Structure and compatibility

Retain old routes and significant anchors when splitting a page. Keep the
source-to-route map and sidebar aligned. Add the source path to workflow triggers
when expanding imported content. Shared user guidance lives under `docs/`;
contributor instructions and implementation history stay outside the user sidebar.

State whether a page follows a source checkout or a released version. Public installation
versions are maintained separately from source package metadata.
An installed-consumer test or successful wheel build does not mean publication.

## AI quickstart and Skill maintenance

`docs/ai-quickstart.json` defines the public package version, language/runtime
labels, exact installation commands, public documentation URLs and English/Chinese
brief templates. The version remains independent of source package manifests.
`website/src/lib/ai-quickstart.mjs` renders README briefs, server previews and
clipboard content with the same plain-text substitution rules.

From `website`, run `npm run sync:ai` after changing the shared data; review the
marked blocks in both READMEs. It also regenerates ignored
`src/generated/ai-quickstart.json`. `npm run check:ai` checks README drift, public
installation guides, prompt requirements, Skill metadata and complete examples.
The site preparation checks this contract and regenerates site data without
rewriting tracked READMEs. CI runs the content check when Skill files change.

Keep language examples in `skills/anki-forge/references` identical to the marked
public quickstarts. Update references and Skill version together with a public
release. Run `node skills/anki-forge/scripts/verify_quickstart.mjs` from the root
for the standalone contributor check. Links and text checks do not replace
actual SDK consumers or APKG inspection. Tool loading locations and official
sources are recorded in [AI assistant setup](ai-assistants.md).

When validating generated AI work, use isolated application directories for
Basic, media, custom models and an answer-only update from an archived original
APKG. Record actual source, commands, results and inspected contents; copying a
brief successfully does not establish that the AI's program works.

## Verification

From `website`, run `npm test`, `npm run check`, `npm run build`, and
`npm run check:site`. The build checks source snippets before importing them.
Verify changed installation paths in a separate application/environment as well.

Package generation and inspected counts do not prove client playback, review
history or every upgrade import. Record actual Anki versions and scenarios for
manual/client verification. Keep generated previews clearly distinguished from
Anki's own renderer.

## Performance presentation

`docs/benchmark-presentation.json` selects the dataset, measurement commit,
evidence commit and displayed note count. Both the website and
`scripts/generate_readme_assets.py` read this selection and the existing CSV.
`npm test` checks both README numeric alt text, headlines, method tables and
all four chart variants. `check:site` checks the actual rendered homepage values
and bar scales. Do not rerun the archived benchmark during a site build.

The CSV snapshot lives in `docs/performance/20261008/`; its checksum and original
path are recorded in `docs/benchmark-presentation.json`. `dataFile` is the local
input; `dataset` remains the historical evidence path for public links. The
archived reports and experiment tools are available at the pinned evidence
commit. They are not required by website builds. Verify public reachability
before publishing; never reconstruct measurements or re-run them during a build.

## Full-site translations and public recipe

`website/site-model.mjs` registers every normal topic in English and Chinese,
including homepage, examples, blog, guides and API references. Native Starlight
locales share one documentation layout and search experience. Chinese source
Markdown lives in `docs/zh-cn/`; generated content is never an editable source.
All internal content links must resolve to the current language. Shared downloads,
registries and external source links keep their common targets.

Add both authoritative sources and metadata when registering a topic. Add paired
blog posts with the same stable `translationKey`, date and category. Preserve the
same task sections, tables, technical limitations, code and next actions; do not
substitute an English fallback. After reviewing both texts, update their hashes
and reviewed status in `website/content-reviews.json`. Changes to either source
invalidate that review until a maintainer checks the pair again. Preserve old
anchors in `website/legacy-sections.json`; preparation supplies stable section
aliases for language switching. UI messages and dynamic/accessibility feedback
belong in the paired catalogs under `website/src/i18n/`.

`check:content` independently discovers authored sources, rejects missing or
unregistered translations, validates catalogs and checks review records.
`check:site` discovers the real built page set, navigation, actions, downloads,
metadata and links. `check:gates` and the content contract tests deliberately
remove translations, actions, anchors and messages to prove that these checks
fail. Browser tests use real HTTP output, Pagefind and browser clipboard APIs.
CI repeats build, artifact and browser checks at the root and a project prefix.
See [website maintenance](../website/README.md) for commands and adding content.

The paired source blocks share programs and fixtures in `docs/recipes/json-to-anki/`.
`npm run check:public-recipe` installs exact registry packages into temporary apps,
compiles TypeScript, independently reads ZIP/SQLite/media, compares identities and
answers, checks policy and verifies that repeat runs preserve v1. It needs network,
Rust, Node and Python with venv support. Linux x64 is the CI release gate; source
example checks remain separate. Evidence is saved under `target/geo-public-consumers/`.
The optional `json_recipe_oracle` binary in `scripts/roundtrip_oracle` observes real
Anki backend imports and scheduling in isolated collections; GUI and playback
checks still need separate evidence.

`SITE_DEPLOYMENT=production` requires the formal origin, root base and matching
CNAME. Previews can use other configurations. After deployment run `npm run check:live`;
its production origin is deliberately independent of build configuration, and
it requires the verified `site-manifest.json` artifact. It compares production
against that artifact and source commit, rather than accepting a previous release
with working English pages. Unknown URLs must return genuine 404 responses.
GitHub Pages has one static error document: JavaScript selects the path language,
while the no-script fallback presents both languages with working recovery links.
Measurement and independent-adoption records live under `docs/plans/`, outside
product copy. Never label prepared protocols as completed observations.
