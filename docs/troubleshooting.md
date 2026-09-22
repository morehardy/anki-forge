# Troubleshooting

Start with the failed operation and its diagnostic code. Fix the cause, repeat
the same small example, and inspect the new report before adding more content.

## Installation

| Symptom | Check | Action |
| --- | --- | --- |
| Rust rejects the compiler version | `rustc --version` | Use Rust 1.92 or later |
| Cargo cannot find the local dependency | Path in your application's `Cargo.toml` | Point it at the checkout's `anki_forge` directory |
| Python reports `BINDING.EXTENSION_UNAVAILABLE` | Interpreter/platform and installed package | Install a matching wheel or build with Maturin; `PYTHONPATH` alone is insufficient |
| Python reports `BINDING.VERSION_MISMATCH` | Mixed Python and native files | Reinstall a matching package in a clean environment |
| Node cannot load its native package | Node version and platform package | Follow the source setup, then run its installed-package check |
| A public package installation cannot find the candidate | Actual published versions | Use the documented source setup; declared versions do not prove publication |

See [Rust installation](installation.md), [Node installation](node/quick-start.md),
[Python installation](python/quick-start.md), and [release status](compatibility.md).

## Authoring and validation

Duplicate or blank IDs, unknown note types and field keys can fail when a note
is added. In Rust, match the typed `ErrorCode` through `ErrorCodeExt`; in Node
and Python, inspect the structured error's code. Avoid matching message text.

A validation checkpoint checks authoring structure. Media availability,
normalization, writer checks and update safety still run during a build.
A successful `validate()` alone is not proof that an APKG can be exported.

## Media and templates

| Symptom or code | Likely cause | Fix |
| --- | --- | --- |
| Missing media | A field/template/CSS references an unregistered local filename | Register that exact export filename or remove the reference |
| `MEDIA.SOURCE_CHANGED` | A registered file changed before export | Keep inputs unchanged; rebuild the project from the intended current files |
| Filename collision | Different content shares an export name | Assign distinct filenames and update references |
| `MEDIA.UNUSED_BINDING` | Registered media is not referenced | Remove it or reference it from a note, template or CSS |
| Inline size failure | A Project byte payload exceeds the inline limit | Register a file and use the normal path-backed build |
| Template field not found | HTML uses a key instead of a display name, or a misspelling | Match declared display names in HTML; use keys in rules and authoring |
| No card generated | Generation rule lacks the fields it needs | Supply those fields or choose an appropriate explicit rule |
| `PRODUCT.CLOZE_MARKER_MALFORMED` | Invalid Cloze markup, including the known hide-one IO limitation | Check markers and use hide-all-guess-one for current IO exports |

For image/audio playback problems, first check the registered filename, then the
actual file and codec in the target Anki client. Package validation is not a media decoder.

## Updates and outputs

| Symptom or code | Fix |
| --- | --- |
| Imported content does not change | Keep stable IDs, build against the previous APKG or maintained lockfile, and check Anki import settings/local edits |
| Missing revision evidence | Recover the last distributed APKG; do not invent or discard identity evidence |
| `PROJECT.PATH_COLLISION` | Separate baselines, outputs, lockfiles, report JSON and writable staging directories |
| `INSPECT.RESOURCE_LIMIT_EXCEEDED` | Inspect the input and report; raise only the relevant budget for a trusted large package |
| Temporary artifact disappears | Retain the owning report/handle or persist the artifact before releasing it |
| Report JSON requires an output | Set `output` or `artifacts_dir`; JSON cannot keep a temporary APKG alive |

Read [updates](updates.md), [output ownership](build-guarantees.md), and
[Python diagnostics](python/diagnostics.md) for complete behavior.

## Report a reproducible issue

Include the language/package version, OS, smallest input and code that fails,
diagnostic code/report, and whether the failure occurs at registration,
validation, build or Anki import. For import problems include the Anki version.
Use a small shareable dataset and attach the previous/current package only when
needed to demonstrate the update behavior.
