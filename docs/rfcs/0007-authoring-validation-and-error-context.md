# RFC 0007: Media Usage, Structured Addition Errors and Project Recipe v2

The current typed content API can package audio as an image and images as sound
without diagnostics. AddError identifies failure categories but hides field and
asset locations in prose. Project titles impose validation without contributing
to generated packages. The accepted [design](../plans/2026-09-28-rust-api-validation-and-errors-design.md)
and [ADR 0024](../adr/0024-authoring-validation-and-error-context.md) resolve these
three issues through the existing public authoring interface.

Keep Media.image/sound infallible and validate their usage at Project.add, where
full note and field context exists. Expose AddContext, AddTarget and AddDetail,
with MediaUsage and MediaConflictKind as small domain classifications. Traverse
original nested content to preserve sequence paths and original UTF-8 ranges.
Do not retain Note/Media owners in errors or add recovery/snapshot frameworks.

Delete project title entry points in Rust, Node, Python and tool recipes. Recipe
v2 rejects removed fields, including null values, and rejects v1 without guessing
intent. Model, field, template and deck names retain their behavior. No original
APKG needs migration: namespace identity and identity-v1 are unchanged.

Bundle 1.0.0 to 2.0.0 is behavior_changing_incompatible with migration notes.
Rust and SDK package versions follow their independently verified release
states. Node's native protocol becomes 5. Python's existing metadata check also
requires bundle 2.0.0, rejecting same-package-version stale binaries.

Acceptance covers the design's media matrix, all error locations, atomic failed
additions, resource lifetimes, SDK error typing and source chains, recipe/schema
parity, existing package update identities, and packaged consumers. Contract
inventory and governance verification use the implementation's actual Git base.
