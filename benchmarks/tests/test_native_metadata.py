"""Known native metadata must not weaken archive payload accounting."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import zstandard
import media_verify
import verify


def envelope():
    return {"format_version": "ankiforge-identity-v1", "collection_blake3": "a" * 64,
            "identity_blake3": "b" * 64, "media": {},
            "identity": {"namespace": "benchmark", "models": {}, "notes": {}}}


class NativeMetadataTests(unittest.TestCase):
    def write(self, path, identity, extra=None):
        compress = zstandard.ZstdCompressor().compress
        with zipfile.ZipFile(path, "w") as archive:
            archive.writestr("meta", b"\x08\x03")
            archive.writestr("collection.anki2", b"placeholder")
            archive.writestr("collection.anki21b", compress(b"SQLite format 3\x00test"))
            archive.writestr("media", compress(b""))
            archive.writestr(verify.NATIVE_IDENTITY, identity)
            if extra:
                archive.writestr(extra, b"unaccounted")

    def test_known_metadata_is_recorded_without_claiming_identity_validation(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "native.apkg"
            self.write(path, json.dumps(envelope()))
            _, report = verify.read_package(path)
            self.assertEqual(report["native_identity"]["format_version"], "ankiforge-identity-v1")
            self.assertIn("not verified", report["native_identity"]["scope"])
            self.assertEqual(len(report["native_identity"]["sha256"]), 64)
            self.assertEqual(media_verify.check_media(path, {"media": []}), 0)

    def test_unknown_payload_still_fails_both_verifiers(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "native.apkg"
            for extra in ("unlisted", "0", "ankiforge-other.json"):
                self.write(path, json.dumps(envelope()), extra)
                for check in (lambda: verify.read_package(path),
                              lambda: media_verify.check_media(path, {"media": []})):
                    with self.subTest(extra=extra), self.assertRaises(verify.InvalidArtifact):
                        check()

    def test_malformed_duplicate_unknown_version_and_oversized_metadata_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "native.apkg"
            invalid = ["{}", "[]", '{"format_version":1,"format_version":2}',
                       json.dumps({**envelope(), "format_version": "future"}),
                       json.dumps({**envelope(), "identity_blake3": "bad"})]
            for value in invalid:
                self.write(path, value)
                with self.assertRaises(verify.InvalidArtifact):
                    verify.read_package(path)
                with self.assertRaises(verify.InvalidArtifact):
                    media_verify.check_media(path, {"media": []})
            self.write(path, json.dumps(envelope()))
            with patch.object(verify, "MAX_IDENTITY_BYTES", 10):
                with self.assertRaisesRegex(verify.InvalidArtifact, "budget"):
                    verify.read_package(path)
                with self.assertRaisesRegex(verify.InvalidArtifact, "budget"):
                    media_verify.check_media(path, {"media": []})


if __name__ == "__main__":
    unittest.main()
