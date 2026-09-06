# ADR 0017: Own Rust product objects behind the Node SDK

Status: implemented locally; platform and publication gates remain open.

The Node 0.1 wrapper launches `contract_tools` and passes product documents over
JSON. That remains useful for contract tooling, but it cannot preserve a Rust
`Project` or `Deck` between incremental additions, media registration and build.
Reconstructing those objects from JavaScript documents would repeat stateful
validation and lose registration-time evidence.

Node 0.2 exposes a typed facade over napi-rs. The adapter owns Rust product
objects and registered media handles; JavaScript only holds immutable input
descriptions until an addition calls the Rust API. Rust remains authoritative
for identity, escaping, templates, media evidence, build, risk and publication.
This decision is based on product semantics, not a claimed performance gain.

Each native object is Ready, Busy or Failed. An asynchronous operation takes
exclusive ownership before returning its promise, performs work on a worker,
and returns ownership before settling the promise. A domain failure preserves
the object for reuse. An unwinding Rust panic retires that object. The adapter
does not promise cancellation or rollback through a JavaScript timeout. Deck
builds take a Rust snapshot in the worker using `Project::from(Deck)` while
preserving the owned Deck and its identity indexes for future additions.

Tasks run on napi-rs's built-in blocking runtime and settle through `JsDeferred`.
The 3.12.2 `AsyncTask` completion callback can panic when a Worker is terminated
with native work pending, reproduced in debug builds on Node 22 and 24. Deferred
callbacks are drained when their environment closes, releasing the task and its
project lease without trying to invoke JavaScript in that environment. The
built-in runtime retains the addon image while native work can still run. Worker
termination does not cancel a build or roll back its filesystem effects.

Media references retain Rust's filename identity. A real registered reference
may be used across projects; missing destination media is a core build error.
The adapter neither invents media registrations nor rejects references based
on JavaScript object identity. Large Project buffers use owned temporary files
and the core's published inline size limit. All public paths resolve at the
facade's captured base directory; core staging stays in its own artifact
workspace, avoiding writes into the user's input directory.

The CommonJS implementation is canonical. ESM reexports its values so mixed
imports share constructors and native module state. The main npm package has
exact-version optional dependencies on four platform packages. Installation
does not compile Rust, download through lifecycle hooks, discover a checkout,
or launch a CLI. A temporary registry tests actual tarball installation and npm
platform selection; loading errors expose missing runtimes and version mismatch.

The adapter pins napi 3.12.2, napi-derive 3.6.3 and napi-build 2.4.1, requiring
Rust 1.88 or later; this workspace uses 1.92. Node-API 8 is sufficient for the
current implementation. The native crate is a cdylib with Rust test/doctest
harnesses disabled; its public behavior is tested through Node. The core keeps
its workspace `unsafe_code = forbid`; the adapter uses `deny` so generated N-API
FFI can apply its own scoped allowance. No handwritten unsafe block is added.
The adapter specifies the core path dependency's version for the workspace
dependency policy. The `libloading` 0.9.0 ISC exception is limited to that crate
and version; its notice is included in the main npm tarball and all platform
tarballs, with installation tests checking those file lists.

The old API moves to `anki-forge-node/legacy` and still requires its configured
CLI/contracts. Its validation preview forbids retained publication paths and
lockfile writes. The new `Project.validate()` calls Rust validation directly.
The SDK introduces no contract bundle change or alternate identity algorithm.

Release requires all target installations, documented dynamic-library baselines,
API/behavior parity evidence, package ownership, and a registry rehearsal. A
working macOS arm64 package alone does not close those gates. The current Rust
writer rejection of grouped image-occlusion cloze markup is recorded separately
in the implementation plan; the binding preserves that diagnostic.
