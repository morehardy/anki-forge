# Manual Anki Desktop APKG Scenario Design

Date: 2026-04-10

## Goal

Design a practical scenario set to validate real APKG generation and manual import usability in
Anki Desktop, without depending on upstream Anki source-level oracle comparison.

## Scope

- Primary target: Anki Desktop (latest stable expected by current codebase direction)
- Output focus: real `.apkg` artifacts that can be imported manually
- Verification style: manual checklist evidence (import, rendering, media, study flow)
- Out of scope: upstream oracle parity checks

## Scenario Strategy

Use a risk-layered set with 8 scenarios:

1. Smoke coverage
2. Media coverage
3. Combination/edge-like behavior coverage within current defaults

This balances coverage and execution cost for iterative manual validation.

## Scenario Set

- `S01_basic_text_minimal`
- `S02_cloze_minimal`
- `S03_io_minimal`
- `S04_basic_image`
- `S05_basic_audio`
- `S06_basic_video`
- `S07_cloze_mixed_media`
- `S08_io_plus_audio`

## Data Model and Build Constraints

- Inputs are `authoring-ir` JSON and run through `normalize -> build -> inspect`.
- Current default build context uses `media_resolution_mode: inline-only` and
  `unresolved_asset_behavior: fail`.
- Therefore each media reference in note fields must have a corresponding inline media entry in
  the source fixture.

## Repository Layout

- Scenario inputs:
  - `contracts/fixtures/phase3/manual-desktop-v1/<scenario>/input/authoring-ir.json`
  - `contracts/fixtures/phase3/manual-desktop-v1/<scenario>/assets/README.md`
- Generated artifacts:
  - `tmp/manual-desktop-v1/<scenario>/package.apkg`
  - `tmp/manual-desktop-v1/<scenario>/apkg.inspect.json`
- Manual validation records:
  - `docs/manual-validation/anki-desktop-v1/<scenario>.md`
  - `docs/manual-validation/anki-desktop-v1/TEMPLATE.md`

## Execution Flow

Provide a helper script to reduce command complexity:

- `./scripts/run_manual_desktop_scenarios.sh` (all scenarios)
- `./scripts/run_manual_desktop_scenarios.sh <scenario>` (single scenario)

Script behavior:

1. Normalize from scenario authoring input
2. Build APKG with default writer/build selectors
3. Inspect APKG and store inspect report

## Manual Acceptance Criteria

For each scenario:

- Import succeeds without blocking errors
- Notes/cards match expected count
- Front/back rendering is correct
- Media behavior matches scenario type (image/audio/video)
- Study/review flow is functional
- Evidence (paths/screenshots) is recorded

## Risks and Tradeoffs

- Tiny embedded media payloads are useful for plumbing checks but may be less representative than
  richer real study media.
- Video playback support can vary by local environment codecs even when APKG import succeeds.

## Future Extensions

- Add richer real-world media payload fixture variants
- Add stress scenarios (multiple notes, larger media payloads, mixed decks/tags)
- Add optional cross-platform sampling after desktop baseline stabilizes
