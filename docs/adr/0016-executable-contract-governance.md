# ADR 0016: Tie Contract Governance to Production Sources and Actual Changes

The existing registry gate validated only entries already in the registry, and
the versioning gate validated static evolution examples rather than a release's
changes. The semantics gate also maintained a second list of asset keys. These
checks could pass while the published contract and its implementation drifted.

Production diagnostic and risk codes are inventoried from Rust syntax in both
source trees, independent of constructor/helper names. Whole code literals must
be registered and not removed. Test-only items and documentation are excluded;
all potentially active production features/platforms are included.

Semantic document discovery follows the manifest's resolved `semantics/` assets
and `_semantics` keys. There is no second per-document allowlist.

Repository CI checks a concrete baseline bundle from the PR base or push-before
commit. It uses the packager's asset closure and requires a change record with
the exact before/after digests, classification, summary and corresponding version
increase. Obvious removals/retargeting and retired codes cannot be classified as
compatible. Semantic compatibility still requires human review; a static file
diff cannot prove that a changed rule preserves user behavior.

Standalone bundle verification remains independent of Git and source checkout.
The repository checks are explicit `verify` options; release automation requires
a previous bundle ref. The new registry entries and governance metadata are
published in bundle 0.6.0 without changing runtime diagnostic strings or the
Rust consumer interface. See RFC 0006 for operational details.
