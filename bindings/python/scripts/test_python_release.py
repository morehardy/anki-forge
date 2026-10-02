"""Release safety tests, independent of the native extension and registry."""
from __future__ import annotations

import copy
import io
import json
from pathlib import Path
import shutil
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import python_release as release


class ReleaseTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.dist = self.root / "dist"
        self.dist.mkdir()
        self.metadata = {"version": "0.2.0", "core_version": "0.2.0",
                         "contract_version": "2.0.0", "requires_python": ">=3.11"}
        self.package_metadata = ("Metadata-Version: 2.4\nName: anki-forge\nVersion: 0.2.0\n"
                                 "Requires-Python: >=3.11\nLicense-Expression: MIT\n\n").encode()
        for platform in ("manylinux_2_17_x86_64.manylinux2014_x86_64", "win_amd64",
                         "macosx_10_12_x86_64", "macosx_11_0_arm64"):
            self.write_wheel(platform)
        with tarfile.open(self.dist / "anki_forge-0.2.0.tar.gz", "w:gz") as archive:
            member = tarfile.TarInfo("anki_forge-0.2.0/PKG-INFO")
            member.size = len(self.package_metadata)
            archive.addfile(member, io.BytesIO(self.package_metadata))

    def write_wheel(self, platform: str, *, metadata: bytes | None = None, abi: str = "abi3") -> Path:
        path = self.dist / f"anki_forge-0.2.0-cp311-{abi}-{platform}.whl"
        with zipfile.ZipFile(path, "w") as archive:
            archive.writestr("anki_forge-0.2.0.dist-info/METADATA", metadata or self.package_metadata)
            tags = "".join(f"Tag: cp311-{abi}-{part}\n" for part in platform.split("."))
            archive.writestr("anki_forge-0.2.0.dist-info/WHEEL", "Wheel-Version: 1.0\n" + tags + "\n")
        return path

    def candidate(self) -> dict:
        return release.create_candidate(self.dist, self.root / "evidence", self.metadata,
                                        commit="a" * 40, ref="python-v0.2.0", run_id="123", run_url="https://example.test/run/123")

    def published(self, record: dict, count: int = 5) -> dict:
        return {"info": {"name": "anki-forge", "version": "0.2.0"}, "urls": [
            {"filename": entry["filename"], "size": entry["size"], "digests": {"sha256": entry["sha256"]}, "yanked": False}
            for entry in record["files"][:count]]}

    def test_candidate_records_exact_files_and_bytes(self) -> None:
        record = self.candidate()
        self.assertEqual(release.read_record(self.root / "evidence/release.json"), record)
        self.assertEqual(len((self.root / "evidence/SHA256SUMS").read_text().splitlines()), 5)
        release.verify_local(record, self.dist)

    def test_missing_or_extra_file_is_rejected(self) -> None:
        (self.dist / "extra.txt").write_text("not a distribution")
        with self.assertRaisesRegex(ValueError, "exactly"):
            self.candidate()
        (self.dist / "extra.txt").unlink()
        next(self.dist.glob("*.whl")).unlink()
        with self.assertRaisesRegex(ValueError, "exactly"):
            self.candidate()

    def test_duplicate_platform_does_not_replace_missing_platform(self) -> None:
        next(self.dist.glob("*win_amd64.whl")).unlink()
        self.write_wheel("macosx_11_0_x86_64")
        with self.assertRaisesRegex(ValueError, "platform wheel"):
            self.candidate()

    def test_wrong_abi_and_unapproved_linux_platform_are_rejected(self) -> None:
        original = next(self.dist.glob("*win_amd64.whl"))
        original.unlink()
        path = self.write_wheel("win_amd64", abi="cp311")
        with self.assertRaisesRegex(ValueError, "cp311-abi3"):
            self.candidate()
        path.unlink()
        self.write_wheel("linux_x86_64")
        with self.assertRaisesRegex(ValueError, "unsupported wheel platform"):
            self.candidate()

    def test_metadata_version_requires_python_and_license_must_match(self) -> None:
        for before, after, message in ((b"Version: 0.2.0", b"Version: 0.3.0", "wrong version"),
                                       (b">=3.11", b">=3.12", "Requires-Python"),
                                       (b"License-Expression: MIT", b"License-Expression: GPL-3.0", "MIT")):
            self.write_wheel("win_amd64", metadata=self.package_metadata.replace(before, after))
            with self.assertRaisesRegex(ValueError, message):
                self.candidate()

    def test_wheel_internal_tags_must_match_filename(self) -> None:
        path = self.write_wheel("win_amd64")
        with zipfile.ZipFile(path, "w") as archive:
            archive.writestr("anki_forge-0.2.0.dist-info/METADATA", self.package_metadata)
            archive.writestr("anki_forge-0.2.0.dist-info/WHEEL", "Tag: cp311-abi3-linux_x86_64\n\n")
        with self.assertRaisesRegex(ValueError, "WHEEL tags"):
            self.candidate()

    def test_symlink_is_not_a_distribution(self) -> None:
        path = next(self.dist.glob("*win_amd64.whl"))
        outside = self.root / path.name
        path.rename(outside)
        path.symlink_to(outside)
        with self.assertRaisesRegex(ValueError, "regular file"):
            self.candidate()

    def test_local_mutation_after_validation_is_rejected(self) -> None:
        record = self.candidate()
        self.write_wheel("win_amd64", metadata=self.package_metadata + b"changed readme\n")
        with self.assertRaisesRegex(ValueError, "local candidate changed"):
            release.verify_local(record, self.dist)

    def test_initial_upload_stages_all_five_files(self) -> None:
        record = self.candidate()
        output = self.root / "upload"
        missing = release.prepare_upload(record, self.dist, output, None, attempt=1, run_id="123")
        self.assertEqual(len(missing), 5)
        self.assertEqual(sorted(path.name for path in output.iterdir()), missing)

    def test_retry_stages_only_original_missing_files(self) -> None:
        record = self.candidate()
        published = self.published(record, 2)
        output = self.root / "retry"
        missing = release.prepare_upload(record, self.dist, output, published, attempt=2, run_id="123")
        self.assertEqual(len(missing), 3)
        self.assertEqual(sorted(path.name for path in output.iterdir()), missing)
        for path in output.iterdir():
            self.assertEqual(path.read_bytes(), (self.dist / path.name).read_bytes())

    def test_complete_upload_retry_needs_no_upload(self) -> None:
        record = self.candidate()
        self.assertEqual(release.prepare_upload(record, self.dist, self.root / "done", self.published(record),
                                                attempt=2, run_id="123"), [])

    def test_first_attempt_cannot_adopt_an_existing_version(self) -> None:
        record = self.candidate()
        with self.assertRaisesRegex(ValueError, "only a retry"):
            release.prepare_upload(record, self.dist, self.root / "upload", self.published(record, 1),
                                   attempt=1, run_id="123")

    def test_another_run_or_rehearsal_cannot_publish_candidate(self) -> None:
        record = self.candidate()
        with self.assertRaisesRegex(ValueError, "original tag run"):
            release.prepare_upload(record, self.dist, self.root / "upload", None, attempt=2, run_id="456")
        record["ref"] = "main"
        with self.assertRaisesRegex(ValueError, "original tag run"):
            release.prepare_upload(record, self.dist, self.root / "upload", None, attempt=1, run_id="123")

    def test_registry_bytes_extras_and_yanks_block_recovery(self) -> None:
        record = self.candidate()
        for key, value in (("size", 0), ("yanked", True), ("filename", "unknown.whl"),
                           ("digests", {"sha256": "0" * 64})):
            published = self.published(record)
            published["urls"][0][key] = value
            with self.assertRaises(ValueError):
                release.missing_files(record, published)

    def test_registry_identity_and_duplicate_files_are_rejected(self) -> None:
        record = self.candidate()
        published = self.published(record)
        published["info"]["version"] = "0.3.0"
        with self.assertRaisesRegex(ValueError, "identity"):
            release.missing_files(record, published)
        published = self.published(record)
        published["urls"].append(copy.deepcopy(published["urls"][0]))
        with self.assertRaisesRegex(ValueError, "unexpected registry file"):
            release.missing_files(record, published)

    def test_record_cannot_contain_traversal_filename(self) -> None:
        record = self.candidate()
        record["files"][0]["filename"] = "../outside.whl"
        path = self.root / "evidence/release.json"
        path.write_text(json.dumps(record))
        with self.assertRaisesRegex(ValueError, "unsafe"):
            release.read_record(path)

    def test_publication_waits_for_complete_set_and_rejects_permanent_partial_set(self) -> None:
        record = self.candidate()
        with patch("builtins.print"), patch.object(release, "registry_release", side_effect=[None, self.published(record)]):
            release.verify_published(record, attempts=2, delay=0)
        with patch.object(release, "registry_release", return_value=self.published(record, 1)):
            with self.assertRaisesRegex(ValueError, "incomplete"):
                release.verify_published(record, attempts=1, delay=0)

    def test_source_tag_and_versions_are_consistent(self) -> None:
        metadata = release.source_metadata(release.REPOSITORY)
        release.source_metadata(release.REPOSITORY, f"python-v{metadata['version']}")
        with self.assertRaisesRegex(ValueError, "expected tag"):
            release.source_metadata(release.REPOSITORY, "anki-forge-v0.2.0")

    def test_source_binding_core_and_contract_mismatches_are_rejected(self) -> None:
        checkout = self.root / "checkout"
        paths = ("bindings/python/pyproject.toml", "bindings/python/native/Cargo.toml",
                 "anki_forge/Cargo.toml", "bindings/python/src/anki_forge/_loader.py")
        for name in paths:
            target = checkout / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(release.REPOSITORY / name, target)
        bundle = next((release.REPOSITORY / "anki_forge/assets/contracts").glob("anki-forge-contract-bundle-*.tar.gz"))
        target = checkout / "anki_forge/assets/contracts" / bundle.name
        target.parent.mkdir(parents=True)
        target.touch()
        loader = checkout / paths[-1]
        original = loader.read_text()
        for assignment, message in (("__version__", "binding versions"),
                                    ("_CORE_API_VERSION", "core versions"),
                                    ("_CONTRACT_VERSION", "contract version")):
            with self.subTest(assignment=assignment):
                loader.write_text(original + f"\n{assignment} = '9.9.9'\n")
                with self.assertRaisesRegex(ValueError, message):
                    release.source_metadata(checkout)


if __name__ == "__main__":
    unittest.main()
