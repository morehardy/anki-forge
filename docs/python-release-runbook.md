# Python / PyPI release runbook

The public distribution is `ankiforge`; users import `ankiforge`. Only a
protected `python-vX.Y.Z` tag can publish. The tag must point to a commit already
on `main`. Python tags do not trigger the independent crates.io release workflow.

## One-time setup

1. On GitHub, create the `pypi` environment with required reviewers. Allow only
   **tags** matching `python-v*` in its deployment policy. If there is only one
   maintainer, do not enable Prevent self-review without adding another reviewer.
2. Create an active tag ruleset for `python-v*`, restrict creation to release
   maintainers, and prevent tag updates/deletion. The workflow checks
   `github.ref_protected` before building a production candidate.
3. On PyPI, configure a Pending Trusted Publisher for the initial release:

   | Field | Value |
   | --- | --- |
   | PyPI project name | `ankiforge` |
   | Repository owner | `morehardy` |
   | Repository name | `anki-forge` |
   | Workflow filename | `python-pypi-release.yml` |
   | Environment name | `pypi` |

   For an existing project, add this publisher under the project's Publishing
   settings instead. Pending publishers do not reserve names. No API token or
   GitHub secret is needed. PyPI account login requires verified email and 2FA.

## Candidate gates

`.github/workflows/python-pypi-release.yml` performs these checks on the exact
release commit:

- The tag, `pyproject.toml`, native Cargo manifest and Python loader binding
  versions agree. The linked core and embedded contract match loader expectations.
- The stable `X.Y.Z` version is absent from PyPI before building. Prerelease
  version/tag formats are not supported by this workflow.
- `scripts/verify-ci.sh` passes, including the release safety unit tests.
- The reusable native workflow builds a standalone sdist and wheels for Linux
  x64 manylinux2014, Windows x64, macOS x64 and macOS arm64. Each cp311-abi3 wheel
  is exercised on ordinary CPython 3.11, 3.12, 3.13 and 3.14. This does not cover
  free-threaded interpreters, subinterpreters or other platforms.
- Exactly five files have the expected project, version, MIT metadata,
  Requires-Python and wheel tags. `twine check --strict` passes.

The `python-pypi-candidate` artifact holds `dist/` (the five distributions),
`release.json` (commit, tag, run ID, metadata, file sizes/hashes) and `SHA256SUMS`.
Its retention is 90 days, subject to repository limits. The upload job consumes
these exact bytes; it does not rebuild distributions.

After a package-name change, rebuild rehearsal candidates from the renamed
source. Do not rename old wheel/sdist files: their metadata and import package
would still refer to the old project.

## Rehearsal and publication

1. Merge the release changes, including consistent versions and compatibility
   documentation. Keep the workflow filename stable because PyPI binds it.
2. Run **Actions → python-pypi-release → Run workflow** on the candidate ref.
   Manual runs execute metadata, quality, build and candidate checks only. They
   never enter the PyPI environment or request upload credentials.
3. Review the successful run's file list, hashes and complete platform results.
   If desired, separately configure TestPyPI and exercise its OIDC upload before
   the first production release; TestPyPI has separate accounts and publishers.
4. Create and push the protected annotated `python-vX.Y.Z` tag on the reviewed
   `main` commit. Do not reuse the Cargo `anki-forge-vX.Y.Z` tag.
5. Review the new tag run's `python-pypi-candidate` artifact and approve its
   `pypi` deployment. The upload job alone receives `id-token: write`; the
   official PyPA action signs/uploads attestations using Trusted Publishing.
6. Wait for the registry verification and all 16 public-PyPI consumer jobs.
   These compare all file hashes, select an actual public wheel with pip, and
   run isolated native, lifetime, media, example and typing checks. Mark release
   availability in the website/docs only after this complete run succeeds.

## Retry and partial upload recovery

PyPI accepts files individually, so a failed upload may leave a partial release.
Do **not** rerun all jobs, rebuild the same version, retag, or use skip-existing.

- Before upload, use **Re-run failed jobs** to retry failed gates with the
  existing successful artifacts. A new manual rehearsal is also safe.
- If upload fails, use **Re-run failed jobs** on the **original tag run**.
  On attempts after the first, the upload planner verifies every existing PyPI
  file against that run's original manifest and stages only missing files.
  All original local files are also rechecked before staging. An unexpected
  file, different hash, yanked file or different run ID stops recovery.
- If all five files arrived despite an upload error, the planner skips uploading
  and proceeds to registry/consumer verification.
- If publication succeeded and a registry or consumer check failed, rerun only
  the failed checks. The workflow does not automatically yank a release.
- If the original artifacts expired or the release is defective, stop recovery.
  Review whether to yank the version, fix the source, increment the version and
  publish a new protected tag. Already uploaded filenames cannot be replaced.

## References

- [PyPI pending publishers](https://docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/)
- [PyPA publishing action](https://github.com/pypa/gh-action-pypi-publish)
- [GitHub environment protection](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments)
- [PyPI yanking](https://docs.pypi.org/project-management/yanking/)
