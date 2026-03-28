# Contract Change Policy

This policy applies to changes that affect the contract bundle, its meaning, or
the way contract assets are validated.

- Changes to schemas, semantics, compatibility rules, or the error registry need
  a documented review trail.
- If a change affects external behavior or compatibility claims, add an ADR and
  open an RFC before merging.
- Keep bundle changes incremental and update the manifest in the same change.
- Use the contract tooling to validate the bundle after every contract-affecting
  edit.
- For Phase 1 readiness and release evidence, record the verification commands
  and outputs in `docs/superpowers/checklists/phase-1-exit-evidence.md`.
- The release-readiness bar is `verify`, `summary`, and `package` against
  `contracts/manifest.yaml`; contract-affecting changes should not merge without
  that evidence path being satisfied.
