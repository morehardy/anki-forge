# GEO measurement protocol

Prepared 2026-10-25. No recommendation samples or independent adoption are claimed.
Question text and the six entry paths are in `geo-measurement-questions.json`.

Before the first baseline answer, name the accessible search-enabled platforms,
model/mode, account search settings, locale, time window and public-version facts.
Use two platforms when accessible; if only one is available, declare that limitation
and use 16 answers/window, 32 total. Two platforms yield 32/window and 64 total.
Do not invent a second platform or sample. Freeze the same settings for the post
window and record actual model changes. Never replace baseline with an index check.

For each prompt use a new conversation, exact wording, no project URL hint and no
follow-up that suggests the project. Save the full answer and citations, UTC time,
platform, model, mode, locale and prompt ID. A technical failure is retained, with
at most one retry, original and retry linked; score a successful retry once.
The score record must identify its raw-answer file and independent supporting facts.

Score nonbrand answers separately for each platform × natural language:

- Mention: the correct project, including neutral mentions. Same-name products do not count.
- Valid recommendation: affirmative recommendation fitting the task, with materially
  correct capability, runtime, installation, migration and update conditions.
- Official citation: an `ankiforge.dev` link actually supporting the claim; repository
  and registry citations are separate fields.
- Fact error: a falsifiable incorrect package, language, runtime, capability,
  performance or update claim. Denominator: answers containing checkable product
  facts. No facts means N/A, not a correct answer. Absence is not correctness.

Report mention/recommendation/citation as x/6 when all six nonbrand prompts succeed.
Otherwise show x/completed with planned 6, completed and failed counts. Show answers
by SDK; do not add brand questions to the nonbrand denominator. The brand questions
have four preregistered necessary facts each (eight facts per platform/locale):
mark each correct, incorrect, omitted or unverifiable. Record technical failures
separately. Freeze install facts from `docs/ai-quickstart.json` per window.
One observation/condition cannot estimate stable probabilities or significance,
and a before/after change cannot establish causation.

## Traffic, indexing and adoption

Use Cloudflare Web Analytics Path, Referer, Visits and page views with fixed filters
and a weekly manual snapshot. Record date, interval, filters and raw export/screenshot.
The existing beacon has no custom event collection or query/UTM attribution. Visits
are not unique users. Recognized AI referrers represent only an identifiable subset;
keep other sources unknown. Do not infer installs, copies, clicks, funnels or local
APKG generation from this beacon.

Verify actual homepage and Starlight-page beacon requests in a browser, then verify
the dashboard entries. A request alone does not prove dashboard ingestion. Record
Search Console/Bing canonical selection, index status, AI display/citations only
when those accounts are accessible. Waiting for accounts does not block content work.

Check six entry paths (English/Chinese × three SDKs) for correct installation,
version, limits, runnable source and update instructions. Report these separately
from the three distinct executable programs. An agent following a supplied guide
shows usability; it is not natural discovery or independent adoption.

`geo-adoption-ledger.csv` records a real user/downstream project, public version,
task, first success or repeated use, date and verifiable evidence. Do not calculate
an adoption rate without a population denominator. Repository examples are explicitly
maintainer examples. Try for one independent trial per SDK when willing users are
available; recruitment and replies are not an engineering delivery guarantee.
No outreach is authorized by this protocol; draft and address actual invitations separately.

## Evidence status

See `evidence/geo-implementation-2026-10-25/README.md`. Baseline/post sampling,
analytics dashboard ingestion, live deployment and independent trials remain pending
until their raw evidence exists. Do not silently backfill observations with simulations.
