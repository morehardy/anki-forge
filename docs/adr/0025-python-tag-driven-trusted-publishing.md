# ADR 0025: Publish Python Candidates through a Protected Tag and OIDC

Python releases use `python-vX.Y.Z` tags independently of the Cargo tags in
ADR 0010. The authoritative tag points to a commit on `main` and matches the
Python/native binding versions. A protected `pypi` environment gates the single
upload job in `python-pypi-release.yml`, which uses PyPI Trusted Publishing.

## Consequences

- Existing cross-platform build/testing remains reusable. The upload job lives
  in the top-level workflow because PyPI's publishing action does not support
  Trusted Publishing inside reusable workflows.
- Manual dispatch rehearses the same candidate pipeline without publication.
  A tag run rebuilds its own candidate and uploads those tested files.
- One manifest records all five distributions and hashes. Only the original
  run can recover a partial upload, after checking every existing registry file
  and staging only missing original files. Duplicate uploads are not ignored.
- Only the upload job requests OIDC permission. No long-lived registry secret
  is provisioned. The workflow reports completion only after public PyPI file
  and cross-platform installed-consumer checks pass.
