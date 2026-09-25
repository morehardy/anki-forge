---
asset_refs:
  - schema/template-bundle.schema.json
  - schema/project-input.schema.json
  - schema/normalized-ir.schema.json
  - errors/error-registry.yaml
---

# Templates and card planning

Native Rust, Node, Python and product CLI declarations share one Rust schema,
compiler and writer. Low-level normalized IR contains Anki display field names;
user templates and note assignment use stable field keys. The compiler rewrites
only parsed field reference source ranges, preserving surrounding markup, scripts,
comments and whitespace. Errors point at original UTF-8 byte positions.

Expressions cover field replacement, filter chains, matching conditional/inverted
sections, comments and Anki special fields. Empty filter segments, unknown fields,
unmatched delimiters and mismatched sections are errors. Unknown well-formed
filters are portability warnings. Recognized filters are cloze, hint, text and
type. Browser front/back templates follow the same binding rules. Runtime HTML,
CSS, JavaScript and third-party filter execution are not statically simulated.

A custom Cloze model selects one declared cloze field and exactly one template;
its front must render that field through cloze. Numbered deletions must be complete,
nonempty, non-nested and use integers 1–500. Repeated numbers produce one card;
distinct numbers map to ordinal N-1. A normal model cannot use cloze filters.

For normal templates, a statically visible front has no required field; direct
alternative field replacements infer any; a positive conditional path can infer
all. An explicit all/any rule lists distinct existing field keys and overrides
inference. Unrepresentable predicates require an explicit rule instead of storing
an approximation that changes card generation after import. The planner supplies
both package card rows and stored Anki generation requirements.

`template-bundle-v2` is a directory with an anki-template.yaml manifest. The model
requires key/fields/templates; names default to keys. cloze_field selects Cloze,
without a second kind flag. Old id/identity/optional fields and unknown properties
are rejected. A Cloze bundle cannot declare a normal generation rule. At most one
field is the sort field. Every template names front_file/back_file; browser files,
stylesheet and explicit path/export_as assets are optional.

Paths must be portable, relative and contained within the canonical bundle root,
including symlink resolution. Inputs must be regular files. The manifest limit is
256 KiB; each template/style text limit is 2 MiB and requires UTF-8. Both declared
length and actual read length are checked. Asset imports use MediaLimits. Loading
returns an immutable NoteType and owned asset closure only after every item has
validated, so source removal cannot invalidate the returned model.
