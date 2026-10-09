# Troubleshooting

Use the structured `kind` and `code` from the failing operation, plus its source
chain and observations. Display text is for humans and may change. A failed
operation and a completed comparison whose policy blocks publication are
different results.

## Installation

| Symptom | Check |
| --- | --- |
| Rust rejects the toolchain | Run `rustc --version`; use Rust 1.92+ and check `cargo tree -i ankiforge` for the installed version |
| Node cannot load its native package | Run `node -p "process.platform + '/' + process.arch"` and `npm ls ankiforge`; reinstall with `npm install --include=optional ankiforge@0.3.0` |
| Node reports a native/wrapper mismatch | Use matching package versions; reinstall from the same registry version, or rebuild both from the same checkout |
| Python cannot load its extension | Run `python -m pip show ankiforge` and `python -m pip debug --verbose`; check interpreter and wheel OS/CPU tags |
| Python selects a source distribution | Choose a matching wheel/interpreter, or follow [source builds](source-builds.md) with a compiler |

See [verified environments](compatibility.md), [Node setup](node/quick-start.md)
and [Python setup](python/quick-start.md).

## Models and notes

| Symptom | Correction |
| --- | --- |
| Invalid or empty key | Supply an explicit stable key; display names are separate |
| `NOTE.KEY_DUPLICATE` | Give each logical source record one unique key |
| `NOTE.FIELD_UNKNOWN` | Assign the field's key, not its display name |
| `NOTE.FIELD_REQUIRED` | Fill fields declared `required()` before adding |
| `NOTE.MODEL_CONFLICT` | Reuse the same completed model for one model key |
| Template reference error | Use declared keys in templates; inspect the original source byte range |
| HTML appears literally | Use `Content::html` only for intentional markup |
| Missing Cloze cards | Supply `{{c1::answer}}` syntax with supported numbers 1–500 |

An add failure is atomic. Correct the note/model and retry; no partial model or
media registration needs to be undone. Model builders must finish with `.build()`
before they can create custom notes.

## Media and templates

| Symptom | Correction |
| --- | --- |
| File import fails | Check the source at `Media::file` time; later source deletion is supported |
| MIME/content mismatch | Pass the actual MIME type to the bytes constructor |
| Export-name conflict | Choose unique portable names; case and Unicode normalization variants conflict |
| Missing raw HTML/CSS/script resource | Declare it with model `asset` or project `add_asset` |
| Media import exceeds a limit | Explicitly raise `MediaLimits.max_bytes` before importing |
| Build inspection exceeds a limit | Adjust the relevant `InspectLimits` counter for this operation |
| Bundle load fails | Use bundle-v2, explicit keys, key-based templates and paths within the bundle |
| IO rectangle fails | Check decoded display dimensions, finite coordinates and unique mask keys |

Text defaults are uniform across all note kinds. Media nodes collect dependencies;
manually writing a filename does not import its bytes. A valid package still
requires client playback checks for the codec/font you intend to use.

## Updates and outputs

Use an original distribution APKG containing complete identity evidence as the
baseline. Anki re-exported packages do not retain that evidence. A namespace
mismatch or corrupt evidence cannot be accepted by an update policy.

A completed comparison can have `allows_publication == false`. Inspect original
findings and evidence before selecting an explicit risk-category allowance.
Do not blanket-allow every result. The same policy must be passed to the actual
update build if you want the same decision. A policy on a create-only request
is an error.

If a temporary path disappeared, retain its output/artifact owner until all
readers finish. Reports and JSON snapshots are not owners. For persistence
failures, inspect the error's publication stage and durability: an error can be
returned after a destination was already published. See
[output guarantees](build-guarantees.md).
