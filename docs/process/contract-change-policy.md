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
- Run `bash scripts/check_contract_governance.sh` against the branch merge-base.
  CI sets `CONTRACT_BASE_REF` to the PR base SHA or push-before SHA; an explicit
  previous commit/tag may also be supplied locally or for a contract release.
- Add every built-in diagnostic/risk code to the registry before use. Production
  codes must be whole Rust string literals, never synthesized strings; the source
  gate checks constants, helpers, macros, and optional feature/platform branches.
- For published asset changes, bump `bundle_version`, use `contract_tools changes`
  to generate an exact before/after inventory, and reference a completed record
  as `assets.bundle_change`. Review the compatibility class and summary rather
  than accepting the generated template unchanged. Regenerate after further
  contract edits. See `contracts/versioning/policy.md` for the bump rules.
- Record verification results in the PR, including the baseline commit used.
- The release-readiness bar is `verify`, `summary`, and `package` against
  `contracts/manifest.yaml`; contract-affecting changes should not merge without
  that evidence path being satisfied.
