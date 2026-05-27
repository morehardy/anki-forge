# Phase 5 Python Adoption Design

- Date: 2026-05-26
- Status: Approved design draft
- Source: `docs/api-design.md` Phase 5
- Scope: Python Product API adoption, packaging, diagnostics, docs, and release readiness

This spec is authoritative for Phase 5 implementation where it is more specific than `docs/api-design.md`.

## Confirmed Decisions

1. Phase 5 uses a pure Python Product API wrapper with a bundled Rust CLI runtime.
2. The first release does not include CSV or pandas helpers.
3. Python users import `anki_forge`; the existing `anki_forge_python` package remains a lower-level runtime wrapper or compatibility layer.
4. Phase 5 does not introduce PyO3, maturin, native Python extension modules, or generated Rust source execution.
5. Python Product API must lower into the same Rust Product/IR/build/report pipeline as the Rust API.

## Goal

Phase 5 makes `anki-forge` adoptable by Python users, especially users who know genanki and want a safer build system. A user should be able to install a wheel, write a Python `Project`, build an `.apkg`, inspect structured diagnostics, and migrate basic/custom/media decks without installing Rust.

Phase 5 is not the first Python technology spike. The repository already has low-level Python contract wrappers, target API sketches, Rust Product APIs, `BuildReport`, and Phase 4 diff/risk behavior. This phase turns those pieces into a product-grade Python package.

## Non-Goals

- CSV or pandas helpers.
- PyO3 or native Python extension bindings.
- A Python API that mirrors Rust ownership or chaining patterns.
- Exposing Authoring IR, Normalized IR, or contract tooling as the first user-facing mental model.
- A genanki-compatible API clone. Migration is concept-oriented, not drop-in compatibility.
- Mixed field content builders that interleave text and media fragments in one call. Phase 5 supports whole-field text, HTML, sound, and image values; richer composition can come later.
- A dry-run build that validates without writing any artifact. Phase 5 build paths produce an `.apkg`; callers that want temporary validation can use a temporary output path.
- Built-in subprocess timeouts. Long-running builds are caller-managed in Phase 5.
- Thread-safe mutation. Treat `Project`, `NoteType`, `Note`, and `MediaRegistry` as not thread-safe.
- Customizing reserved stock Basic/Cloze note types. Users who need custom CSS, extra templates, or altered stock fields should define a custom note type with a non-reserved id.
- Removing or reordering notes after `Project.add_note(...)`. Phase 5 stores notes in insertion order only.

## Recommended Approach

Use a Python Product API plus a bundled Rust CLI.

Python owns the user-facing object model and serialization. Rust owns lowering, normalization, writer behavior, inspection, diff, risk, and report generation.

```text
Python Product objects
  -> ProductDocument JSON
  -> bundled contract_tools product-build
  -> Rust ProductDocument -> Project::from_product_document(...)
  -> existing lowering / normalize / writer / inspect / diff / risk
  -> BuildReport JSON
  -> Python BuildReport / DiagnosticsError
```

This approach avoids PyO3 release complexity while preserving one semantic pipeline. It also aligns with the existing `contract_tools product-build` command and `anki_forge::runtime::build_product_document_with_writer_stack(...)`.

## Alternatives Considered

### Current Contract Tools Only

Python could generate Authoring IR and call the existing `normalize`, `build`, `inspect`, and `diff` wrapper APIs.

This has the smallest code delta, but it exposes low-level contract concepts as the main Python experience. It also risks diverging from the Rust Product API, especially for media, identity, diagnostics, and `BuildReport`.

### Temporary Rust Runner

Python could generate temporary Rust code or a config consumed by a Rust runner.

This reuses Rust builders quickly, but it is brittle for packaging, debugging, error locations, and cross-platform wheels. It is not suitable as the adoption path.

## Package Layout

Add a new public package under `bindings/python/src/anki_forge`:

```text
anki_forge/
  __init__.py
  project.py        # Project and write_apkg
  notetype.py       # NoteType, Field, Template, GenerationRule
  note.py           # Note and Content helpers
  media.py          # MediaRegistry and MediaRef
  report.py         # BuildReport projections
  diagnostics.py    # Diagnostic and DiagnosticsError
  runtime.py        # bundled/workspace runtime discovery and product-build invocation
  product_json.py   # ProductDocument transport serialization
```

Keep `anki_forge_python` as the low-level wrapper for existing `normalize`, `build`, `inspect`, and `diff` workflows. The main README and Python guide should use `anki_forge`, not `anki_forge_python`.

Implement the new public `anki_forge` package as its own package. It must not import from the dev-only `anki_forge_python` package at runtime.

The top-level `anki_forge.__init__` should export the public user-facing API: `Project`, `Note`, `NoteType`, `Field`, `Template`, `GenerationRule`, `MediaRef`, `MediaRegistry`, `BuildReport`, `Diagnostic`, `DiagnosticsError`, `ValidationError`, `RuntimeNotFoundError`, `RuntimeInvocationError`, and `ProtocolError`.

Phase 5A should implement `anki_forge` independently rather than extracting shared helpers from `anki_forge_python`. This instruction wins over possible helper sharing during Phase 5A. The old wrapper stays untouched except for tests needed to keep it green. A later cleanup can consolidate subprocess/runtime/report parsing helpers into a private `anki_forge._runtime` or `anki_forge._contract` module after the public API has landed; do not create a public dependency between the two top-level packages.

Because `anki_forge` and `anki_forge_python` will temporarily have independent subprocess/report helpers, Phase 5A implementation comments should point both packages at the intended future consolidation module. This reduces drift risk around argument encoding, stdout parsing, and report validation without expanding the first implementation slice.

Use `anki_forge._runtime` as the named future consolidation target in those comments so the debt is grep-able and consistent.

## Public Python API Shape

The main entry points are:

- `Project(name, stable_id=None, default_deck=None)`
- `Project.add_notetype(note_type)` and `Project.add_note(note)`
- `NoteType.custom(id, name=None, css=None)`
- `NoteType.field(field)`, `NoteType.template(template)`, and `NoteType.css(value)` mutate the note type and return `self`
- `Field(name, key=None, identity=False, sort=False, required=False)`
- `Template(name, key=None, front=..., back=..., generate_when=None)`
- `GenerationRule.anki_default()`, `.all(fields)`, `.any(fields)`, `.cloze(field)`
- `Note.basic(front, back, stable_id=None, deck_name=None)`
- `Note.cloze(text, back_extra="", stable_id=None, deck_name=None)`
- `Note(note_type_id, stable_id=None, deck_name=None).text(field_key, value).html(field_key, value).sound(field_key, media_ref).image(field_key, media_ref).tag(tag).tags(tags).deck(name)`
- `Project.media.add_file(path, export_as=None)`
- `Project.media.add_bytes(source_label=..., data=..., export_as=...)`
- `Project.write_apkg(path, compare_to=None, fail_on=None, report_json=None)`

`text()` is safe text by default. `html()` is explicit raw HTML. Python should prefer mutable object style with optional fluent helpers where they read naturally.

Concurrency: `Project`, `NoteType`, `Note`, and `MediaRegistry` are mutable builder objects and are not thread-safe. Users building in parallel should create independent projects or synchronize their own access.

`NoteType` is constructed through `NoteType.custom(...)`, a `@classmethod` alternate constructor on `NoteType`; direct `NoteType(...)` construction is not public in Phase 5. Custom note type `id` is immutable after construction. `name` defaults to `id` when omitted. `css` accepts `str | None`; `None` serializes as JSON `null`. `NoteType.css(value)` replaces the CSS value and accepts the same types; `.css(None)` clears previously set CSS. Stock note types are auto-declared only and do not expose CSS customization in Phase 5.

The `.css(None)` clearing behavior must be explicit in the method docstring. Phase 5 does not add a separate `clear_css()` method.

`Field` and `Template` are immutable value objects after construction. `NoteType.field(field)` and `NoteType.template(template)` copy the field/template values into the note type; mutating a local object after adding it is not part of the public API and must not change serialization.

`Project.write_apkg(...)` returns `BuildReport` whenever `product-build` returns parseable, contract-valid report JSON, including invalid, blocked, or error reports. It raises `RuntimeNotFoundError`, `RuntimeInvocationError`, or `ProtocolError` only when no valid report can be recovered. Users call `report.ensure_success()` when they want invalid, blocked, error, missing-artifact, or error-diagnostic reports to become `DiagnosticsError`.

Note field methods are:

- `note.text(field_key, value) -> self`
- `note.html(field_key, value) -> self`
- `note.sound(field_key, media_ref) -> self`
- `note.image(field_key, media_ref) -> self`
- `note.tag(tag) -> self` for one tag string
- `note.tags(tags) -> self` for an iterable of tag strings, appending rather than replacing

If multiple content methods target the same `field_key`, the last call wins. This is a replacement operation, not mixed content composition.

Tags are exact strings. Python rejects empty tags and tags containing whitespace or ASCII control characters (`U+0000` through `U+001F` or `U+007F`). `tag()` and `tags()` preserve first-seen order but de-duplicate exact repeated tags during serialization.

`Note(note_type_id, stable_id=None, deck_name=None)` accepts an optional per-note deck override. It validates `note_type_id` immediately in `Note.__init__`: after stripping ASCII whitespace it must be non-empty and contain no ASCII control characters (`U+0000` through `U+001F` or `U+007F`). It validates `stable_id` immediately when provided with the same stripped-non-empty and ASCII-control-character rules. It also validates `deck_name` immediately when provided: it must be a non-empty string. `Note.basic(front, back, stable_id=None, deck_name=None)` and `Note.cloze(text, back_extra="", stable_id=None, deck_name=None)` are `@classmethod` alternate constructors on `Note` and perform the same immediate validation through the normal constructor. They return normal `Note` instances, not subclasses or separate factory result types. `Note.basic(...)` maps to the stock Basic note type id `"basic"` and treats both string arguments as safe text. `Note.cloze(...)` maps to stock Cloze note type id `"cloze"`; its `text` argument is explicit HTML so cloze markers are preserved, while `back_extra` is safe text.

`Note.basic(front, back, stable_id=None, deck_name=None)` is equivalent to `cls("basic", stable_id=stable_id, deck_name=deck_name).text("front", front).text("back", back)`. `Note.cloze(text, back_extra="", stable_id=None, deck_name=None)` is equivalent to `cls("cloze", stable_id=stable_id, deck_name=deck_name).html("text", text).text("back_extra", back_extra)`. The constructor path through `cls(...)` is required so `note_type_id`, `stable_id`, and deck validation behave identically to manually constructed stock notes.

The `Note.basic(...)` and `Note.cloze(...)` classmethods should be discoverable through standard Python introspection and IDE autocomplete on `Note`. The `Note.cloze(...)` docstring and migration guide must make the `text` HTML / `back_extra` safe-text split prominent because the API signature alone does not reveal the asymmetry. Include a docstring example along the lines of `Note.cloze("{{c1::<b>term</b>}}", back_extra="<i>hint</i>")`, where the cloze body is HTML and `back_extra` renders escaped safe text.

Python serialization includes one stock Basic declaration if any note has note type id `"basic"` and one stock Cloze declaration if any note has note type id `"cloze"`. Stock declarations are computed by scanning the current ordered note list at each serialization/build call; `Project` does not eagerly cache stock declarations in `add_note()`. `Note.basic()` and `Note.cloze()` are the normal way to create those notes, but the trigger is the resolved note type id. Users cannot register custom note types with reserved ids `"basic"` or `"cloze"`; Python raises `ValidationError`.

When both stock and custom note types are serialized, the note type order is: auto Basic if referenced, auto Cloze if referenced, then custom note types in `Project.add_notetype(...)` insertion order. This order is deterministic and snapshot-tested.

Existing builder-backed Rust `Project` paths auto-add stock ProductDocument note types in `Project::to_product_document()` when direct Project notes reference stock ids. `product-v2` input is different: it is already a ProductDocument transport, so Rust lowers exactly the declared `note_types`. The bypass condition is `product_document_version == "product-v2"` with explicit stock declarations present. Rust must not add a second Basic/Cloze note type for those ids. If a hand-written `product-v2` note references `basic` or `cloze` without the matching stock declaration, Rust reports a missing note type diagnostic rather than implicitly creating one.

Rust must enforce the same reserved-id rule for all `product-v2` inputs, not only Python-generated JSON. `{"kind": "custom", "id": "basic"}` and `{"kind": "custom", "id": "cloze"}` are invalid and produce a structured reserved-note-type-id diagnostic. `{"kind": "stock"}` is valid only for ids `basic` and `cloze`.

Reserved-id validation runs in `Project.add_notetype(...)` and again at serialization for safety with identical rules. Auto-declared stock note types are internal declarations and do not conflict with the reserved-id rule.

Stock metadata:

- Basic fields: `front` is required and sort; `back` is not required. Rust's stock identity recipe is `basic.core.v1` from `contracts/semantics/note-stable-id.md`: infer `afid:v1:<blake3(canonical_payload)>` from the NFC- and newline-normalized `front` value only, with canonical payload fields `algo_version`, `recipe_id`, `notetype_family`, `notetype_key`, and `components.selected_fields`.
- Basic template: `card_1` with front `{{Front}}`, back `{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}`, and Anki default generation.
- Cloze fields: `text` is required and sort; `back_extra` is not required. Rust's stock identity recipe is `cloze.core.v2` from `contracts/semantics/note-stable-id.md`: infer `afid:v1:<blake3(canonical_payload)>` from the NFC- and newline-normalized `text` value after cloze parsing, with canonical payload fields `algo_version`, `recipe_id`, `notetype_family`, `notetype_key`, and `components` containing the cloze `text_skeleton` plus parsed deletions.
- Cloze template: `cloze` with front `{{cloze:Text}}`, back `{{cloze:Text}}<br>\n{{Back Extra}}`, and cloze generation on `text`.

Stock auto-declarations are internal declarations, not `NoteType.custom(...)` objects. The generic custom-note identity derivation rule does not apply to them. Python serializes stock note-type identity as omitted, and if the transport requires field-level `identity` booleans it serializes them as `false` for stock fields. Rust then applies the stock recipes above during lowering.

The stock identity recipes are not new Phase 5 inventions: they exist in the Rust deck identity layer (`anki_forge/src/deck/identity.rs`) and are covered by existing deck identity contract tests. Phase 5A's work is to route `product-v2` stock notes through those recipes. The expected path is lowering-only: `product-v2` stock notes should lower into the field payload shape that the existing identity code already consumes. Adding a private lowering adapter is in scope; changing stock identity recipe semantics or exported identity call signatures is a stop-and-replan signal for the 5A.0 gate.

The 5A.0 stock identity gate passes only if product-v2 Basic and Cloze fixtures derive the same stable IDs and recipe ids as equivalent builder-backed Product/Deck paths using existing stock recipe semantics and without changing exported identity signatures, contract semantics, normalized IR, or writer-core APIs. The minimum acceptable change is a private adapter in product lowering or deck identity internals that preserves the same canonical payload. If parity requires broader API or semantics changes, stop, update this spec, and split a stock-identity-adapter implementation plan before Python serialization claims stock compatibility.

Field content methods are content-kind helpers, not stock-field type enforcement. For example, `Note(note_type_id="basic").html("front", "<b>bold</b>")` is a supported long-term workflow and serializes HTML content for the Basic front field. `Note.basic(...)` remains the recommended safe-text convenience constructor. Requiredness and identity checks operate on the resolved field content value after Rust lowering; Python should not reject HTML, sound, or image content solely because the stock display field is commonly text-oriented.

Python owns the stock field key list for early helper validation: Basic allows only `front` and `back`; Cloze allows only `text` and `back_extra`. `Note("basic").text("nonexistent", "value")` raises `ValidationError` immediately. Custom note field keys are not validated by `Note.text(...)` because the note has no parent context there.

`Project.name` must be a non-empty string and is a valid fallback deck name. `Project.default_deck` is optional and may be `None`; when it is set, it must be a non-empty string. `Project.stable_id` is optional and may be `None`; when provided, it uses the same immediate validation as note `stable_id`: after stripping ASCII whitespace it must be non-empty and contain no ASCII control characters (`U+0000` through `U+001F` or `U+007F`). Explicit note deck names and resolved deck names must also be non-empty strings. Explicit note `stable_id` values are validated at `Note` construction as described above. Python should validate these simple checks early; deeper Anki deck-name and identity compatibility remains Rust lowering's authoritative diagnostic.

Deck fallback is late-bound at serialization time. A note without an explicit deck uses the current `Project.default_deck` or `Project.name` at the moment `write_apkg()` serializes the project, not the values that existed when the note was added. The `Project.default_deck` docs should call out this timing.

`Project.stable_id` maps to top-level `document_id` in `product-v2`. The Python API uses `stable_id` because users think of it as the project's stable identity; the wire transport keeps the existing `document_id` name. If `Project.stable_id` is provided, it wins for `document_id` even when it differs from `Project.name`; `Project.name` still remains the fallback deck name. If omitted, Python uses `Project.name` as `document_id`, matching current Rust `Project` behavior. For example, `Project(name="Japanese::Core", stable_id="jp-core")` serializes `document_id: "jp-core"` and uses `"Japanese::Core"` as the deck fallback.

`Project.media` is always initialized as an empty `MediaRegistry` during `Project.__init__`. Accessing it never returns `None`.

`MediaRef` is a public value object. `MediaRegistry` is also importable for type annotations, but `Project.media` is the supported way to obtain a registry in normal use.

`MediaRegistry()` direct construction is not part of the public Phase 5 API. Tests or advanced users that need an isolated registry should create a `Project` and use `project.media`.

`Project` stores notes internally as an ordered list. Phase 5 does not expose a public `Project.notes` collection API; users add notes through `Project.add_note(...)`. The `project.notes[...]` spelling in `source_path` is a diagnostic address, not a public mutable collection.

`Project.write_apkg(...)` does not seal or freeze the project. Users may add note types, notes, or media after one build and call `write_apkg(...)` again; each call serializes the current in-memory state in insertion order.

`Project.add_notetype(note_type)` stores the mutable `NoteType` object for serialization; it does not deep-copy and freeze the whole note type. It validates the note type when added, and serialization validates it again because callers may still hold and mutate the registered object.

The `NoteType` docstring must call out that post-registration mutation is allowed but can produce serialization/build-time errors that point to the current project state rather than to the earlier mutation site. Phase 5 does not add a freeze/seal method.

`Project.add_note(note)` fast-fails with `ValidationError` if `note.note_type_id` is neither a reserved stock id nor a custom note type already registered on the project. Users register custom note types before adding notes. For custom notes, `add_note()` also validates the note's current field keys against the registered `NoteType` for better common-case feedback. It does not freeze the note; custom field-key checks run again during serialization and Rust lowering so later note mutation is still caught. Mutating a registered `NoteType` after notes have been added is allowed but caller-beware: missing newly required fields or changed generation rules surface as Python serialization errors where possible and Rust diagnostics otherwise.

`note.deck(name)` sets or replaces the per-note deck override and returns `self`. Passing `None` clears the override. `name` must be non-empty when set. The constructor `deck_name=...` and fluent `deck(...)` are equivalent.

The `note.deck(...)` docstring must explicitly state that `deck(None)` clears the note-level override.

When `Field.key` or `Template.key` is omitted, Python derives it immediately in `Field.__init__` or `Template.__init__` from the display name with the Rust-compatible slug rule: strip leading/trailing ASCII whitespace (`U+0009` through `U+000D` and `U+0020`), lowercase ASCII letters, preserve ASCII digits, replace each run of non-ASCII-alphanumeric characters with a single `_`, then trim leading/trailing `_`. Empty derived keys fail immediately with `ValidationError`. If two fields or two templates on the same note type derive or declare the same key, Python raises `ValidationError` in `NoteType.field(...)` or `NoteType.template(...)` and includes both display names in the error message. Custom note type field display names must also be unique, because Anki template references use display names while transport metadata uses keys; Python raises `ValidationError` in `NoteType.field(...)`, repeated in `Project.add_notetype(...)` and serialization.

The empty-key error should explicitly tell users with non-ASCII field/template names to pass an explicit ASCII `key=...`.

## Transport Schema

Python serializes Product objects to `ProductDocument` JSON and calls `contract_tools product-build`.

The current Rust `ProductDocument` transport is narrower than the target Python API. Phase 5 must extend that transport before relying on it for the Python API. Required additions include:

- Field metadata needed by Python `Field`: identity, sort, required.
- Template generation rules using stable field/template keys.
- Typed content for notes: safe text, raw HTML, sound media, image media.
- Media asset declarations that support `add_file`, `add_bytes`, and `export_as`.
- Stable source paths that let Rust diagnostics point back to Python project objects.

Python must not silently degrade typed content into plain strings when that would change escaping, media handling, identity behavior, or diagnostics.

Use a versioned `product_document_version` field on the top-level JSON object. The exact wire field name is `product_document_version`; Rust structs should use explicit serde rename rules if needed, for example `#[serde(rename = "product_document_version")]`, and shared `contracts/fixtures/product-v2/` fixtures are the arbiter for the exact snake_case names. Existing unversioned `ProductDocument` input remains valid legacy transport and is treated as `product-v1` by Rust. Python Phase 5 always emits explicit `product-v2`. The name `product-v2` is intentional even though `product-v1` was unversioned: it preserves the compatibility story that legacy product documents are the first transport generation and the Python transport is the second generation. `product-build` must reject unknown explicit product document versions with a structured diagnostic instead of attempting a best-effort parse. Future incompatible transport changes should bump this field to `product-v3`; `BuildReportJson.schema_version` remains a separate version axis for emitted build reports.

The transport shape should be explicit enough for Python and Rust snapshot tests to agree:

```json
{
  "product_document_version": "product-v2",
  "document_id": "jp-core",
  "default_deck_name": "Japanese::Core",
  "note_types": [
    {
      "kind": "custom",
      "id": "jp-vocab",
      "name": "Japanese Vocabulary",
      "fields": [
        {
          "name": "Expression",
          "key": "expr",
          "identity": true,
          "sort": true,
          "required": true,
          "source_path": "project.note_types[\"jp-vocab\"].fields[\"expr\"]"
        }
      ],
      "templates": [
        {
          "name": "Recognition",
          "key": "recognition",
          "front": "{{Expression}}",
          "back": "{{FrontSide}}<hr id=\"answer\">{{Meaning}}",
          "generation_rule": {"kind": "all", "fields": ["expr"]},
          "source_path": "project.note_types[\"jp-vocab\"].templates[\"recognition\"]"
        }
      ],
      "identity": {"kind": "fields", "fields": ["expr"]},
      "css": null
    }
  ],
  "notes": [
    {
      "kind": "custom",
      "note_type_id": "jp-vocab",
      "stable_id": "jp-vocab:taberu",
      "deck_name": "Japanese::Core",
      "fields": {
        "expr": {"kind": "text", "value": "食べる"},
        "meaning": {"kind": "html", "value": "<b>to eat</b>"},
        "audio": {"kind": "sound", "media_id": "media:000001"}
      },
      "tags": ["jlpt-n5"],
      "source_path": "project.notes[\"jp-vocab:taberu\"]"
    }
  ],
  "media": [
    {
      "id": "media:000001",
      "source": {"kind": "file", "path": "/absolute/path/to/media/taberu.mp3"},
      "export_as": "taberu.mp3",
      "source_path": "project.media[\"taberu.mp3\"]"
    }
  ]
}
```

The exact Rust structs can differ from this JSON example, but the wire semantics must not. Python golden tests and Rust deserialization/lowering tests should snapshot this transport before end-to-end wheel work begins.

Rust `product-v2` transport design for Phase 5:

```text
ProductDocumentV2 {
  product_document_version: "product-v2",
  document_id: String,
  default_deck_name: String?,
  note_types: Vec<ProductNoteTypeV2>,
  notes: Vec<ProductNoteV2>,
  media: Vec<ProductMediaV2>
}

ProductNoteTypeV2 = CustomNoteTypeV2 | StockNoteTypeV2
ProductFieldV2 { name, key, identity, sort, required, source_path }
ProductTemplateV2 { name, key, front, back, generation_rule, source_path }
IdentityRecipeV2 = { kind: "fields", fields: Vec<FieldKey> }
GenerationRuleV2 = anki_default | all(fields) | any(fields) | cloze(field)
ProductNoteV2 { kind, note_type_id, stable_id?, deck_name, fields: Map<FieldKey, FieldContentV2>, tags, source_path }
FieldContentV2 = text(value) | html(value) | sound(media_id) | image(media_id)
ProductMediaV2 { id, source: file(path) | inline_base64(source_label, data_base64), export_as, source_path }
```

For `inline_base64` media sources, `data_base64` uses standard RFC 4648 base64 with padding. The transport does not use URL-safe or unpadded base64.

`ProductNoteV2.kind` is `"custom"` for notes whose `note_type_id` refers to a custom note type and `"stock"` for notes whose `note_type_id` is `basic` or `cloze`. Stock notes use the same `fields` map shape as custom notes but with stock wire keys. Python serializes `Note.basic(...)` as `{"kind": "stock", "note_type_id": "basic", ...}` and `Note.cloze(...)` as `{"kind": "stock", "note_type_id": "cloze", ...}`. Rust rejects `{"kind": "stock"}` with any non-reserved `note_type_id` and rejects `{"kind": "custom"}` with reserved stock ids.

Use internally tagged serde enums with `kind` strings matching the JSON examples. Unknown `kind` values, unknown `product_document_version` values, and malformed field/media/generation references should become structured diagnostics in a contract-valid build report when the process reaches report generation. Transport deserialization should remain separate from Authoring IR and Normalized IR structs; lowering maps `ProductDocumentV2` into the existing Product/Rust pipeline and is the only place that expands stock defaults, resolves typed media content, applies identity recipes, and converts source paths into diagnostics.

The `kind` tag is intentionally reused in separate object scopes: `ProductNoteTypeV2` elements inside `note_types` and `ProductNoteV2` elements inside `notes` each use their own `#[serde(tag = "kind")]` enum. Rust must deserialize them as distinct enum types based on their containing array, not through a shared global enum, so `"custom"` and `"stock"` values are never interpreted without object context.

The Phase 5A Rust work is therefore bounded to:

- serde transport structs and version dispatch for `product-v1`/`product-v2`;
- lowering from `ProductDocumentV2` into the existing Product build path;
- diagnostics for invalid product-v2 references and unsupported values;
- shared JSON fixtures proving Rust and Python agree on the wire format.

If implementation needs broad changes outside these boundaries, the 5A.0 sizing gate must split that Rust transport work into a separate reviewed plan before Python API implementation continues.

Stock note type wire semantics:

| Stock note type | Note type id | Field keys | Template key | Template front | Template back |
| --- | --- | --- | --- | --- | --- |
| Basic | `basic` | `front`, `back` | `card_1` | `{{Front}}` | `{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}` |
| Cloze | `cloze` | `text`, `back_extra` | `cloze` | `{{cloze:Text}}` | `{{cloze:Text}}<br>\n{{Back Extra}}` |

Rust lowering maps those stable wire keys to Anki stock display field names such as `Front`, `Back`, `Text`, and `Back Extra`. Python should not invent alternate stock keys.

Auto-declared stock Basic serializes exactly as:

```json
{
  "kind": "stock",
  "id": "basic",
  "name": "Basic",
  "fields": [
    {
      "name": "Front",
      "key": "front",
      "identity": false,
      "sort": true,
      "required": true,
      "source_path": "project.note_types[\"basic\"].fields[\"front\"]"
    },
    {
      "name": "Back",
      "key": "back",
      "identity": false,
      "sort": false,
      "required": false,
      "source_path": "project.note_types[\"basic\"].fields[\"back\"]"
    }
  ],
  "templates": [
    {
      "name": "Card 1",
      "key": "card_1",
      "front": "{{Front}}",
      "back": "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}",
      "generation_rule": {"kind": "anki_default"},
      "source_path": "project.note_types[\"basic\"].templates[\"card_1\"]"
    }
  ],
  "css": null,
  "source_path": "project.note_types[\"basic\"]"
}
```

Auto-declared stock Cloze serializes exactly as:

```json
{
  "kind": "stock",
  "id": "cloze",
  "name": "Cloze",
  "fields": [
    {
      "name": "Text",
      "key": "text",
      "identity": false,
      "sort": true,
      "required": true,
      "source_path": "project.note_types[\"cloze\"].fields[\"text\"]"
    },
    {
      "name": "Back Extra",
      "key": "back_extra",
      "identity": false,
      "sort": false,
      "required": false,
      "source_path": "project.note_types[\"cloze\"].fields[\"back_extra\"]"
    }
  ],
  "templates": [
    {
      "name": "Cloze",
      "key": "cloze",
      "front": "{{cloze:Text}}",
      "back": "{{cloze:Text}}<br>\n{{Back Extra}}",
      "generation_rule": {"kind": "cloze", "field": "text"},
      "source_path": "project.note_types[\"cloze\"].templates[\"cloze\"]"
    }
  ],
  "css": null,
  "source_path": "project.note_types[\"cloze\"]"
}
```

Stock declarations intentionally omit the note-type-level `identity` property rather than serializing `null`. Their field-level `identity` values are always `false`; Rust applies stock identity recipes from `contracts/semantics/note-stable-id.md` during lowering.

Template `front` and `back` strings are Anki template source and refer to Anki display field names, such as `{{Expression}}`, not transport wire keys. Stable wire keys are used for transport fields, generation rules, identity recipes, and source paths. Rust lowering must use `field.name` as the Anki display/field name; `field.key` is transport metadata only. Rust lowering is responsible for preserving the display-name template semantics while using keys for update-safe metadata.

Python does not parse or validate field references inside template `front`/`back` strings in Phase 5. Broken display-name references surface through Anki/writer behavior and diagnostics where available. Python validates only structured generation-rule field keys.

Field metadata semantics:

- For custom note types, `identity=True` marks the field as eligible for the note type's default identity recipe.
- `sort=True` marks the field as the preferred sort field metadata. At most one field per note type may have `sort=True`; Python raises `ValidationError` at `NoteType.field(...)` when a second sort field is added, and repeats the check at serialization.
- `required=True` means notes of this type should provide a non-empty value for the field; missing or empty values produce error diagnostics.
- `required=False` is the relaxed default: missing values are allowed without a missing-field diagnostic.
- For custom note types, users normally choose `required=True` for identity/card-critical fields and leave relaxed fields at the default.
- Field requiredness does not itself control card generation. Card generation is controlled by template `generation_rule`.

For required-field diagnostics, "missing" means the field key is absent from the note's `fields` map. "Empty" is content-kind specific: `text` and `html` are empty only when their value is the zero-length string; whitespace is considered provided. `sound` and `image` are empty when `media_id` is the zero-length string or does not resolve to a declared media entry. A required media field with a valid `media_id` but an unreadable source file is not a missing-field error; it is a media source diagnostic.

Python does not preflight required-field completeness for stock or custom notes in Phase 5. It validates field keys and identity availability, then lets Rust produce required-field diagnostics so missing/empty semantics stay centralized with typed content and media resolution.

`generation_rule` is serialized as one of:

- `null` or `{"kind": "anki_default"}` for Anki default card generation.
- `{"kind": "all", "fields": ["field_key"]}` for all required fields.
- `{"kind": "any", "fields": ["field_key"]}` for any required field.
- `{"kind": "cloze", "field": "field_key"}` for cloze field generation.

`GenerationRule.*(...)` constructors validate argument shape and string safety because they have no note-type context. `GenerationRule.all(fields)` and `GenerationRule.any(fields)` reject empty field lists with `ValidationError`; use `GenerationRule.anki_default()` for default generation. `GenerationRule.cloze(field)` rejects an empty field key. `NoteType.template(template)` validates that referenced field keys already exist on that note type; users should declare fields before templates. `Project.add_notetype(...)` and serialization repeat the check so later field/template mutation cannot bypass it. Rust lowering remains authoritative.

Rust must enforce the same rule for hand-written `product-v2`: empty `all.fields`, empty `any.fields`, and empty `cloze.field` are invalid generation rules and should produce structured diagnostics, not vacuous all/any card-generation behavior.

Cloze generation follows Anki's stock semantics: create one card for each distinct positive cloze ordinal present in the target field, such as `{{c1::...}}` and `{{c2::...}}`. Multiple deletions with the same ordinal generate one card for that ordinal. If a required cloze field is non-empty but contains no cloze markers, Rust should report the structured generated-card/no-cloze diagnostic used by the Product pipeline rather than Python guessing from raw text.

`GenerationRule.anki_default()` should serialize as `{"kind": "anki_default"}`. Python `Template(generate_when=None)` and `Template(generate_when=GenerationRule.anki_default())` are equivalent and serialize the same explicit `{"kind": "anki_default"}` value. Omitted generation rule is accepted only for compatibility with older hand-written documents and is normalized by Rust to the same semantics. Phase 5 intentionally does not preserve a distinction between "unspecified" and "explicit default"; any future incompatible default-generation change should use a new generation rule kind or a new `product_document_version`.

The generic `.text(...)` helper is safe text for every note type, including Cloze. It preserves braces and therefore cloze markers such as `{{c1::value}}`, but it escapes HTML-sensitive characters. `Note.cloze(...)` uses HTML semantics for convenience when users already have cloze HTML. Python should not special-case `.text("text", ...)` for Cloze because doing so would make the same method name unsafe only on one stock field.

Note-type `identity` defines how Rust derives a stable note identity when a note omits `stable_id`. The custom note type identity check is performed at serialization/build time against the complete project, not at `add_note()` time. Valid identity forms for Phase 5 are:

- `{"kind": "fields", "fields": ["field_key"]}` for custom note types.
- Omitted identity for stock Basic and Cloze note types, which use Rust stock recipes.
- Omitted identity for custom note types only when every note of that type has an explicit `stable_id`; Rust should emit the existing missing-identity warning.

Unknown identity kinds are invalid. If a custom note type omits identity and at least one note of that type omits `stable_id`, Python raises `ValidationError` during serialization before invoking Rust. Rust must still report a structured diagnostic for equivalent hand-written `product-v2` JSON because it cannot derive a stable note identity.

Python derives the custom note type identity recipe from all fields marked `identity=True`, in field declaration order. If no field is marked `identity=True`, the identity field list is omitted. This derivation rule applies only to custom note types. Stock Basic and Cloze always omit the note-type identity recipe in transport and rely on Rust stock recipes. An explicit identity setter is out of scope for Phase 5.

For custom note types, Rust lowering treats the note-type-level `identity` recipe as authoritative. Field-level `identity` booleans are metadata used by Python to derive that recipe and by fixtures/docs to preserve user intent; Rust should not derive or modify identity from field-level booleans when a hand-written `product-v2` document diverges. Python serialization keeps field-level booleans and the note-type recipe consistent by construction.

For custom note types, `product-v2` lowering reuses the existing Product identity recipe from `anki_forge/src/product/project.rs`: recipe id `custom.notetype.fields.v1`, `notetype_family: "custom"`, `notetype_key: note_type.id`, and components `selected_fields` in identity field order, each containing `key`, display `name`, and NFC/newline-normalized rendered `value`. Phase 5 does not introduce a new custom identity recipe id. Python/Rust fixture parity should prove that an equivalent builder-backed Product project and a `product-v2` document derive the same stable IDs for custom notes.

Rust owns canonical identity derivation. Before hashing or comparing identity payloads, Rust must normalize identity field text to NFC and normalize newlines according to `contracts/semantics/note-stable-id.md`. Python serializes user-provided text as-is and must not derive final stable IDs independently.

A note-level `stable_id` is an explicit override and wins for that note. Rust is the authoritative validator for resolved stable IDs, collisions, and identity diagnostics because it has the complete ProductDocument after defaults and lowering. Python preflights duplicate explicit `stable_id` values during serialization for faster feedback using one project-wide set across all notes and note types. It does not derive stock/custom identities, so collisions involving derived IDs, or one explicit ID colliding with a Rust-derived ID, are Rust diagnostics and must be covered by Rust/Python E2E failure tests.

Note `deck_name` is resolved during Python serialization: explicit note deck wins, then `Project.default_deck`, then `Project.name`. The serialized `product-v2` note always carries the resolved deck name.

`source_path` is a stable diagnostic address string, not a Python object pointer. Rust diagnostics should preserve it verbatim when reporting Product-level issues. Python exposes it as `Diagnostic.source` and does not need to map it back to a live object in Phase 5. The path grammar is:

```text
project.note_types["<note_type_id>"]
project.note_types["<note_type_id>"].fields["<field_key>"]
project.note_types["<note_type_id>"].templates["<template_key>"]
project.notes["<stable_id>"]
project.notes["<stable_id>"].fields["<field_key>"]
project.notes[<zero-based-index>]
project.notes[<zero-based-index>].fields["<field_key>"]
project.media["<export_as>"]
```

Keys inside string brackets use JSON string escaping and use stable wire keys, not Anki display names. Notes with `stable_id` use `project.notes["<stable_id>"]`; notes without `stable_id` use the zero-based absolute position in the full serialized notes list, including notes that do have `stable_id`. For `[A(stable_id="a"), B(no stable_id), C(stable_id="c"), D(no stable_id)]`, B is `project.notes[1]` and D is `project.notes[3]`. Index-based note paths are intentionally simple, keep Python-list bracket syntax, and are computed during serialization; Python must not invent synthetic UUID/hash paths or alternate `index=...` syntax for unstabilized notes. Index-based paths are only reliable for the exact project serialization that produced the report and are not stable across reordering or later mutation, so persisted/long-lived diagnostics require explicit note `stable_id`.

This source-path grammar is an intentional wire contract for Phase 5 diagnostics and should be fixture-snapshotted. It is not merely an internal formatting detail.

Phase 5 does not emit warning diagnostics solely because a note omits `stable_id`; that would be too noisy for quick-start decks. Documentation should recommend explicit `stable_id` values for long-lived or diagnostic-heavy projects.

`text()` safety means escaping exactly `&`, `<`, `>`, `"`, and `'` when rendering into HTML fields, while preserving all other user text, whitespace, newlines, backslashes, braces, LaTeX delimiters, and cloze marker characters as literal text. `html()` bypasses that escaping. `Note.cloze(...)` should continue to use explicit HTML semantics for cloze markers, matching the Rust README warning.

Typed media content lowers through the Rust Product pipeline: `sound` renders as Anki `[sound:<export_as>]`, and `image` renders as an `<img src="<export_as>" alt="">` HTML fragment.

## Runtime Behavior

`anki_forge.runtime` locates a runtime in this order:

1. Explicit runtime override for tests and advanced users.
2. Bundled wheel runtime: platform-specific `contract_tools` executable plus contract assets.
3. Workspace runtime: repository checkout with `contracts/manifest.yaml`, used by development and CI when no bundled runtime is present.

Bundled-before-workspace fallback is intentional for normal installed users: an installed wheel should not accidentally pick up a nearby checkout. Development and CI that need the current workspace binary must pass the explicit runtime override.

Workspace fallback locates the repository root by finding `contracts/manifest.yaml`, then searches for the executable at `target/release/contract_tools` first and `target/debug/contract_tools` second, using `contract_tools.exe` on Windows. If neither binary exists, `RuntimeNotFoundError` should say which paths were tried and suggest building the workspace binary or using the explicit runtime override.

`Project.write_apkg()` writes a temporary ProductDocument JSON file and invokes:

```bash
contract_tools product-build \
  --manifest <runtime>/contracts/manifest.yaml \
  --product-input <tmp/project.json> \
  --apkg-out <target.apkg> \
  --output contract-json
```

`compare_to`, `fail_on`, and `report_json` are passed through to `product-build`.

Accepted Python `fail_on` values are the existing risk policy levels: `"info"`, `"low"`, `"medium"`, `"high"`, and `"critical"`. The risk scale is `info < low < medium < high < critical`. The threshold is inclusive: `"medium"` means fail when the highest risk is medium, high, or critical. `None` means no risk policy gate. Invalid values fail before subprocess invocation with a Python validation error.

`compare_to` accepts a filesystem path to a previous `.apkg`. Python validates that the argument is path-like and passes an absolute path to `product-build`, but does not preflight readability or existence. Python rejects `compare_to` when its resolved absolute path equals the target `.apkg` path, because comparing against the same file that will be replaced creates baseline/overwrite ambiguity. Rust remains authoritative for other baseline diagnostics. A missing, unreadable, or corrupt baseline always produces an invalid report with comparison diagnostics, `comparison: "unavailable"`, and no diff result. `product-build` should exit non-zero with the invalid-status exit code after writing valid report JSON to stdout and `report_json` when requested. Python therefore returns `BuildReport`; callers get `DiagnosticsError` only when they call `ensure_success()`. `fail_on` level does not change this behavior: unavailable baseline failure is driven by invalid status, not by risk-threshold comparison, and it is invalid for every `fail_on` value and when `fail_on` is `None`. If Rust includes a baseline-unavailable risk summary, policy may also report a blocked threshold, but that is not required for failure. If `product-build` fails before producing report JSON, Python raises `RuntimeInvocationError` or `ProtocolError` according to the exception boundary below.

`compare_to` without `fail_on` still computes comparison, diff, and risk when the baseline can be inspected and attaches them to the report; it simply does not apply a risk threshold gate. With `compare_to` and `fail_on`, `product-build` must still write the full report JSON, including comparison, diff, and risk sections when available, before exiting non-zero for a policy failure. This is required Phase 5 behavior whether it already exists in the current binary or must be fixed in Rust; Phase 5A should add a CLI regression fixture for this stdout-before-exit behavior because Python depends on parsing that report.

`write_apkg` overwrites an existing target `.apkg` if Rust successfully writes the artifact.

`BuildReport.status` remains the coarse Phase 4 status enum: `success`, `blocked`, `invalid`, or `error`. Missing/unreadable `compare_to` baselines and empty projects both use `invalid`; callers distinguish them by diagnostic codes and sources rather than by adding new statuses in Phase 5.

Rust `product-build` owns `.apkg` atomicity. It should materialize the artifact to a temporary file in the target directory and replace the target only after success. This is separate from Python's temporary ProductDocument JSON directory. If the process is interrupted before the replace step, the previous target should remain intact where the platform supports atomic replacement.

Use platform replace semantics, not delete-then-rename. On POSIX, use an atomic rename/replace operation from a temp file in the target directory. On Windows, use `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING` and `MOVEFILE_WRITE_THROUGH`, or a well-reviewed Rust crate/helper that provides those semantics. If the platform replacement primitive fails, report an output-write diagnostic and leave any previous target untouched.

Rust should create the target parent directory before creating the temporary artifact file, matching current output-directory behavior. If parent directory creation, temp-file creation, artifact copy, flush, or final replace fails after build execution has reached report generation, `product-build` should return a contract-valid report with an error diagnostic such as `PROJECT.OUTPUT_DIR_FAILED` or `PROJECT.OUTPUT_WRITE_FAILED`, no artifact, and stdout JSON. Python then returns `BuildReport`; `ensure_success()` raises `DiagnosticsError`. If the failure happens before report generation can start, Python raises `RuntimeInvocationError`.

An empty project with no notes should produce an invalid report with a structured diagnostic and no artifact. If current Rust behavior silently writes an empty `.apkg`, Phase 5A must change `product-build` and add a regression test before Python `write_apkg()` is considered compatible.

`fail_on` requires `compare_to` in the Python API. If `fail_on` is set without `compare_to`, Python raises `ValidationError` before subprocess invocation because Phase 5 import risk is comparison-based.

If no runtime can be resolved, `write_apkg()` raises `RuntimeNotFoundError` with the attempted lookup locations and a short remediation message. It must not fall through to a cryptic subprocess `ENOENT`.

Temporary ProductDocument files should be created under `tempfile.TemporaryDirectory()` and cleaned up after the subprocess returns. Tests may opt into retaining the temporary JSON for snapshot/debug output. Local failures while creating the temporary directory or writing the ProductDocument JSON, such as disk full or permission errors, are wrapped in `RuntimeInvocationError` with the original exception attached as the cause.

`MediaRegistry.add_file(path, export_as=...)` resolves `path` to an absolute path at registration time using the current working directory. This avoids later `chdir` changes altering the build. `source_label` in diagnostics may preserve the user-provided display path, but the transport source path is absolute.

If `export_as` is omitted for `add_file`, Python uses `Path(path).name`. `add_bytes` requires explicit `export_as`.

`add_file(...)` does not preflight file existence, readability, or MIME/content type at registration time. Rust build remains the authoritative point where missing or unreadable media becomes structured diagnostics. The migration guide should call this out so users understand registration is not a filesystem validation step.

Duplicate path-backed media registration is keyed by resolved absolute path and `export_as`, not by file content. If the file changes on disk between registration calls or between registration and build, the same `MediaRef` still points at that path and the build uses the file content available at build time. The media documentation should call this out rather than hashing path-backed files at registration time.

`add_bytes(source_label=..., data=..., export_as=...)` requires `source_label` as a non-empty `str` without ASCII control characters (`U+0000` through `U+001F` or `U+007F`). It is a human-readable diagnostic label, not an identity key and not the packaged filename. The packaged filename is always `export_as`. Python rejects zero-length byte payloads with `ValidationError`; hand-written `product-v2` inline media with empty decoded bytes is a Rust media diagnostic.

`add_bytes(data=...)` accepts bytes-like data: `bytes`, `bytearray`, or `memoryview`. It rejects `str`. Python computes and stores a SHA-256 digest and byte length at registration time for duplicate checks, so repeated `add_bytes` calls with the same `export_as` compare `(length, sha256)` first and do not repeatedly scan all existing byte payloads. The transport serializes inline bytes as standard RFC 4648 base64 with padding, matching Python `base64.b64encode`. This makes ProductDocument JSON larger by roughly one third for inline media; fixtures should keep inline bytes small, and larger user assets should prefer `add_file(...)`.

`add_file` and `add_bytes` return a `MediaRef` object with at least `media_id` and `export_as` attributes. `note.sound(...)` and `note.image(...)` accept `MediaRef`, not a bare string.

`MediaRegistry` assigns deterministic project-local media ids in registration order: `media:000001`, `media:000002`, and so on. Returning an existing duplicate ref does not allocate a new id. Media ids are not derived from paths, bytes, or `export_as`; this keeps ids stable for a given registration order and avoids leaking local paths into note field content.

The six-digit padding is a minimum width, not a hard cap. After `media:999999`, ids continue as `media:1000000`, `media:1000001`, and so on.

The padded media id format is chosen for deterministic fixture readability and lexical ordering in generated JSON; callers should treat media ids as opaque project-local tokens.

`export_as` must be a helper-safe bare filename: non-empty, no `/` or `\` path separators, no `..` traversal segment, not exactly `.` or `..`, not absolute, no URL escapes, and no ASCII control characters (`U+0000` through `U+001F` or `U+007F`). Leading-dot filenames such as `.hidden.mp3` are allowed. Python validates these simple filename rules at registration time, and Rust remains authoritative during lowering. Duplicate `add_bytes(..., export_as=...)` calls with the same `export_as` and identical `(length, sha256)` return the same `MediaRef`. Duplicate `add_bytes` calls with the same `export_as` and different digest fail early with `ValidationError`. Identical bytes registered under different `export_as` values are distinct packaged media entries. Duplicate `add_file` calls with the same absolute path and `export_as` return the same ref. Duplicate file registrations with the same `export_as` but different absolute paths fail early with `ValidationError` because they would collide in the packaged `.apkg`. Cross-source duplicates between `add_file` and `add_bytes` with the same `export_as` fail early with `ValidationError`.

The same absolute file path registered with different `export_as` values creates distinct packaged media entries with distinct `media_id` values. Python does not deduplicate across different export names because the packaged filenames and Anki references differ.

Python serializes all registered media entries, not only referenced entries. Unreferenced media should produce the existing unused-media warning diagnostics from the Rust pipeline.

Path-backed media in `ProductMediaV2.source.path` is an OS-native absolute path for the machine running `product-build`. Python does not normalize separators for transport; Windows backslashes are encoded as normal JSON string escapes and POSIX paths remain POSIX paths. Path-backed ProductDocument JSON is not a cross-OS portable interchange format. Users who need portable self-contained JSON should use `add_bytes(...)`, accepting the inline-size trade-off.

Python must invoke `contract_tools` with `subprocess.run([...], shell=False, capture_output=True, text=True, encoding="utf-8")` or equivalent list-argument form with captured UTF-8 stdout/stderr. This is required for paths with spaces, non-ASCII paths, command-injection safety, and populated runtime error objects. Phase 5A should add a Windows CLI preflight using a non-ASCII `--product-input` path to verify the Rust `clap`/argv boundary handles wide-character paths correctly before wheel packaging relies on it.

With `--output contract-json`, `product-build` stdout must contain only UTF-8 JSON for the full `BuildReportJson`, plus optional trailing whitespace. Human logs and warnings must go to stderr. Python parses the entirety of stdout after trimming leading/trailing whitespace.

Phase 5A must include a Rust-side stdout-purity preflight before Python runtime integration: invoke `product-build --output contract-json` on a fixture, assert the flag value is accepted, and assert stdout parses as exactly one JSON report. If this fails because human logs are written to stdout, Python `Project.write_apkg()` integration tests cannot be marked passing until logs move to stderr. This is expected to be a CLI/logging configuration fix; if it becomes a cross-cutting logger refactor, split it into its own implementation plan while other transport/object-model work continues behind the mini-gate.

Required `product-build` CLI flags for Phase 5 are:

- `--manifest <path>`
- `--product-input <path>`
- `--apkg-out <path>`
- `--compare-to <path>` when `compare_to` is set
- `--fail-on <level>` when `fail_on` is set
- `--report-json <path>` when `report_json` is set
- `--output contract-json`

Current repository inventory: all required `product-build` flags already exist in `contract_tools/src/main.rs` and `contract_tools/src/product_build_cmd.rs` as of this spec. Phase 5A should not need new CLI flags for Python invocation. Expected Rust CLI work is semantic: product-v2 deserialization/version dispatch, stdout-purity tests, empty-project behavior, and any report/protocol adjustments uncovered by fixtures.

Python validation policy:

| Validation | Python behavior | Rust behavior |
| --- | --- | --- |
| `Project.name`, `Project.stable_id`, deck name emptiness | Required fast-fail `ValidationError` | Authoritative for deeper Anki deck compatibility |
| `Note.note_type_id` blank/control characters | Required fast-fail in `Note.__init__` | Authoritative for lowering/source paths |
| Custom note type id safety | Required fast-fail `ValidationError` for blank/ASCII-control-character ids and reserved ids | Authoritative for lowering/source paths |
| `fail_on` accepted values and `compare_to` requirement | Required fast-fail `ValidationError` | Authoritative risk policy behavior |
| `export_as` simple filename safety | Required fast-fail `ValidationError` | Authoritative media diagnostics |
| Duplicate explicit `stable_id` | Required serialization-time preflight using one project-wide set of explicit stable ids across all notes and note types | Authoritative after identity derivation |
| Generation rule references existing fields | Required check at `NoteType.template(...)`, repeated at `Project.add_notetype(...)` and serialization | Authoritative lowering diagnostic |
| Stock note field references | Required fast-fail in `Note.text/html/sound/image(...)` for `basic` and `cloze` ids | Authoritative lowering diagnostic |
| Custom note field references | Required `ValidationError` in `Project.add_note(...)` for current field keys, repeated during Python serialization when a note references a field key missing from its registered `NoteType` | Authoritative lowering diagnostic |
| Custom note type identity availability | Required serialization-time `ValidationError` when a custom note type has no identity fields and any note of that type omits `stable_id` | Authoritative lowering diagnostic |
| Required field completeness | No Python preflight for missing or empty required fields | Authoritative typed-content/media diagnostic |
| `compare_to` readability/existence | No preflight | Authoritative baseline diagnostic |
| output `.apkg` parent existence/writability | No Python preflight | Rust creates parent dirs or returns output diagnostics |
| `report_json` writability | No preflight | Authoritative report/write diagnostic |

## Reports And Exceptions

Python exposes `BuildReport` as a typed projection over `BuildReportJson`, including:

- artifact path
- counts
- media summary
- diagnostics
- inspect summary
- update safety summary
- diff summary
- risk summary
- status and comparison status

Top-level report projections should use simple dataclasses for counts, media summary, diagnostics, artifact, inspect summary, and status. Diff, risk, and update-safety can remain plain `dict | None` in the first release, with typed projections added later.

`BuildReport` projects from `contracts/schema/build-report.schema.json`. Python must minimally require `kind == "anki-forge-build-report"`, `schema_version == "phase4-build-report-v1"`, `status`, `comparison`, `counts.notes`, `counts.cards`, `counts.media`, the Phase 4 media summary fields, `diagnostics`, `metrics.duration_ms`, and `policy`. Optional report objects such as `artifact`, `inspect`, `previous_inspect`, `update_safety`, `diff`, and `risk` are projected when present and otherwise exposed as `None`.

`comparison` is a required enum projection with values `not_requested`, `complete`, `partial`, and `unavailable`. When `compare_to=None`, Rust emits `comparison: "not_requested"`; it is not absent and not `null`. `diff`, `risk`, `previous_inspect`, and `update_safety` may be absent or `null` according to the report schema.

Current repository inventory: `contracts/schema/build-report.schema.json` already declares the required `comparison` enum. Phase 5A should preserve that contract and add regression coverage if Python projection depends on it; no new report-schema axis is required for this field unless the schema changes before implementation.

Rust transport structs should comment that `product_document_version` is the input transport format version, while `BuildReportJson.schema_version` is the output report schema version. They are intentionally separate compatibility axes.

`DiagnosticsError` is the primary Product API failure exception raised by `BuildReport.ensure_success()`. It includes:

- `message`
- `report`
- `diagnostics`
- `status`
- `exit_status`
- `stdout`
- `stderr`

If `product-build` exits non-zero but stdout contains valid report JSON, Python returns `BuildReport` with that parsed report. If stdout cannot be parsed as a report, Python raises `RuntimeInvocationError` that preserves argv, stdout, stderr, exit status, and runtime details.

`BuildReport.ensure_success()` raises `DiagnosticsError` when report status is invalid, blocked, error, missing an artifact, or contains error diagnostics. It does not raise for warning-level diagnostics on a successful build; callers that want warnings to fail must inspect `report.diagnostics` and apply their own policy.

For Phase 5, "error diagnostics" means diagnostics whose Rust/contract severity is exactly `error`. Current build-report diagnostics expose `warning` and `error`; if future reports add `critical`, Python should treat it as error-or-worse under the same `ensure_success()` gate.

Exception hierarchy:

```text
AnkiForgeError
  DiagnosticsError
  RuntimeNotFoundError
  RuntimeInvocationError
  ProtocolError
  ValidationError
```

`DiagnosticsError` is for parsed build reports that failed. Runtime and protocol errors are separate subclasses so callers can distinguish build diagnostics from local execution failures while still catching `AnkiForgeError` for all package errors.

Use this boundary:

- `RuntimeNotFoundError`: no bundled, workspace, or explicit runtime can be resolved.
- `RuntimeInvocationError`: local temp-file serialization/setup fails, subprocess cannot spawn, is interrupted or terminated by signal, exits non-zero without parseable report JSON, or fails before protocol validation can run.
- `ProtocolError`: subprocess exits successfully but stdout is not valid report JSON or does not match the expected report contract.
- `DiagnosticsError`: report JSON is parseable and contract-valid, but the report status or diagnostics indicate build failure.
- `ValidationError`: Python-side argument or object validation fails before subprocess invocation.

`RuntimeInvocationError` must expose a stable `.kind` string so callers can distinguish common local/runtime failures without matching messages. Phase 5 kinds are `setup_failed`, `spawn_failed`, `interrupted`, `decode_failed`, and `exit_without_report`. `decode_failed` means stdout or stderr could not be decoded as UTF-8. `exit_without_report` means the process exited non-zero and decoded stdout was empty, non-JSON, or not a contract-valid report. The exception also exposes `argv`, `exit_code`, `stdout`, `stderr`, and `__cause__` when available.

Python is responsible for minimal report contract validation after JSON parsing. A successful process whose stdout is missing required report keys, has wrong required-key types, or is not a JSON object raises `ProtocolError`, not `DiagnosticsError`. Unknown extra fields are preserved or ignored according to the `BuildReport` projection and do not by themselves raise. If Rust can produce a contract-valid report whose status/diagnostics describe an internal build problem, Python returns `BuildReport` and `ensure_success()` raises `DiagnosticsError`.

Exit code 0 with empty, non-JSON, or non-contract stdout always raises `ProtocolError`. Exit code 0 with non-UTF-8 stdout/stderr raises `RuntimeInvocationError(kind="decode_failed")` because Python cannot inspect the protocol payload. Keep `ProtocolError` as a separate exception class for Phase 5. It means the CLI/runtime violated the report protocol after starting successfully, while `RuntimeInvocationError` means local setup, decoding, or process execution failed before Python could trust the protocol. Both remain under `AnkiForgeError` for broad catches.

The `decode_failed` classification is deliberate: Python must decode stdout/stderr before it can reliably determine whether the runtime produced a protocol payload, so non-UTF-8 output stays in the runtime-invocation bucket even when the process exits 0.

## Packaging And Release

The publish package should be named `anki-forge`, with import path `anki_forge`. The dash in the package name and underscore in the import name follow normal Python packaging convention.

Wheel contents must include:

- Python `anki_forge` package.
- Platform-matching `contract_tools` executable.
- `contracts/manifest.yaml` and required bundle assets.

Bundle only runtime contract assets needed by `product-build`: `contracts/manifest.yaml`, referenced schema files, policy files, and semantics files required by manifest resolution. Do not include `contracts/fixtures/`, test-only fixture indexes, or generated test artifacts in wheels.

Phase 5B must generate a deterministic bundle file list from `contracts/manifest.yaml` and the manifest asset references that `product-build` resolves, then smoke-test the installed wheel using only that bundled directory. Missing-file failures in that smoke test block release.

The current manifest model is a static asset map, so Phase 5B can enumerate `contracts/manifest.yaml`, declared asset paths, and the runtime asset references reached by `product-build`. If manifest resolution grows globs, conditionals, or environment-variable interpolation before Phase 5B, add a deterministic `contract_tools list-assets --manifest <path>` command or equivalent and make wheel assembly consume that output.

Phase 5 pins the manifest format to the current static asset-map model for packaging. Any dynamic manifest feature added before Phase 5B blocks wheel assembly until `contract_tools list-assets --manifest <path>` or an equivalent deterministic enumerator exists.

Release automation should build platform wheels in CI by compiling the Rust `contract_tools` binary on each target runner, then copying the executable and contract assets into Python package data before wheel assembly. Phase 5 should not download opaque pre-built binaries during wheel build.

Use setuptools as the first Phase 5 wheel build backend, extending the current Python packaging setup to include platform-specific package data. Revisit the backend only if setuptools cannot produce correctly tagged platform wheels with bundled executables.

The wheel layout should be stable:

```text
anki_forge/
  _runtime/
    contracts/
      manifest.yaml
      ...
    bin/
      contract_tools      # contract_tools.exe on Windows
```

`runtime.py` resolves the bundled binary relative to `anki_forge.__file__`, not from `PATH`. Workspace mode remains available for repository tests.

A packaging smoke test must install the wheel into a clean virtual environment and build a basic deck without a Rust toolchain.

Required first-release wheel targets:

| Platform | Required wheel tag / architecture | Runtime decision |
| --- | --- | --- |
| Linux | `py3-none-manylinux_2_17_x86_64.manylinux2014_x86_64` | Build `contract_tools` for `x86_64-unknown-linux-gnu` inside a manylinux2014-compatible environment or repair to that tag. |
| macOS Intel | `py3-none-macosx_10_13_x86_64` | Build a separate x86_64 `contract_tools`; no universal2 wheel in Phase 5. |
| macOS Apple Silicon | `py3-none-macosx_11_0_arm64` | Build a separate arm64 `contract_tools`; no universal2 wheel in Phase 5. |
| Windows | `py3-none-win_amd64` | Build with the MSVC Rust target and rely on the system UCRT/MSVC runtime only; no MinGW runtime DLLs. |

macOS CI should set `MACOSX_DEPLOYMENT_TARGET=10.13` for the Intel wheel and `MACOSX_DEPLOYMENT_TARGET=11.0` for the Apple Silicon wheel before compiling `contract_tools`, then smoke-test the built executable on the oldest available compatible runner.

Linux aarch64 and Windows arm64 wheels are explicitly out of scope for the first Phase 5 release unless CI already provides a low-friction runner. Adding them later should not change the Python API or ProductDocument transport.

Windows wheels must validate the bundled executable in a clean virtual environment on a Windows runner. If `contract_tools.exe` has non-system DLL dependencies, the release workflow must either bundle them next to the executable or change the build configuration to avoid them. The smoke test is the release gate for this.

Phase 5A runs the first Windows binary dependency preflight and owns any Rust build-configuration fixes required for a clean standalone executable. Phase 5B repeats the dependency preflight on the final CI-built `contract_tools.exe`, then runs the clean-venv smoke test after wheel installation. If bundling non-system DLLs becomes the selected fallback, bundle them next to `contract_tools.exe` under `anki_forge/_runtime/bin/`, include any required license files in package data, and rerun the clean-venv smoke test from an environment that does not have those DLLs on `PATH`. This fallback changes release risk and must be explicitly reviewed before Phase 5B proceeds.

The bundled Rust CLI and contract assets will make the wheel larger than a typical pure-Python package. Phase 5 does not set a hard size budget, but release notes should make the bundled-runtime trade-off explicit.

The current `bindings/python/pyproject.toml` package name `anki-forge-python` should not be the final user-facing package name for Phase 5.

Minimum Python version is 3.11 for the first Phase 5 release, matching the current Python binding README and CI. The package can lower that floor later after wheel and typing support are stable.

Supported Python range for the first release is 3.11 and newer. CI must at least run the wheel smoke suite on Python 3.11 and 3.12. Additional newer stable versions can be added opportunistically, but 3.11 and 3.12 are the minimum release gates for Phase 5.

PEP 604 union syntax such as `str | None` is allowed because the minimum Python version is 3.11. Use `from __future__ import annotations` in public modules if it keeps forward references and import-time typing dependencies simpler, but it is not required for the version floor.

The public `anki_forge` API should ship inline type annotations for Phase 5. CI should run mypy on Python 3.11 over the public `anki_forge` package with a release-gate configuration that at minimum enables `disallow_untyped_defs`, `disallow_incomplete_defs`, `no_implicit_optional`, `warn_return_any`, `warn_unused_ignores`, and `strict_equality`. Full `mypy --strict` can be a follow-up ratchet once JSON projection code and dataclasses settle.

Do not include `anki_forge_python` in the public `anki-forge` wheel by default. It remains a workspace/dev package for existing low-level wrapper tests. If shared code is needed, move it under `anki_forge._runtime` or another private module rather than shipping a second top-level public package.

`report_json` in `Project.write_apkg(..., report_json=...)` is `str | os.PathLike[str]`, including `pathlib.Path`. It is not a bytes path, file-like object, or boolean flag. Python validates that the argument is path-like, converts it to an absolute path before invoking `product-build`, but does not preflight writability.

Python rejects `report_json` when its resolved absolute path equals the target `.apkg` path or the `compare_to` path. It does not need to compare against the temporary ProductDocument path because Python owns that path inside a fresh `TemporaryDirectory()`.

When `compare_to` is set, the JSON file contains the full `BuildReportJson`, including comparison, diff, and risk fields when Rust produced them. When `compare_to` is `None`, the JSON file still contains the full `BuildReportJson`; comparison, diff, and risk fields are absent or `null` according to the Rust report schema.

Existing `report_json` files are overwritten by Rust. If Rust reaches report generation, it should write `report_json` for success and diagnostic/policy failure reports before Python returns the `BuildReport`. A written `report_json` file persists on disk even if the caller later invokes `report.ensure_success()` and receives `DiagnosticsError`; cleanup is the caller's responsibility. If writing `report_json` fails after an otherwise successful build, Rust must reflect that failure in the stdout report with an error diagnostic/status so Python returns `BuildReport` and `ensure_success()` raises `DiagnosticsError`; the requested file is not guaranteed to exist or contain fresh content. If Rust fails before producing report JSON, Python raises `RuntimeInvocationError` or `ProtocolError` and does not synthesize the file.

The `compare_to` parameter docs must state that `fail_on=None` disables only the risk-threshold policy. It does not ignore baseline read/inspect failures; unavailable baselines still produce invalid reports.

## Documentation

Python docs should start with Product API examples:

1. Basic deck quick start.
2. Long-term `Project` with stable IDs.
3. Custom note type with fields and templates.
4. Media registration with `sound()` and `image()`.
5. Diagnostics and `BuildReport`.
6. genanki concept migration guide.

The quick start and API reference must prominently state that `Project`, `NoteType`, `Note`, and `MediaRegistry` are mutable builders and are not thread-safe. Users building in async apps or thread pools should create independent projects per task or synchronize their own mutations.

The `Project` class docstring must also state the non-thread-safe mutable-builder constraint because it is the first API surface most users will inspect.

The `Project.default_deck` docs should include a small late-binding example showing that changing `project.default_deck` between two `write_apkg()` calls changes deck resolution for notes without explicit `deck_name`.

The genanki migration guide must call out that `Note.cloze(text, back_extra=...)` treats `text` as explicit HTML so cloze markers survive, while `back_extra` is safe text. Users who interpolate untrusted text into the cloze body should escape it themselves or use safe-text helpers in custom note fields. The guide must also explain Basic safe text versus HTML: `Note.basic(front="<b>hi</b>", ...)` escapes the tags, while `Note("basic").html("front", "<b>hi</b>")` renders HTML.

The diagnostics guide should explain that `source_path` values are diagnostic address strings, not a public object access API. `project.notes[3]` in a report means the fourth note in the serialization that produced that report; the address is scoped to that report and may decay after project mutation or note reordering. Users who need long-lived traceability should set explicit note `stable_id` values.

The `project.notes[...]` syntax is intentionally kept as a diagnostic address convention even though there is no public `Project.notes` collection. Do not switch to a separate `notes@3` grammar in Phase 5; instead, make the "not public object access" warning explicit in docs and error examples.

The diagnostics guide should explicitly explain that required media fields with valid media ids can fail as media-source diagnostics rather than required-field diagnostics when their file source is unreadable or missing. Users should not filter only for required-field codes when deciding whether a media-heavy build is safe.

The diagnostics guide should also explain that `project.note_types["basic"]` and `project.note_types["cloze"]` may refer to Python-generated stock declarations, even though users never passed those note types to `Project.add_notetype(...)`.

The low-level contract wrapper remains documented separately for advanced workflows.

## Testing

Phase 5 needs these test layers:

- Python unit tests for object modeling, ProductDocument dict serialization, and report/diagnostic projections.
- Golden tests comparing Python-generated ProductDocument JSON with Rust-side expected snapshots. Store shared transport fixtures under `contracts/fixtures/product-v2/` so Rust and Python tests read one source of truth. Rust owns the canonical fixture shape because Rust owns transport deserialization; Python tests validate that serialization matches those fixtures. Fixture updates are explicit source changes reviewed in git and should land with matching Rust/Python test updates in the same PR; tests compare exact JSON rather than auto-updating snapshots in CI.
- Golden stock fixtures for Basic and Cloze must assert omitted stock identity, exact template strings, display-name template references, stock field keys, and cloze generation rules.
- Transport-order fixtures should include a hand-written `product-v2` document whose explicit `note_types` order differs from Python's auto-declaration order and assert Rust lowering/output remains semantically equivalent.
- Custom identity parity fixtures must assert that `product-v2` custom notes using `identity=True` fields derive the same stable IDs as the existing builder-backed Product path, including the `custom.notetype.fields.v1` recipe id and selected `key`/`name`/`value` payload shape.
- Source-path fixtures must cover mixed stable and unstabilized notes and assert unstabilized indexes are absolute positions in the full serialized notes list. Include the exact case `[A(stable_id="a"), B(no stable_id), C(stable_id="c"), D(no stable_id)]` and assert B uses `project.notes[1]` and D uses `project.notes[3]`.
- Validation parity tests for rules checked in both layers: reserved stock ids, duplicate explicit stable IDs, generation-rule field references, empty `all`/`any`/`cloze` generation rules, custom field-key references, missing custom identity with unstabilized notes, duplicate field display names, sort-field uniqueness, and `export_as` filename/source conflicts. Each parity case should assert Python fast-fail behavior and Rust diagnostic behavior for equivalent hand-written product-v2 input.
- End-to-end Python tests for basic, custom note type, and media projects that produce real `.apkg` artifacts.
- Failure tests for duplicate stable IDs, missing media, empty projects, no-cloze Cloze notes, `fail_on` policy, and unwritable `report_json`.
- Runtime tests for bundled, workspace, and explicit override discovery.
- Shared runtime/protocol tests should exercise subprocess argument construction, UTF-8 stdout/stderr decoding, and report parsing expectations for both the new `anki_forge` helpers and the existing `anki_forge_python` wrapper while their implementation code is duplicated.
- Wheel smoke tests in a clean environment without Rust, including at least one path-with-spaces case and one non-ASCII path case on Windows if CI permits.
- Import-isolation smoke test for the built wheel: install only `anki-forge` into a clean virtual environment, assert `import anki_forge` works, assert `import anki_forge_python` fails, and build a basic deck successfully.
- Strict type-checking for the public `anki_forge` package.
- Existing `anki_forge_python` raw/structured tests kept green.

## Implementation Slices

### Phase 5A: Product API And Transport

- Phase 5A.0 transport sizing gate: before user-visible Python API work claims compatibility, implement or spike Rust `product-v2` parsing for one Basic fixture, one custom typed-content fixture, and one media fixture. This gate must validate both Rust feasibility and the Python API's serialized shapes against real fixtures. It must also quantify current CLI protocol behavior for `--output contract-json`, empty projects, and policy-failure stdout JSON before committing to the Phase 5A timeline. If fixtures force different field shapes, metadata conventions, or API assumptions than this spec describes, implementation must stop, this spec must be updated and externally re-reviewed, and the user-review gate must run again before Python implementation proceeds. If this exposes broad Rust pipeline refactors beyond transport deserialization, lowering, and diagnostics, stop and split that Rust transport work into its own implementation plan.
- Extend Rust `ProductDocument` transport to accept explicit `product-v2` while preserving unversioned `product-v1` compatibility.
- Add shared `contracts/fixtures/product-v2/` transport fixtures and Rust deserialization/lowering tests.
- Verify or preserve `BuildReportJson` schema support for the required `comparison` enum field (`not_requested`, `complete`, `partial`, `unavailable`) before Python report projection depends on it.
- Add or update Rust `product-build` behavior for empty projects so they return an invalid report with a structured diagnostic and no artifact.
- Verify or add Rust behavior where `--compare-to` without `--fail-on` still computes comparison, diff, and risk report sections without applying a policy gate.
- Add the stdout-purity preflight test for `product-build --output contract-json`; Phase 5A blocks if the flag is missing, stdout is not report JSON, or human logs appear on stdout.
- Add Rust-side atomic `.apkg` replacement in the Product build path. Current Product build copies the writer artifact to the requested output path directly, so this is new Phase 5A Rust work: write to a temp file in the target directory and replace after success.
- Run an early Windows `contract_tools.exe` dependency preflight three times: first on the current main-branch binary, again after product-v2 Rust work lands in Phase 5A, and finally in Phase 5B after the release binary is built and installed from the wheel. Phase 5A owns the first two preflights and any Rust build-configuration fixes needed to produce a clean standalone `contract_tools.exe` with only system runtime dependencies. The same Windows passes should include a non-ASCII `--product-input` path. Phase 5B starts wheel assembly only after the post-product-v2 preflight is clean, then repeats the preflight after wheel installation as the release gate. If the only viable fix is bundling non-system DLLs, use the wheel layout defined above and review the packaging plan before Phase 5B proceeds.
- Add public `anki_forge` Python package.
- Implement public `anki_forge` subprocess/runtime/report helpers independently in Phase 5A. Do not extract shared helpers from `anki_forge_python` during this phase.
- Implement Product object model and `to_product_document()` against those fixtures.
- Add `product-build` invocation and Python `BuildReport`/`DiagnosticsError`.
- Ship runnable Python examples for basic, custom note type, and media.

Phase 5A is intentionally sequential at the transport boundary: Rust `product-v2` parsing and fixtures land first, then Python serialization targets those fixtures. Python object modeling can start in parallel only if it does not claim wire compatibility until fixture parity passes.

Expected Phase 5A integration work includes product-v2/product-v1 dispatch, typed content lowering, stock identity recipe routing for product-v2 stock notes, media-id lookup, and source-path propagation. "Broad Rust pipeline refactor" means changes beyond those boundaries, such as Normalized IR schema changes, writer-core public API signature changes beyond accepting already-lowered field HTML/media fragments, changes to the identity recipe semantics in `contracts/semantics/note-stable-id.md`, or product-v2 fixtures proving that the Python API cannot represent the required Rust shape without API changes.

Expected Rust files/modules for Phase 5A changes include `contract_tools/src/product_build_cmd.rs`, `anki_forge/src/product/model.rs`, `anki_forge/src/product/lowering.rs`, `anki_forge/src/product/project.rs`, and focused tests/fixtures. Touching writer-core public APIs, contract semantics docs, or normalized schema files is a stop-and-replan signal unless the change is purely test coverage.

The 5A.0 gate specifically gates transport shape and lowering feasibility. Other Phase 5A Rust tasks, such as atomic `.apkg` replacement, stdout purity, empty-project reporting, and Windows binary preflight, are independent Phase 5A work items. If any of those independent tasks requires a broader refactor than scoped above, split that task into its own reviewed plan without blocking already-validated product-v2 fixture work.

Those independent work items each need a mini-gate before Python API compatibility is claimed: atomic replacement must preserve an existing target on simulated write failure, stdout purity must prove `--output contract-json` emits only one report JSON object, empty-project behavior must return an invalid report and no artifact, and Windows preflight must prove dependency and non-ASCII argv behavior. Atomic replacement tests should include at least one forced failure path, such as target-directory permission failure, temp-file/replace failure, or a Windows locked-target case when CI can exercise it. A failed mini-gate splits that task into its own plan; it does not invalidate completed transport fixture work unless it changes the ProductDocument shape or Python API.

Stdout purity is a hard gate only for Python runtime integration and release readiness. Rust product-v2 transport fixtures, lowering parity, and Python object modeling may proceed in parallel while a logging/CLI fix is split out, but `Project.write_apkg()` cannot be considered compatible and user-visible examples cannot be marked passing until `product-build --output contract-json` satisfies the JSON-only stdout contract.

### Phase 5B: Packaging And Adoption

- Bundle `contract_tools` and contract assets into wheels.
- Add clean-venv wheel smoke tests.
- Write Python quick start and genanki migration guide.
- Add release workflow artifacts for Linux, macOS, and Windows.
- Preserve low-level wrapper documentation for advanced users.

## Acceptance Criteria

1. `pip install anki-forge` provides `import anki_forge`.
2. Basic, custom note type, and media Python examples run without a local Rust toolchain.
3. Python Product API builds through the Rust Product/IR/build/report pipeline.
4. Python exposes structured diagnostics exceptions, not only strings.
5. `BuildReport` exposes counts, media summary, artifact path, diagnostics, inspect, diff/risk where available, and status.
6. Documentation guides genanki users by concept migration.
7. CI validates Linux, macOS, and Windows wheel artifacts.
8. Existing low-level Python wrapper tests continue to pass.
