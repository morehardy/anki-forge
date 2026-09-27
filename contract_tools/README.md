# Repository contract tools

The `internal-tools` feature exposes the selected `ankiforge::tools` operations used by this crate. It does not expose implementation modules, a second Project, Deck, or the retired build API. Ordinary Rust consumers use the root Project/Note/NoteType/Media API.

## Native project commands

```sh
cargo run -p contract_tools -- product-build --project lesson.json --apkg-out lesson-v1.apkg
cargo run -p contract_tools -- product-compare --project lesson.json --baseline lesson-v1.apkg
cargo run -p contract_tools -- product-build --project lesson.json --apkg-out lesson-v2.apkg \
  --update-from lesson-v1.apkg --allow RISK.NOTE_REMOVED
```

`product-build` invokes native `Project::build`. Updates require the previous original APKG with complete embedded identity evidence. Missing evidence and namespace mismatches are hard errors. Create requests reject explicit update policy.

`product-compare` invokes `Project::compare` and publishes no APKG. A completed comparison exits successfully even when its policy blocks publication. It returns an `ankiforge-comparison-v1` snapshot; incomplete comparison returns `ankiforge-comparison-error-v1` with the actual error code, message and partial observation report.

Both accept `--base-dir`, `--fail-on info|low|medium|high|critical`, repeated `--allow RISK.CODE`, `--output contract-json|human`, and `--report-json`. Default update policy blocks High and Critical. Unknown and hard-error allowances are rejected. Defaults use the finite budgets in native `InspectLimits` and `MediaLimits`; these CLI commands currently do not override them.

Build JSON is the native `ankiforge-build-v1` BuildSnapshot. Exit codes are 0 for success, 2 for blocked build policy, 3 for invalid build configuration/content or incomplete comparison, and 4 for build I/O/resource/internal/publication errors. Input decoding, CLI parsing and invalid textual policy fail before the operation and print to stderr.

`--report-json` atomically replaces a separate snapshot file after the operation returns. If that write fails, stdout retains the actual operation snapshot (including build success and its artifact), stderr describes the separate report failure, and the command exits 4. The report path may not replace the input, baseline or APKG destination, including filesystem aliases. No multi-file transaction is claimed.

Removed native-product flags: `--manifest`, `--product-input`, `--compare-to`, `--identity-lockfile`, `--write-identity-lockfile`, and `--update-safety`. Native product operations use the embedded library contracts. Legacy ProductDocument JSON is rejected rather than converted with its old identity or HTML semantics.

## Project input: ankiforge-project-v1

This is a repository tool input recipe, loaded by `tools::load_project`. It is not serialization of Project internals. Unknown fields are rejected. Stable keys are required; no names are silently converted to keys.

```json
{
  "format_version": "ankiforge-project-v1",
  "namespace": "biology",
  "name": "Biology",
  "default_deck": "Biology::Cells",
  "assets": [
    {"key": "cell", "source": {"kind": "file", "path": "cell.png"}},
    {"key": "audio", "source": {"kind": "file", "path": "cell.wav"}}
  ],
  "models": [{
    "kind": "custom",
    "key": "vocabulary",
    "name": "词汇",
    "fields": [
      {"key": "front", "name": "题目", "required": true, "sort": true},
      {"key": "back", "name": "答案"}
    ],
    "templates": [{"key": "recognition", "front": "{{front}}", "back": "{{FrontSide}}<hr>{{back}}"}]
  }],
  "notes": [
    {"key": "definition", "content": {"kind": "basic", "front": "What is a cell?", "back": "A unit of life"}},
    {"key": "term", "content": {"kind": "custom", "model": "vocabulary", "fields": {
      "front": {"kind": "sequence", "items": ["Identify: ", {"kind": "image", "asset": "cell"}, {"kind": "sound", "asset": "audio"}]},
      "back": {"kind": "html", "value": "<strong>Cell</strong>"}
    }}},
    {"key": "cloze", "content": {"kind": "cloze", "text": "{{c1::Cells}} are units of life"}},
    {"key": "diagram", "content": {"kind": "image_occlusion", "image": "cell", "mode": "hide_all_guess_one", "masks": [
      {"key": "nucleus", "x": 10, "y": 10, "width": 20, "height": 20}
    ], "fields": {"header": "Identify the part"}}}
  ]
}
```

- `namespace` and `notes` are required. `name`, `default_deck`, `assets`, and `models` are optional. Each note requires `key` and `content`; `deck` and `tags` may be set beside them.
- Content is either a string (Text) or a tagged object: `text/value`, `html/value`, `image/asset`, `sound/asset`, or `sequence/items`. Basic front/back and ordinary strings escape literal HTML. Cloze syntax is preserved inside Text. HTML requires an explicit object.
- Basic content requires front/back. Cloze requires text and optionally back_extra. Custom requires model and a fields map keyed by stable field keys. ImageOcclusion requires an image asset key and pixel rectangles with stable mask keys. Its optional mode is hide_all_guess_one (default) or hide_one_guess_one; optional fields may set header/back_extra/comments. Native validation rejects overriding generated image/occlusion fields.
- File asset paths resolve against `--base-dir` or the JSON file's directory. A source may alternatively be `{"kind":"bytes","data":[0,1,2],"mime":"application/octet-stream"}`. Each asset may set `export_as`. All declared assets are included, including assets used in raw HTML/CSS/scripts. Import creates owned snapshots; deleting source files after successful loading does not affect a build.
- Custom models declare key, fields and templates. Optional name/css/cloze_field and an assets list are supported. Field options are name/required/sort. Template options are name/browser_front/browser_back/target_deck/generation_rule. Generation rules are `{"kind":"anki_default"}`, `{"kind":"all","fields":["key"]}`, or `{"kind":"any","fields":["key"]}`. All template references use stable keys.
- A model may instead be `{"kind":"bundle","path":"templates/cells"}`. It loads native template-bundle-v2 through `NoteType::from_bundle`, retaining the model's declared key and complete asset closure. Notes reference that key.
- Native model, media, note, mask and project validation applies. Duplicate asset/model keys, unknown references and legacy formats fail. No ProductDocument, lowering plan or lockfile is returned.

## Low-level conformance operations

`normalize`, `build`, `inspect`, and `diff` remain protocol fixture operations. Their IR and report DTOs are explicitly listed under tools; they are not the native product authoring interface. Existing protocol schemas and fixture oracles continue to verify their output.

| tools operation | Actual repository caller | Executable coverage |
| --- | --- | --- |
| load_project | product_build_cmd; product_roundtrip_oracle_prepare | product_cli_tests: native build, custom/media/IO, rejected legacy input |
| normalize; normalize_with_options | fixtures | fixture_gate_tests; compat_oracle_tests |
| build_contract | fixtures; compat_oracle | fixture_gate_tests; compat_oracle_tests |
| inspect_apkg; inspect_staging; diff_reports | fixtures; compat_oracle | fixture_gate_tests; compat_oracle_tests; package_tests |
| resolve_stock_notetype; extract_media_references | compat_oracle | compat_oracle_tests |
| policy_ref; build_context_ref | summary command's exact runtime references | cli_tests |
| load_runtime; normalize_from_path; build_from_path | normalize_cmd; build_cmd; conformance_surface | cli_tests; conformance_surface unit tests |
| discover_workspace_runtime | minimal_flow; conformance_surface | conformance_surface unit tests; minimal_flow execution |
| inspect_apkg_path; inspect_staging_path; diff_from_paths | inspect_cmd; diff_cmd; conformance_surface | cli_tests |
| canonical_json | normalize_cmd; build_cmd; inspect_cmd; diff_cmd; fixtures | cli_tests; fixture_gate_tests |
| authoring_contract_version; writer_contract_version | summary command's protocol inventory | workspace_smoke_tests; cli_tests |
| contract_template_bundle_paths | package and baseline asset hashing | package_tests: historical declarations, declared paths, nested paths, symlinks, escapes; fixture_gate_tests: missing dependencies |

DTO exports are limited to the concrete signatures and nested fields these operations exercise: authoring/normalized IR, normalization requests/results/options, writer policy/context/build results, inspection/diff reports, runtime resolution and structured inspection failure. No wildcard implementation module exports are used.
