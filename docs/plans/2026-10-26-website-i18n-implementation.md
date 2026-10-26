# Website language parity implementation evidence

Issue: [#62](https://github.com/morehardy/anki-forge/issues/62). Date: 2026-10-26.

## Baseline observed before implementation

- Working branch: `main`, source HEAD `b0cc49a5d9e621d197658cdcc756404cccac5b8b`.
- Latest successful Website workflow observed: [40264015907](https://github.com/morehardy/anki-forge/actions/runs/40264015907), source `a2afe14d7c035e561d10c52781c20c9972c2ec2d`.
- Its `github-pages` artifact was `12625109637`, 1,306,097 bytes, unexpired when checked.
- Pages deployment `7323403694` reported success at `https://ankiforge.dev/`, with [deployment job 121383142666](https://github.com/morehardy/anki-forge/actions/runs/40264015907/job/121383142666).
- The production sitemap returned HTTP 200; `/zh-cn/` returned HTTP 404. This established a difference between the current source and the served release. It did not establish why the intervening source had not been published.

## Implementation and verification seams

The shared model and native Starlight migration cover 32 real paired topics:
27 documentation topics, the homepage, examples, the blog list and two articles.
The former six Chinese guides were compared against their complete English
sources; 21 Chinese guides and both articles were added. Shared templates and
catalogs carry the interactions, accessibility text and metadata in both languages.
Translation review hashes cover all 29 long-form pairs. They are a change-detection
gate after review, not a substitute for editorial review.

The acceptance seam is generated output served over HTTP plus the public content
and artifact-check commands. Tests do not render mocked component trees. Executable
Rust/Node/Python examples, media and benchmark inputs retain their existing sources.
No public SDK API, note identity or update policy changed.

| Contract | Evidence |
| --- | --- |
| T1, T6 | Independent authored-source and built-route discovery; reviewed translation hashes; section/table/code comparison; native Starlight sidebars, source edit links, TOCs and pagination |
| T2, T3 | Built navigation/action order, all install commands, both languages' benchmark numbers; real mobile navigation, legacy anchors and query-preserving language round trips |
| T4, T5 | Browser keyboard tabs, flips, playback/pause, update/reset and six APKG downloads; actual clipboard reads, denied browser permissions, selection fallback and recovery |
| T7 | Actual Pagefind UI in each language; accessible result sections, empty results, blocked-index failures, shortcut/close/focus behavior |
| T8, T11 | Paired blog metadata and RSS; localized rendered social images; canonical/alternate/OG/beacon checks; HTTP 404 and bilingual no-script recovery |
| T9, T10 | Light/dark/system continuity; 320/390/768/1440 px layouts, brand and tool visibility, 200% text zoom, reduced motion and no-script content |
| T12 | Separate prefix and root builds; output manifest records source commit, input fingerprint, observed routes/capabilities and resource hashes; deployment compares the exact verified manifest and runs live browser checks |
| T13 | CLI fixtures remove a translation, add an unregistered source, change a reviewed source, use English fallback and break message keys/interpolation; artifact mutations remove an update action, break an anchor and cross languages |

Local checks are recorded separately from production publication. Commands are
in `website/README.md`; CI runs the same build, artifact and browser acceptance
for both configurations. Browser screenshots are generated in
`target/website-i18n-62/{prefix,root}/` and uploaded by the Website workflow.

The focused regressions caught and corrected compact-header brand hiding,
pre-initialization theme input, native clipboard coverage and search focus timing.
`check:examples` executed 14 complete Rust programs and independently inspected
23 APKG outputs. The complete website unit suite passed 22 tests. Existing CI
jobs additionally validate native Python/Node consumers and the public recipe.

## Deployment boundary

A local build does not prove that production is current. The release workflow
uploads `website-verified-manifest` alongside `github-pages`; the post-deploy check
requires that manifest and the workflow's commit, checks every normal route and
shared resource, and compares the live sitemap set. The manifest and workflow run
are the durable release evidence; an earlier successful deployment is insufficient.

GitHub Pages serves one static `404.html`. With JavaScript it chooses the requested
path language; without JavaScript it presents recovery actions in both languages.
This is an explicit host limitation, not a claim of server-side localized 404s.
No hosting migration or edge routing change is included. The browser suite verifies
both responses through real unknown URLs. Analytics beacon count is checked;
delivery to the analytics dashboard is not claimed.
