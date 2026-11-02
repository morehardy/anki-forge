# SEO standards review

Reviewer: `/root/standards_review`, 2026-11-02. Scope: working-tree SEO changes against `3e3eebd8bad2a419c1f1ff30fa81b6d7d2e6f628`, including the untracked IndexNow implementation, tests, configuration and proof file. No implementation edits were made by this reviewer.

Sources inspected: `website/README.md`, `website/scripts/site-manifest.mjs`, `website/scripts/release-evidence.mjs`, `website/scripts/post-build-reviews.mjs`, and `.github/workflows/website.yml`. Fowler smell baseline applied as judgement, with repository contracts taking precedence.

## Finding

**[P2] Reject invalid successful baseline responses** — `website/scripts/indexnow.mjs:43-44` uses `if (manifest) pageHashes(manifest)`. An HTTP 200 response containing JSON `null` or `false` therefore skips validation and is saved as an absent baseline, causing `changedUrls` to initialize all current URLs. This violates the documented fail-closed contract in the new README: “only an actual HTTP 404 means no previous manifest exists.” Reproduced locally with `capturePrevious('/tmp/indexnow-review-null.json', async () => Response.json(null))`; it completed successfully and wrote `null`. Validate every HTTP 200 result unconditionally and add a regression for null/false responses. This is a documented-contract violation, not a smell judgement.

## Other observations

The workflow retains the existing strict release gates and runs notification after live artifact verification. Submission checks the expected live manifest and proof file, rejects redirects, constrains routes to the production host, excludes 404, handles deleted URLs, batches at 10,000 and distinguishes 202 from indexed status. The public verification key is intentionally an ownership-proof file. Notification failures preserve manifests for the documented manual retry path. No additional actionable smell findings.

Initial result: one P2 finding; zero smell findings. Production publication and live IndexNow receipt are outside this static review and remain release acceptance requirements.

## Resolution verification

The implementation now asserts that a manifest is an object and calls `pageHashes` for every HTTP 200 capture response. I independently reran `node --test website/scripts/indexnow.test.mjs` with Node 24.19.0: all five tests passed, including the new `null`/`false` successful-response regressions. The original P2 finding is resolved; the final standards result has no open findings. This verification does not substitute for release-artifact or live submission acceptance.
