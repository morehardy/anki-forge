# ADR 0022: Preserve Node artifact ownership and copy authoring state explicitly

> Historical decision. Current public authoring and update behavior is defined by [ADR 0023](0023-owned-authoring-and-package-update-evidence.md) and the [clean-slate plan](../plans/2026-09-23-rust-api-clean-slate-design.md).

Status: implemented locally; external release gates remain separate.

The supported comparison target is Rust's product interface, as defined by
ADR 0012. Node adapts builders to options, blocking I/O to promises and u64 input
to safe numbers or exact bigints. It does not copy Rust identity/rendering rules.

A build returns JSON plus a real native ApkgArtifact owner on both success and
domain failure. `BuildReport.artifact` remains its compatible path snapshot;
`artifactHandle` is a separately owned capability that cannot be reconstructed
from JSON. The latter can clone, persist and close. This preserves manual report
construction without pretending that a pathname owns a temporary file. Report
and handle garbage collection release ownership; explicit close runs on a worker.
In-flight persistence takes an independent owner before scheduling.

`Project.fromDeck`, `Project.clone` and `Deck.clone` reserve their source and copy
core state in a worker. A new SharedProject has its own Ready/Busy/Failed machine;
cloning its Arc would have shared mutable state and is not the public contract.
Large-buffer staging uses Arc<TempDir> so copies share immutable resources until
the last owner drops. Private adoption creates genuine JS objects with private
fields and never allocates a throwaway Rust Project or bypasses the constructor.

Readonly describe DTOs select product getters and explicitly projected Deck note
fields. They expose canonical keys, identities and rendered fields without
validating incomplete authoring input as a Project addition. The JS facade deeply
freezes snapshots; async Deck.describe reserves a consistent state.

WriteTo builds an owned temporary artifact and transfers it with a 64 KiB readable
buffer, awaiting each write and any required drain. It closes the source before
releasing the temporary artifact (including on Windows), leaves the caller's
Writable open, and preserves the original error if cleanup also fails. This is
bounded transfer of a finished archive, not incremental archive generation.

Binding protocol 2 adds ownership-bearing results, state-copy and observation
methods, and exact decimal strings for bigint inspection budgets. JS checks both
the package version and protocol, so an old same-version development native
binary is rejected. All platform package versions remain tied to the main package;
release publication and version selection continue through RELEASING.md.
