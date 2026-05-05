# Manual Anki Desktop APKG Scenarios (v1)

This directory contains manual validation scenarios for Anki Desktop importability.

## Scenario List

- `S01_basic_text_minimal`
- `S02_cloze_minimal`
- `S03_io_minimal`
- `S04_basic_image`
- `S05_basic_audio`
- `S06_basic_video`
- `S07_cloze_mixed_media`
- `S08_io_plus_audio`
- `S09_io_rect`

Each scenario includes:

- `input/authoring-ir.json`: source input for `normalize -> build -> inspect`
- `assets/`: source copies of media files mirrored under `input/assets/` for path-based media sources

## Generate APKG

From repo root:

```bash
./scripts/run_manual_desktop_scenarios.sh
```

Run one scenario only:

```bash
./scripts/run_manual_desktop_scenarios.sh S05_basic_audio
```

Outputs are written to:

- `tmp/manual-desktop-v1/<scenario>/package.apkg`
- `tmp/manual-desktop-v1/<scenario>/apkg.inspect.json`

## Notes

- Scenarios `S03` to `S09` use real media sources (PNG, WAV with non-zero frames, MP4/WebM).
- If you need to swap assets for local validation, replace files under each scenario `input/assets/` directory and regenerate with:

```bash
./scripts/run_manual_desktop_scenarios.sh <scenario>
```
