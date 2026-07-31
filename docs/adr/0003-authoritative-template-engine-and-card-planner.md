# ADR 0003: Authoritative Template Engine and Card Planner

## Context

Template parsing, Cloze counting, and card creation previously lived in
different layers and language bindings. That allowed BuildReport estimates to
disagree with the APKG and made custom templates appear valid until Anki import.

## Decision

Rust owns template semantics and card planning. All normal and Cloze card
generation flows through one CardPlanner, and `product-build` is the public
end-to-end acceptance seam. ProductDocument v3 adds explicit normal/custom
Cloze semantics. A versioned directory bundle is the external template
interchange format.

Python and Node are thin adapters to ProductDocument v3 and `product-build`.
They do not own independent template or Cloze parsers. Anki Desktop and the
upstream source mirror are behavioral oracles only; no AGPL implementation code
is copied or linked.

## Consequences

- BuildReport, APKG card rows, and inspect use the same card plan.
- Product v2 retains its existing normal custom-note-type interpretation.
- New public schemas, semantics, fixtures, and diagnostics are discoverable from
  the contract manifest under bundle version 0.2.0.
- Third-party filters can be reported as warnings without claiming runtime
  compatibility.
