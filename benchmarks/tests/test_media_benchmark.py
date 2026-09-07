import hashlib
import io
import json
from pathlib import Path
import struct
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import wave
import zipfile
import zlib

import media_bench
import media_verify
import media_workload
import verify
import workload


class MediaFixtureTests(unittest.TestCase):
    def test_interrupted_and_failed_collectors_retain_attempts(self):
        failures = [KeyboardInterrupt(), subprocess.TimeoutExpired(["collector"], 75, output=b"partial", stderr=b"timed out"),
                    OSError("collector unavailable"), "malformed-json", "failed-export"]
        for failure in failures:
            with self.subTest(failure=repr(failure)), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                case = root / "timing-image-100-rust"
                case.mkdir()
                row = {"id": case.name, "adapter": "rust", "profile": "image", "size": 100,
                       "role": "timing", "repeat": 0}
                error_type = (type(failure) if isinstance(failure, BaseException) else
                              json.JSONDecodeError if failure == "malformed-json" else RuntimeError)

                def invoke(*args, **kwargs):
                    started = json.loads((case / "attempt.json").read_text())
                    self.assertEqual((started["id"], started["status"]), (case.name, "running"))
                    (case / "output.apkg").write_bytes(b"partial artifact")
                    if isinstance(failure, BaseException):
                        raise failure
                    stdout = "invalid JSON" if failure == "malformed-json" else json.dumps({"exit_code": 1})
                    return subprocess.CompletedProcess(args[0], 0, stdout, "collector diagnostic")

                with patch.object(media_bench.subprocess, "run", side_effect=invoke):
                    with self.assertRaises(error_type):
                        media_bench.collect_attempt(case, row, ["collector"], {})
                recorded = [json.loads(line) for line in (root / "attempts.jsonl").read_text().splitlines()]
                self.assertEqual(len(recorded), 1)
                self.assertEqual(recorded[0], json.loads((case / "attempt.json").read_text()))
                self.assertEqual(recorded[0]["status"], "cancelled" if isinstance(failure, KeyboardInterrupt) else "failed")
                self.assertTrue(recorded[0]["error"])
                self.assertEqual(recorded[0]["error_type"], error_type.__name__)
                self.assertIn("finished_utc", recorded[0])
                self.assertEqual((case / "output.apkg").read_bytes(), b"partial artifact")
                if isinstance(failure, subprocess.TimeoutExpired):
                    self.assertEqual((case / "collector.stdout.log").read_bytes(), b"partial")
                elif isinstance(failure, str):
                    self.assertIn("diagnostic", (case / "collector.stderr.log").read_text())

    def test_successful_collection_preserves_measurement(self):
        measurement = {"exit_code": 0, "signal": 0, "leftover_descendants": 0,
                       "elapsed_ns": 12345, "peak_rss_bytes": 67890}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            case = root / "timing-text-100-rust"
            case.mkdir()
            row = {"id": case.name, "adapter": "rust", "profile": "text", "size": 100,
                   "role": "timing", "repeat": 0}
            process = subprocess.CompletedProcess(["collector"], 0, json.dumps(measurement), "")
            with patch.object(media_bench.subprocess, "run", return_value=process):
                result = media_bench.collect_attempt(case, row, ["collector"], {})
            self.assertEqual(result["status"], "success")
            self.assertEqual(result["measurement"], measurement)
            self.assertEqual(json.loads((root / "attempts.jsonl").read_text()), result)

    def test_only_requested_tiers_and_real_decodable_media(self):
        self.assertEqual(workload.SIZES, (100, 200, 500, 1000))
        first, second = media_workload.png(1), media_workload.png(2)
        self.assertNotEqual(first, second)
        self.assertEqual(first[:8], b"\x89PNG\r\n\x1a\n")
        offset = 8
        while offset < len(first):
            size = struct.unpack(">I", first[offset:offset + 4])[0]
            kind = first[offset + 4:offset + 8]
            payload = first[offset + 8:offset + 8 + size]
            crc = struct.unpack(">I", first[offset + 8 + size:offset + 12 + size])[0]
            self.assertEqual(crc, zlib.crc32(kind + payload))
            if kind == b"IDAT":
                self.assertEqual(len(zlib.decompress(payload)), 148 * (144 * 3 + 1))
            offset += 12 + size
        audio = media_workload.wav(3)
        with wave.open(io.BytesIO(audio)) as reader:
            self.assertEqual((reader.getnchannels(), reader.getframerate(), reader.getnframes()), (1, 16000, 16000))
        self.assertEqual(audio, media_workload.wav(3))
        self.assertNotEqual(audio, media_workload.wav(4))

    def test_media_references_are_checked_without_allowing_other_markup(self):
        expected = {"media": [{"id": "image", "kind": "image", "filename": "one.png"}],
                    "notes": [{"front": "question <literal>", "back": "answer", "front_media": ["image"]}]}
        fields = ['question &lt;literal&gt;\n<img src="one.png">', "answer"]
        self.assertEqual(verify.decode_fields(fields, expected), ("question <literal>", "answer"))
        for bad in ['question &lt;literal&gt;\n<img src="two.png">', 'question <literal>\n<img src="one.png">']:
            with self.assertRaises(verify.InvalidArtifact):
                verify.decode_fields([bad, "answer"], expected)

    def test_missing_extra_or_corrupt_media_are_rejected(self):
        expected = {"media": [{"filename": "one.wav", "sha256": hashlib.sha256(b"audio").hexdigest()}]}
        with tempfile.TemporaryDirectory() as directory:
            for payload, extra in [(b"audio", False), (b"corrupt", False), (b"audio", True)]:
                path = Path(directory) / "test.apkg"
                with zipfile.ZipFile(path, "w") as archive:
                    archive.writestr("collection.anki2", b"database")
                    archive.writestr("media", json.dumps({"0": "one.wav"}))
                    archive.writestr("0", payload)
                    if extra:
                        archive.writestr("unlisted", b"bad")
                if payload == b"audio" and not extra:
                    self.assertEqual(media_verify.check_media(path, expected), 1)
                else:
                    with self.assertRaises(verify.InvalidArtifact):
                        media_verify.check_media(path, expected)

    def test_timing_and_rss_passes_remain_separate(self):
        rows = []
        for adapter, duration in [("rust", 60), ("genanki", 100)]:
            for repeat in range(3):
                rows += [{"profile": "fixture", "size": 100, "adapter": adapter, "role": "timing",
                          "measurement": {"elapsed_ns": duration * 1_000_000, "peak_rss_bytes": 999999999}},
                         {"profile": "fixture", "size": 100, "adapter": adapter, "role": "rss",
                          "measurement": {"elapsed_ns": 999999999, "peak_rss_bytes": 32 * 2**20}}]
        result = media_bench.summarize(rows)[0]
        self.assertEqual(result["rust"]["median_ms"], 60)
        self.assertEqual(result["rust"]["rss_mib"], 32)
        self.assertTrue(result["target_passed"])


if __name__ == "__main__":
    unittest.main()
