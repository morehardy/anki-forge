"""Release boundary tests: bad artifacts, identity, retries and promotion order."""
import io
import json
import os
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import npm_release as release


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.version = json.loads((release.ROOT / "package.json").read_text())["version"]

    def manifest(self, name):
        source = release.ROOT if name == "ankiforge" else release.ROOT / "npm" / name.removeprefix("ankiforge-")
        return release.read_json(source / "package.json")

    def payload(self, name):
        files = {"package.json": json.dumps(self.manifest(name)).encode(),
                 "README.md": b"readme", "LICENSE": (release.ROOT / "LICENSE").read_bytes(),
                 "THIRD_PARTY_NOTICES.md": (release.ROOT / "THIRD_PARTY_NOTICES.md").read_bytes()}
        if name == "ankiforge":
            files.update({f: b"example" for f in ("dist/index.mjs", "dist/index.d.mts", "dist/cjs/index.js",
                                                "dist/cjs/index.d.ts", "RELEASING.md", "COVERAGE.md")})
            files["dist/cjs/package.json"] = b'{"type":"commonjs"}'
        else:
            files["anki-forge.node"] = b"native fixture"
        return files

    def tarball(self, name, files=None, link=False):
        destination = self.directory / f"{name}-{self.version}.tgz"
        with tarfile.open(destination, "w:gz") as archive:
            for filename, data in (files or self.payload(name)).items():
                member = tarfile.TarInfo("package/" + filename)
                member.size = len(data)
                archive.addfile(member, io.BytesIO(data))
            if link:
                member = tarfile.TarInfo("package/link")
                member.type, member.linkname = tarfile.SYMTYPE, "../outside"
                archive.addfile(member)
        return destination

    def record(self):
        for name in release.NAMES:
            self.tarball(name)
        return release.candidate(self.directory, "a" * 40, f"npm-v{self.version}", "123")

    def test_complete_candidate_roundtrip_and_tampering(self):
        record = self.record()
        self.assertEqual(release.verify_local(self.directory), record)
        target = self.directory / record["packages"][0]["filename"]
        target.write_bytes(target.read_bytes() + b"changed")
        with self.assertRaisesRegex(ValueError, "Candidate changed"):
            release.verify_local(self.directory)

    def test_missing_or_extra_tarball_rejected(self):
        self.record()
        (self.directory / "extra.tgz").write_bytes(b"extra")
        with self.assertRaisesRegex(ValueError, "Tarball set differs"):
            release.verify_local(self.directory)

    def test_duplicate_package_in_record_rejected(self):
        record = self.record()
        record["packages"][1] = record["packages"][0]
        (self.directory / "release.json").write_text(json.dumps(record))
        with self.assertRaisesRegex(ValueError, "four runtimes then main"):
            release.verify_local(self.directory)

    def test_old_name_inexact_dependency_and_lifecycle_hooks_rejected(self):
        for key, value in (("name", "anki-forge-node"), ("optionalDependencies", {release.NAMES[0]: "^0.2.0"}),
                           ("scripts", {"postinstall": "curl example"}), ("publishConfig", {"tag": "latest"})):
            with self.subTest(key=key):
                manifest = self.manifest("ankiforge")
                manifest[key] = value
                with self.assertRaises(ValueError):
                    release.audit_manifest(manifest, "ankiforge", self.version)

    def test_platform_architecture_and_libc_rejected(self):
        manifest = self.manifest("ankiforge-linux-x64-gnu")
        for field, value in (("cpu", ["arm64"]), ("libc", ["musl"]), ("os", ["darwin"])):
            bad = {**manifest, field: value}
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, field):
                release.audit_manifest(bad, bad["name"], self.version)

    def test_main_native_source_leak_and_archive_links_rejected(self):
        for filename in ("native/Cargo.toml", "dist/cjs/leaked.node", "../outside"):
            files = {**self.payload("ankiforge"), filename: b"leak"}
            with self.subTest(filename=filename), self.assertRaises(ValueError):
                release.inspect_tarball(self.tarball("ankiforge", files), "ankiforge", self.version)
        with self.assertRaisesRegex(ValueError, "links are forbidden"):
            release.inspect_tarball(self.tarball("ankiforge", link=True), "ankiforge", self.version)

    def test_freeze_rejects_rebuilt_candidate(self):
        self.record()
        self.tarball(release.NAMES[0], {**self.payload(release.NAMES[0]), "anki-forge.node": b"rebuilt"})
        with self.assertRaisesRegex(ValueError, "Never replace"):
            release.candidate(self.directory, "a" * 40, f"npm-v{self.version}", "123")

    def test_tag_and_stable_version_validation(self):
        self.assertEqual(release.metadata(tag=f"npm-v{self.version}"), self.version)
        for tag in ("v0.2.0", "npm-v9.0.0", "npm-v0.2.0-rc.1"):
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                release.metadata(tag=tag)

    def test_unprotected_manual_wrong_commit_and_other_run_cannot_publish(self):
        record = self.record()
        context = {"GITHUB_REPOSITORY": release.REPOSITORY, "GITHUB_EVENT_NAME": "push", "GITHUB_REF_TYPE": "tag",
                   "GITHUB_REF_NAME": record["ref"], "GITHUB_REF_PROTECTED": "true",
                   "GITHUB_SHA": record["commit"], "GITHUB_RUN_ID": record["run_id"]}
        with patch.dict(os.environ, context, clear=True):
            release.release_context(record)
            for key, value in (("GITHUB_REF_PROTECTED", "false"), ("GITHUB_EVENT_NAME", "workflow_dispatch"),
                               ("GITHUB_SHA", "b" * 40), ("GITHUB_RUN_ID", "456")):
                with self.subTest(key=key), patch.dict(os.environ, {key: value}), self.assertRaisesRegex(ValueError, key):
                    release.release_context(record)

    def test_registry_hash_collision_is_not_skipped(self):
        record = self.record()
        item = record["packages"][0]
        published = {**item["manifest"], "dist": {"integrity": "sha512-wrong", "tarball": "https://registry.npmjs.org/test"}}
        with patch.object(release, "fetch", return_value=json.dumps(published).encode()), self.assertRaisesRegex(ValueError, "different integrity"):
            release.registry_matches(item)
        published["dist"]["integrity"] = item["integrity"]
        with patch.object(release, "fetch", side_effect=[json.dumps(published).encode(), b"wrong bytes"]), self.assertRaisesRegex(ValueError, "bytes differ"):
            release.registry_matches(item)

    def test_partial_publish_skips_identical_native_and_waits_before_main(self):
        record = self.record()
        events = []
        with patch.object(release, "release_context"), patch.object(release, "registry_matches", side_effect=[True, False, False, False, False]), \
                patch.object(release, "npm", side_effect=lambda *args: events.append(args)), \
                patch.object(release, "wait_for_registry", side_effect=lambda packages, seconds: events.append(tuple(p["name"] for p in packages))):
            release.publish(self.directory, record, 0)
        self.assertEqual(len([e for e in events if e[0] == "publish"]), 4)
        self.assertEqual(events[3], tuple(release.NAMES[:-1]))
        self.assertIn("ankiforge-" + self.version + ".tgz", events[4][1])

    def test_registry_conflict_aborts_before_any_publication(self):
        record = self.record()
        with patch.object(release, "release_context"), patch.object(release, "registry_matches", side_effect=[False, ValueError("collision")]), \
                patch.object(release, "npm") as npm, self.assertRaisesRegex(ValueError, "collision"):
            release.publish(self.directory, record, 0)
        npm.assert_not_called()

    def test_main_promoted_last_and_newer_latest_not_downgraded(self):
        record = self.record()
        with patch.object(release, "release_context"), patch.object(release, "wait_for_registry"), \
                patch.object(release, "latest_versions", side_effect=[dict.fromkeys(release.NAMES), dict.fromkeys(release.NAMES, self.version)]), \
                patch.object(release, "npm") as npm:
            release.promote(record, 0)
        self.assertEqual(npm.call_args_list[-1].args, ("dist-tag", "add", f"ankiforge@{self.version}", "latest"))
        with patch.object(release, "fetch", return_value=b'{"latest":"999.0.0"}'), self.assertRaisesRegex(ValueError, "downgrade"):
            release.latest_versions(record["packages"])

    def test_registry_timeout_and_external_tarball_url(self):
        with patch.object(release, "registry_matches", return_value=False), self.assertRaises(TimeoutError):
            release.wait_for_registry([{"name": "missing"}], 0)
        with self.assertRaisesRegex(ValueError, "public npm registry"):
            release.fetch("https://example.com/package.tgz")


if __name__ == "__main__":
    unittest.main()
