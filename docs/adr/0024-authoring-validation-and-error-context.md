# ADR 0024: Media Usage and Authoring Error Context

Decision date: 2026-09-28. Specification:
[authoring validation design](../plans/2026-09-28-rust-api-validation-and-errors-design.md).
Discussion: [RFC 0007](../rfcs/0007-authoring-validation-and-error-context.md).

Project addition validates typed Image/Sound content against the MIME retained
by its owned Media. Images accept image types; Anki sound references accept
audio and video. Unknown categories require an explicit suitable MIME at byte
import. This validates usage, not decoding or playback. Explicit HTML and assets
retain their existing rules. All addition failures remain atomic.

AddError exposes owned context and optional typed evidence. Locations distinguish
note fields, nested content, tags, decks and assets. Errors retain no resource
handles and do not turn diagnostic wording into a machine protocol. Native SDK
adapters expose the same facts, including errors in source chains.

Remove the project-only display title, which was validated but never published.
Keep namespace identity and actual deck/model/field/template names separate.
Tool recipes advance to v2 and reject v1 and the removed top-level name field.
Bundle 2.0.0 records the incompatible input change; APKG identity-v1 and existing
build/report/comparison snapshots remain unchanged. Retain BUILD.NAME_INVALID
as deprecated registry history and register NOTE.MEDIA_USAGE_INVALID.

This supersedes only the corresponding authoring details of ADR 0023. It does
not introduce publication metadata, decoding guarantees or a second authoring
container. Implementation and verification evidence is recorded in the design.
