# Publishing ankiforge to npm

The SDK is `ankiforge`, with four exact-version optional native packages:
`ankiforge-darwin-arm64`, `ankiforge-darwin-x64`, `ankiforge-linux-x64-gnu`,
and `ankiforge-win32-x64-msvc`. All five share one version. The internal Rust
crate remains `anki_forge_node_native`, the addon filename is `anki-forge.node`,
and the binding protocol is **6**. There is no legacy export. A crates.io release
is not a prerequisite for publishing this native SDK.

## One-time account and repository setup

1. Verify the maintainer's npm email and enable 2FA. Confirm ownership of **all
   five** names; a registry 404 does not reserve a name.
2. Create a GitHub environment named `npm` with a required maintainer reviewer,
   and limit its deployment tags to `npm-v*`. Protect those tags with an active
   ruleset: restrict creation to release maintainers and block updates/deletion.
   The workflow rejects unprotected tags and commits that are not on `main`.
3. Require `npm candidate ready` before merging release changes. Keep the
   repository's Rust/API/Anki oracle gates; npm checks do not replace them.
4. Once each package exists, add a Trusted Publisher to **each of the five**:

   | Setting | Value |
   | --- | --- |
   | Provider | GitHub Actions |
   | Organization or user | `morehardy` |
   | Repository | `anki-forge` |
   | Workflow filename | `node-npm-release.yml` |
   | Environment | `npm` |
   | Allowed actions | Allow **npm publish** and **npm dist-tag** |

   CI uses direct OIDC publication after GitHub environment approval. Stage-only
   permission is insufficient. No `NPM_TOKEN` or `NODE_AUTH_TOKEN` is needed.
   The publisher pins Node 24 and npm 11.21.0 for OIDC dist-tag support; consumers
   still support Node 22.13+. Once configured, require 2FA and disallow traditional
   publishing tokens in each package's settings.
5. Review the right to distribute project source, fixtures and assets, including
   outstanding items in the Rust release audit. Generated dependency notices do
   not establish first-party source provenance.

See npm's [Trusted Publisher setup](https://docs.npmjs.com/trusted-publishers/)
and [2FA guide](https://docs.npmjs.com/configuring-two-factor-authentication/).

## Prepare and rehearse

Update the version in `bindings/node/package.json`, all four exact optional
dependency versions, the native `Cargo.toml` package version (and Cargo.lock),
and `VERSION` in `src/internal/native.ts`. Regenerate platform manifests.
The Node SDK version is independent of the Rust core dependency version.

From the repository root:

```sh
cargo fetch --locked
python3 bindings/python/scripts/generate_notices.py --binding node
node bindings/node/scripts/platforms.mjs
python3 bindings/node/scripts/npm_release.py metadata
python3 -m unittest discover -s bindings/node/scripts -p 'test_*.py' -v
cd bindings/node
npm run setup
node scripts/build.mjs --release
npm run check
npm test
npm run test:installed
npm run pack:local
```

Python 3.11+ is a release-tool dependency. Consumers need neither Python, Cargo
nor Rust, and no install hook runs.

`node-sdk-ci.yml` builds and tests macOS arm64/x64, Ubuntu 22.04 x64/glibc and
Windows x64, verifies notices and package payloads, and packs five tarballs
**once**. Sixteen consumer jobs install those same bytes on Node 22.13.0, current
22, 24 and 26. They check optional dependency selection, fresh-cache npm ci,
ESM/CJS identities, TypeScript types, README examples, real APKG output, read-only
node_modules and missing/incompatible runtimes. Product tests also compare the
public API with an independent Rust observer on every platform/Node combination.

Manually run `node-npm-release` to rehearse on a branch. Manual runs cannot
publish or move npm tags. Download `node-npm-candidate`: five tarballs and
`release.json` with commit, ref, run ID, file inventory and SHA-256/SHA-512 hashes.
Retention is 90 days for release rehearsals/tags, 14 days for ordinary CI.

```sh
python3 bindings/node/scripts/npm_release.py verify --directory /path/to/candidate
cd bindings/node
node scripts/installed-smoke.mjs --candidate /path/to/candidate
```

The local test registry serves the frozen tarballs without repacking. Publishing
jobs only accept the candidate from the original protected-tag run.

## First publication: bootstrap ownership

Unpublished names have no package settings in which to register a Trusted
Publisher. Bootstrap is a maintainer action using npm login and 2FA, and creates
real immutable public versions.

Merge the release changes to main and intentionally create the matching
`npm-vX.Y.Z` tag. Let all candidate tests finish. While `publish` waits for
environment approval, download **that run's** candidate, check its commit and
contents, and run the local verification above. Sign in with `npm login`, then:

```sh
# Example: run in the downloaded candidate directory.
npm publish ./ankiforge-darwin-arm64-0.2.0.tgz --access public --tag next --ignore-scripts --registry=https://registry.npmjs.org
npm publish ./ankiforge-darwin-x64-0.2.0.tgz --access public --tag next --ignore-scripts --registry=https://registry.npmjs.org
npm publish ./ankiforge-linux-x64-gnu-0.2.0.tgz --access public --tag next --ignore-scripts --registry=https://registry.npmjs.org
npm publish ./ankiforge-win32-x64-msvc-0.2.0.tgz --access public --tag next --ignore-scripts --registry=https://registry.npmjs.org
# Wait for npm view <each-native-name>@0.2.0 dist.integrity to return all four.
npm publish ./ankiforge-0.2.0.tgz --access public --tag next --ignore-scripts --registry=https://registry.npmjs.org
```

Configure the five Trusted Publishers, then approve the waiting GitHub job.
It skips existing versions only if their bytes exactly match the candidate,
and continues public installation tests and latest promotion. Local bootstrap
does not carry GitHub OIDC provenance; subsequent OIDC releases do. npm also
offers [staged publishing](https://docs.npmjs.com/staged-publishing/) for npm-side
review; maintain the same native-before-main order and `next` tag if using it.
Never substitute a placeholder tarball for the intended version.

## Subsequent releases

Only a protected `npm-vX.Y.Z` tag push can publish. It must match the stable
package version and point to a commit already on main.

1. Reusable CI builds all platforms and tests the frozen candidate. Review its
   evidence before approving the `npm` environment.
2. The publisher rechecks archives and identity, publishes missing native
   packages to `next` with OIDC, waits for their public bytes to match, then
   publishes main. It verifies all five public tarballs.
3. Sixteen public consumer jobs install the exact version from
   **registry.npmjs.org**, with fresh caches and no native compilation.
   Lockfile integrity must match the candidate. They exercise installed API,
   types, README examples and the optional native loader.
4. After all pass, `promote` uses the same environment approval and OIDC to
   update native `latest` tags first, main last. It refuses to downgrade a newer
   release and verifies the resulting tags.

`next` is publicly downloadable, not private staging. Before promotion, use
`ankiforge@next` or an explicit version. The workflow never auto-unpublishes.

## Retry and recovery

- Use **Re-run failed jobs** in the original run, keeping its candidate unchanged.
  Do not rebuild after publication. Re-running all jobs conflicts with immutable
  artifact names and is not recovery. Commit/ref/run/version checks reject
  substituted evidence.
- Partial publication skips byte-identical versions and uploads missing ones.
  Conflicting integrity is an error. Bad immutable versions need a patch release.
- Registry visibility is polled for up to 15 minutes per gate. Fix auth/network
  failures and retry the failed job; do not delete/recreate release tags.
- If public tests fail, latest is not promoted. Partial promotion can be retried,
  but npm cannot atomically change five dist-tags. This workflow serializes
  releases; coordinate any manual tag edits. Rollback/deprecation is a separate
  maintainer decision.
- Back up candidates before retention expires. Missing original evidence blocks
  automated retry; do not publish freshly rebuilt bytes under the old version.

See [npm publish](https://docs.npmjs.com/cli/v11/commands/npm-publish/) for
tarball publication and version immutability.
