# Maintaining user documentation

User guides are imported into the static website from repository Markdown.
The import list in `website/scripts/content-links.mjs` defines routes, titles,
descriptions and each page's editable source. Generated pages are ignored by Git.
Edit the source document, not the generated copy.

## Write a task guide

Give the goal, requirements and working directory first. Include a complete
runnable example or explicitly identify the excerpt's complete program and its
inputs. Explain the expected file, card/note counts and how to inspect the result.
Then cover the relevant errors and link to a next task and API reference.

Keep field keys, display names, note IDs and card counts distinct. Describe
Project text escaping and Deck HTML semantics accurately. Record each language's
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
check repository link targets, and check declared version tables. Use
`npm run sync:docs` after intentional example edits, then review the diff.
It updates marked blocks only; it does not prove the program works.

The website build runs the Rust showcase and documentation workflow. CI also
runs the complete source-linked Basic/custom examples, a Python documentation
example job, and the Node minimal/installed example checks. New guides must
extend the relevant execution check when they add another complete program.

## Structure and compatibility

Retain old routes and significant anchors when splitting a page. Keep the
source-to-route map and sidebar aligned. Add the source path to workflow triggers
when expanding imported content. Shared user guidance lives under `docs/`;
contributor instructions and implementation history stay outside the user sidebar.

State whether a page follows a source checkout or a released version. Read source
versions from package metadata and maintain release/verification status separately.
An installed-consumer test or successful wheel build does not mean publication.

## Verification

From `website`, run `npm test`, `npm run check`, `npm run build`, and
`npm run check:site`. The build checks source snippets before importing them.
Verify changed installation paths in a separate application/environment as well.

Package generation and inspected counts do not prove client playback, review
history or every upgrade import. Record actual Anki versions and scenarios for
manual/client verification. Keep generated previews clearly distinguished from
Anki's own renderer.
