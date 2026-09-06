# Node SDK release procedure

The repository contains a 0.2.0 candidate. No package has been published by this
implementation task. Do not describe registry availability or unexecuted CI
jobs as verified support.

1. Run `node-sdk-ci` on the exact release commit. All 12 combinations must pass:
   macOS arm64/x64, Windows x64, Linux x64 glibc; Node 22, 24 and 26. Linux builds
   and consumer tests use Ubuntu 22.04 (glibc 2.35). macOS artifacts use a deployment
   target of 11.0; verify the intended minimum macOS version on a real runner
   before promising that baseline. Audit dynamic dependencies with `otool -L`,
   `ldd`/`readelf`, or the Windows PE tools. Electron, musl, Linux arm64 and Windows
   arm64 are outside this candidate's matrix.
2. Run the product and independent Rust/Node parity suites; inspect the evidence
   index in `COVERAGE.md`. Resolve the Rust image-occlusion limitation and finish
   the remaining cases in the implementation plan. Generate Desktop packages with
   `npm run prepare:desktop` and record the real import/rendering results. The
   generated checklist starts pending; automatic inspection is not Desktop QA.
3. Collect each platform's `.node` artifact from that commit into the matching
   `bindings/node/npm/<suffix>/anki-forge.node`. Rebuild the TypeScript output
   with `npm run build:js`. Verify binary provenance and version alignment;
   `npm run check:release` rejects a missing platform artifact.
4. Confirm control of `anki-forge-node` and all four platform package names,
   choose the release version, and update the main manifest, native Cargo package,
   loader version and lockfiles together. `node scripts/platforms.mjs` regenerates
   the platform manifests. Registry E404 alone does not establish publishing rights.
5. Pack each platform and the main package with `npm pack --json --ignore-scripts`.
   Review file lists, LICENSE, THIRD_PARTY_NOTICES.md, declarations, versions,
   checksums and sizes. Keep the libloading ISC notice in every native package. The
   main package must contain no Rust source, `.node` binary, contracts directory,
   build-time dependency or install script. Keep the reviewed tarballs as the
   immutable release artifacts; do not rebuild between review and publishing.
6. Rehearse the exact tarballs against a disposable registry on every supported
   target, with empty caches and no Cargo or development override. Use
   `npm run test:installed -- --all` once all four binaries are collected. Confirm ESM,
   CJS, TypeScript, APKG generation and clear errors when optional dependencies
   are omitted. The existing `test:installed` script performs the host rehearsal.
7. Publish the platform tarballs first under a prerelease tag, then the main
   tarball with the same version and exact optional dependencies. Confirm every
   registry package before promoting the main tag. Public publishing and tag
   promotion are separate final release actions, not performed by build scripts.
8. Smoke-test the public registry on the full matrix. Promote the reviewed version
   only after those checks pass. If promotion must be reversed, move the tag back
   to the previous working version and deprecate the bad version; do not overwrite
   a published version or assume a removed package is safe to republish.

Local commands (Node 22.13+ and Rust 1.92 required for development):

```sh
npm run setup
npm run build -- --release
npm test
npm run test:parity
npm run test:legacy
npm run test:installed
npm run check:package
```

`ANKI_FORGE_NATIVE_PATH` is an absolute-path development escape hatch. Release
consumer tests remove it, `NODE_PATH`, Cargo variables, and executable search
paths. It is not part of the installation contract.
