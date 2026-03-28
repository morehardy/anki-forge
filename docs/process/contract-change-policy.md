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
