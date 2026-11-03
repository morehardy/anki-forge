#!/usr/bin/env bash
# Author-run engineering evidence for the 0.3.2 package release candidate.
# Builds each deployment base, proves a same-input rebuild, runs every
# engineering release check, then freezes the artifact for independent review.
set -euo pipefail
root=$(cd "$(dirname "$0")/../../../.." && pwd)
evidence="docs/plans/evidence/release-0.3.2-2026-11-02"
cd "$root/website"
for env in root prefix; do
  if [[ $env == root ]]; then
    export SITE_URL=https://ankiforge.dev SITE_BASE=/ SITE_CUSTOM_DOMAIN=ankiforge.dev
  else
    export SITE_URL=https://morehardy.github.io SITE_BASE=/anki-forge SITE_CUSTOM_DOMAIN=''
  fi
  export SITE_DEPLOYMENT=preview RELEASE_EVIDENCE_DIR="$root/$evidence/engineering/$env"
  mkdir -p "$RELEASE_EVIDENCE_DIR"
  npm run build > "$RELEASE_EVIDENCE_DIR/build.log" 2>&1
  npm run check:rebuild > "$RELEASE_EVIDENCE_DIR/rebuild.log" 2>&1
  node scripts/release-evidence.mjs run --engineering-only > "$RELEASE_EVIDENCE_DIR/run.log" 2>&1
  cp dist/generated/site-manifest.json "$root/$evidence/$env-site-manifest.json"
  rm -rf "$root/target/release-0.3.2-artifacts/$env-dist"
  mkdir -p "$root/target/release-0.3.2-artifacts"
  cp -R dist "$root/target/release-0.3.2-artifacts/$env-dist"
  echo "$env: $(tail -1 "$RELEASE_EVIDENCE_DIR/run.log")"
done
