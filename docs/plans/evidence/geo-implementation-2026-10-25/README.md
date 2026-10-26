# GEO implementation evidence

This record distinguishes repository implementation from deployment and external
observations. The existing 1,893 benchmark deletions and prior audit files were
present before implementation and are outside the GEO commits.

## Implemented

- Checksum-verified historical CSV, separate local/historical paths and manual archived reproduction.
- Fourteen paired English/Chinese core pages, six explicit Chinese guide routes,
  no Starlight fallback, self canonicals, reciprocal alternates and topic switches.
- Three public-SDK JSON/media/update programs sharing identical fixtures and source blocks.
- Exact registry installation checks, independent ZIP/SQLite/media observations,
  answer/identity/policy checks and refusal to overwrite original releases.
- Production parameter guard, public-consumer deployment gate and independent live smoke script.
- Fixed bilingual questions, scoring protocol, adoption ledger and feedback form.

## Validation

Executed validation:

- Full website suite: 18 tests passed.
- `check:ai`, standalone Skill check and `check:docs`: passed (34 source documents,
  27 synchronized excerpts).
- `check:examples`: 14 complete source executions and 23 independently inspected APKGs.
- Astro typecheck: zero errors/warnings/hints.
- Builds and `check:site`: passed for the repository prefix and formal root domain;
  40 pages with exactly seven Chinese routes, 14 paired core pages and valid local links.
- All three exact public 0.3.1 packages: passed on macOS arm64 and Linux x64 GNU/glibc.
  See `public-consumers.json` for registry provenance, versions and artifact hashes.
- Anki 25.09.2 backend: all three macOS-produced and all three Linux-produced release chains imported successfully
  using IfNewer and Always, without model merge; three notes/cards, exact media,
  answer replacement and preserved IDs/scheduled review values. No timestamp manipulation.
- Chrome at widths 390/1280: Chinese homepage, recipe and comparison had no horizontal
  overflow; correct lang/title, enabled copy controls and topic-preserving switches.
  Screenshots and machine observations are retained under `target/geo-browser/`.
- Browser interactions: Chinese AI task input and SDK tabs, 320px homepage layout,
  mobile Starlight topic switch and no-JavaScript recipe content passed.

## Review

Standards: one P2 finding resolved. The Rust Skill's public-example check now
reads the independently selected canonical program; an alternate-file regression
failed before the fix and passes afterward. Final unresolved findings: zero.

Spec: two P2 findings resolved. Removed JSON parsing/TypeScript compilation claims
from Basic quickstarts, and verified the actual unscoped Node native package's
version, registry and OS/CPU instead of recording an empty native list. Final
unresolved code/spec defects: zero; the external observation gates below remain open.

Machine evidence is generated under `target/geo-public-consumers/` (host) and
`target/geo-public-linux/` (Linux x64). These are three SDK executions per environment,
not six independent English/Chinese program tests. English/Chinese source blocks are identical.

## Still requires external observations

- Freeze accessible platform modes/settings and collect original baseline/post answers.
  Sampling has not run; no recommendation gain is reported.
- Cloudflare dashboard ingestion and Search Console/Bing account observations.
- Deployment and post-deployment smoke check against the new content.
- Manual Anki GUI rendering, image visibility, audible playback and learner workflow.
- Independent trials and actual downstream adoption; no outreach has been sent.
- Registry metadata readback after a future normal package release. Source descriptions
  do not change already published npm/PyPI descriptions.

GitHub About was updated and read back through the connected GitHub service:
“Open-source Anki APKG generation library with a Rust core and Rust, Python and
Node.js interfaces. Media, custom templates and reviewed updates. MIT.” The homepage
is `https://ankiforge.dev/`.

A clean Chrome session fetched the existing live homepage and Rust quickstart:
one beacon each, script HTTP 200 and RUM HTTP 204 for each page. These requests
verify client reporting, not dashboard ingestion or the newly built deployment.
`live-beacons.json` records those observations. The fixed public CSV's Git blob ID
also matches the local snapshot (`0a8428f3150d2f84f10333a9b574f6163b042dfe`),
and the pinned report was read successfully through GitHub. Automated backend
observations remain distinct from human GUI or independent-user evidence.
