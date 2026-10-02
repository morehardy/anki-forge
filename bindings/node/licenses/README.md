# License fallback sources

`napi-rs.txt` is the upstream MIT license at the locked `napi` 3.12.2
[source revision 444bf29](https://github.com/napi-rs/napi-rs/blob/444bf29b8534216dd1cec4695a71e5996a173e87/LICENSE).
The napi-rs crates declare MIT but omit license text from their Cargo archives.
The shared notice generator uses this file for those crates and includes their
individual versions and registry source links in the resulting notices.

Generate with `python3 bindings/python/scripts/generate_notices.py --binding node`,
then `node bindings/node/scripts/platforms.mjs`. CI checks for drift against
`Cargo.lock`. The generator also reuses the checked-in common dependency license
fallbacks in `bindings/python/licenses`.
