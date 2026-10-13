# Troubleshooting public Anki Forge 0.3.0

Use the operation's structured error code, source chain and observations.
Do not guess numeric protocol values or substitute undocumented methods.

| Symptom | Executable check or correction |
| --- | --- |
| Rust toolchain/version failure | `rustc --version`; use Rust 1.92+, inspect `cargo tree -i ankiforge` |
| Node cannot load native package | `node -p "process.platform + '/' + process.arch"`; `npm ls ankiforge`; reinstall with `npm install --include=optional ankiforge@0.3.0` |
| Node wrapper/native mismatch | Check lockfile and installed native package versions match the public wrapper |
| Python extension/wheel failure | `python --version`; `python -m pip show ankiforge`; `python -m pip debug --verbose`; compare interpreter/OS/CPU tags to compatibility docs |
| Missing persistent output | Check working directory, `BuildOptions.to` destination and whether output was temporary; retain temporary owners until readers finish |
| Duplicate note key | Assign one stable unique key per source record; do not replace keys with random values |
| Unknown/required field | Assign declared field keys and fill required fields; display names are separate |
| Literal HTML | Use explicit `Content.html` / `Content::html` for intentional markup |
| Missing Cloze cards | Check supported `{{c1::answer}}` deletion numbers and generation rules |
| Missing media | Import bytes through Media; explicitly register raw HTML/CSS/font dependencies |
| MIME/name conflict | Check actual byte format, MIME and extension; use unique portable literal names |
| Invalid occlusion rectangle | Check decoded display dimensions, finite in-bounds pixels, positive dimensions and unique mask keys |
| Update baseline rejected | Use the original distributed APKG and matching namespace; inspect evidence and policy findings |
| Late build/persist failure | Inspect publication path/stage/durability facts before retrying; a file may already have been published |

When a matching public wheel/native package is unavailable, report the specific
platform constraint. Source builds are a separate documented choice, not the
ordinary installation path.

- [Troubleshooting](https://ankiforge.dev/docs/troubleshooting/)
- [Compatibility](https://ankiforge.dev/docs/compatibility/)
- [Node API errors](https://ankiforge.dev/docs/node-api/)
- [Python diagnostics](https://ankiforge.dev/docs/python-diagnostics/)
- [Source troubleshooting](https://github.com/morehardy/anki-forge/blob/main/docs/troubleshooting.md)
