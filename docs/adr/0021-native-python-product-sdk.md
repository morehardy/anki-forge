# ADR 0021: Own Rust product objects behind the Python SDK

Status: accepted for implementation; native trial and release gates are open.

Python 0.1 passes a ProductDocument to a bundled CLI. Its stateless transport
cannot retain Project registration evidence, incremental validation, or temporary
Artifact ownership. The Python parity plan now requires these consumer behaviors,
including the real Deck facade and conversion to an editable Project.

Python 0.2 will use a private PyO3 extension that owns actual Rust objects. The
Python facade keeps familiar constructors and mutable authoring inputs; adding
an input takes a snapshot and calls the Rust facade. Rust owns identity, template
validation, media evidence, build, comparison, risk and publication. Existing
Field(identity=True) declarations become a type-level recipe unless an explicit
recipe is supplied. Rust remains the only identity algorithm.

This supersedes the Phase 5 bundled-CLI/no-PyO3 choice for the new minor line.
The 0.1 maintenance line remains the compatibility route for RuntimeOverride,
ProductDocument export and mutation after addition. The native wheel does not
silently fall back to the CLI or carry a second default runtime.

The adapter holds an exclusive operation lease per Project/Deck, releases the
interpreter during expensive Rust work, restores the object after domain errors,
and retires it after an unwinding panic. It does not claim cancellation or
multi-file rollback. Owning Artifact references keep temporary files alive;
closing the last handle releases them. Path-only JSON confers no ownership.

Native packaging is a decision gate, not an assumed performance improvement.
The trial must establish a Basic export, independent Rust parity, media change
detection, Artifact lifetime, Rust 1.92 compatibility, platform wheel installation,
source-package completeness and measured CLI/native costs. The core retains its
unsafe-code prohibition; generated extension FFI has a separate lint boundary.

If native distribution fails the trial, reconsider the documented stateful CLI
alternative before expanding the implementation. Four platform installations,
old-artifact migration, typing and the capability matrix remain release gates.
This implementation does not authorize publishing a package or release tag.

Scope and acceptance criteria:
[implementation plan](../plans/2026-09-21-python-api-parity-implementation-plan.md),
[gap audit](../plans/2026-09-21-python-api-gap-audit.md).
