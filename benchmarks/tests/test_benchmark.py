import copy
import json
import os
import signal
import sqlite3
import subprocess
import sys
import tempfile
import time
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import bench
import report
import workload
import verify


class WorkloadTests(unittest.TestCase):
    def test_frozen_corpus_and_prefixes(self):
        evidence = workload.generate()
        largest = json.loads((workload.ROOT / "inputs/basic-10000.json").read_text())
        for size in workload.SIZES:
            raw = (workload.ROOT / f"inputs/basic-{size}.json").read_bytes()
            doc = json.loads(raw)
            self.assertEqual(doc["notes"], largest["notes"][:size])
            self.assertEqual(evidence["cases"][str(size)]["categories"],
                             {"english": size // 2, "mixed": size * 4 // 10, "escaping": size // 10})
            self.assertEqual(len({n["front"] for n in doc["notes"]}), size)
            for note in doc["notes"]:
                for field, bounds in (("front", (40, 100)), ("back", (120, 300))):
                    self.assertTrue(bounds[0] <= len(note[field]) <= bounds[1])
                    self.assertFalse(any(ord(c) < 32 or 127 <= ord(c) <= 159 for c in note[field]))
        sample = largest["notes"][18]
        self.assertIn("<b>literal</b> &amp;", sample["front"])
        self.assertEqual(verify.literal("&lt;b&gt;literal&lt;/b&gt; &amp;amp; &quot;x&quot; &#39;y&#39;"),
                         '<b>literal</b> &amp; "x" \'y\'')

    def test_balanced_adjacent_schedule(self):
        schedule = bench.schedule()
        self.assertEqual(len(schedule), 168)
        self.assertEqual(schedule, bench.schedule())
        for role, count in (("timing_warmup", 3), ("timing", 10), ("memory_warmup", 3), ("memory", 5)):
            rows = [r for r in schedule if r["role"] == role]
            for size in workload.SIZES:
                cell = [r for r in rows if r["size"] == size]
                self.assertEqual(len(cell), count * 2)
                first = [r["adapter"] for r in cell[::2]]
                self.assertLessEqual(abs(first.count("rust") - first.count("genanki")), 1)
            for left, right in zip(rows[::2], rows[1::2]):
                self.assertEqual((left["size"], left["round"]), (right["size"], right["round"]))
                self.assertNotEqual(left["adapter"], right["adapter"])


class BuildIdentityTests(unittest.TestCase):
    def test_allocator_metadata_must_match_the_adapter_feature_and_pinned_version(self):
        system = {"features": "default", "allocator": "system", "allocator_version": None, "adapter_features": []}
        mimalloc = {"features": "default", "allocator": "mimalloc", "allocator_version": "0.1.52", "adapter_features": ["mimalloc"]}
        for metadata in (system, mimalloc):
            self.assertEqual(bench.rust_adapter_configuration(metadata)["allocator"], metadata["allocator"])
        for metadata in ({}, {**system, "adapter_features": ["mimalloc"]},
                         {**mimalloc, "allocator_version": "0.1.51"},
                         {**mimalloc, "adapter_features": ["mimalloc", "internal-tools"]},
                         {**system, "features": "internal-tools"}):
            with self.assertRaisesRegex(RuntimeError, "allocator/feature metadata"):
                bench.rust_adapter_configuration(metadata)

    def test_prepare_defaults_to_system_and_explicitly_selects_adapter_features(self):
        for allocator in ("system", "mimalloc"):
            with self.subTest(allocator=allocator), tempfile.TemporaryDirectory() as directory:
                suite = Path(directory)
                adapter = {"id": "rust", "command": [str(suite / "binary")]}
                with patch.object(bench, "SUITE", suite), patch.object(bench, "registry", return_value=[adapter]), \
                     patch.object(bench, "run_build") as builds, patch.object(bench.subprocess, "run"), \
                     patch.object(bench.workload, "generate"):
                    if allocator == "system":
                        bench.prepare("python3.11", False)
                    else:
                        bench.prepare("python3.11", False, allocator)
                argv = builds.call_args_list[0].args[0]
                self.assertIn("--no-default-features", argv)
                if allocator == "mimalloc":
                    self.assertEqual(argv[-2:], ["--features", "mimalloc"])
                else:
                    self.assertNotIn("--features", argv)
        with patch.object(sys, "argv", ["bench.py", "prepare", "--rust-allocator", "mimalloc"]), \
             patch.object(bench, "prepare") as prepare:
            self.assertEqual(bench.main(), 0)
            prepare.assert_called_once_with("python3.11", False, "mimalloc")

    def test_profile_rust_and_target_native_overrides_are_recorded_without_secrets(self):
        expected = {"CARGO_PROFILE_RELEASE_LTO": "thin", "CARGO_PROFILE_RELEASE_CODEGEN_UNITS": "1",
                    "RUSTFLAGS": "", "CARGO_ENCODED_RUSTFLAGS": "-C\x1ftarget-cpu=native",
                    "CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER": "clang",
                    "CC": "clang", "CFLAGS": "-O3 -march=native", "HOST_CFLAGS": "-O2",
                    "CC_aarch64-apple-darwin": "target-clang", "CFLAGS_aarch64_apple_darwin": "-O1",
                    "CXXFLAGS": "-g", "SDKROOT": "/sdk", "RUSTUP_TOOLCHAIN": "1.92.0"}
        environ = {**expected, "API_TOKEN": "secret", "CARGO_REGISTRIES_PRIVATE_TOKEN": "secret",
                   "AWS_SECRET_ACCESS_KEY": "secret", "UNRELATED": "ignored"}
        self.assertEqual(bench.build_environment(environ), expected)
        self.assertEqual(bench.build_environment({}), {})

    def test_build_record_uses_the_build_environment_and_output_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            binary, records = root / "binary", root / "records.json"
            args = ["cargo", "build", "--release"]
            environ = {"PATH": "/toolchain", "CARGO_PROFILE_RELEASE_LTO": "thin", "CFLAGS": "-O3"}

            def build(*call_args, **kwargs):
                self.assertEqual(call_args, (args,))
                self.assertEqual(kwargs["env"], environ)
                binary.write_bytes(b"built-binary")

            with patch.object(bench, "BUILD_RECORDS", records), patch.dict(os.environ, environ, clear=True), \
                 patch.object(bench.subprocess, "run", side_effect=build):
                bench.run_build(args, [binary])
                os.environ["CARGO_PROFILE_RELEASE_LTO"] = "off"
                provenance = bench.build_provenance([{"id": "rust", "command": [str(binary)]}])[str(binary)]
            self.assertEqual(provenance["status"], "verified")
            self.assertEqual(provenance["record"]["environment"], environ)
            self.assertEqual(provenance["record"]["command"], args)
            self.assertEqual(provenance["observed_sha256"], verify.sha256(binary))

    def test_missing_or_stale_record_does_not_claim_a_verified_build(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            binary, records = root / "binary", root / "records.json"
            binary.write_bytes(b"original")
            adapters = [{"id": "rust", "command": [str(binary)]}]
            with patch.object(bench, "BUILD_RECORDS", records):
                self.assertEqual(bench.build_provenance(adapters)[str(binary)]["status"], "unrecorded")
                bench.save(records, {str(binary): {"executable_sha256": verify.sha256(binary), "environment": {}}})
                binary.write_bytes(b"rebuilt-with-unknown-flags")
                self.assertEqual(bench.build_provenance(adapters)[str(binary)]["status"], "stale")
                binary.unlink()
                self.assertEqual(bench.build_provenance(adapters)[str(binary)]["status"], "stale")

    def test_failed_build_cannot_replace_a_previous_build_record(self):
        with tempfile.TemporaryDirectory() as directory:
            records = Path(directory) / "records.json"
            previous = {"old": {"executable_sha256": "old-hash"}}
            bench.save(records, previous)
            with patch.object(bench, "BUILD_RECORDS", records), \
                 patch.object(bench.subprocess, "run", side_effect=subprocess.CalledProcessError(1, ["cargo"])):
                with self.assertRaises(subprocess.CalledProcessError):
                    bench.run_build(["cargo", "build"], [Path(directory) / "missing-binary"])
            self.assertEqual(json.loads(records.read_text()), previous)


class CollectorTests(unittest.TestCase):
    def test_diagnostic_logs_are_open_before_adapter_launch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            out, err, marker = root / "out", root / "err", root / "launched"
            os.mkfifo(out)
            source = "from pathlib import Path; import sys; Path(sys.argv[1]).touch()"
            process = subprocess.Popen([str(bench.COLLECTOR), "3", str(out), str(err),
                                        sys.executable, "-c", source, str(marker)], stdout=subprocess.PIPE)
            reader = None
            try:
                # Hold the diagnostic open while the collector is free to schedule threads.
                deadline = time.monotonic() + .3
                while not marker.exists() and time.monotonic() < deadline:
                    time.sleep(.01)
                launched_before_log_open = marker.exists()
                reader = os.open(out, os.O_RDONLY | os.O_NONBLOCK)
                measured = json.loads(process.communicate(timeout=5)[0])
                self.assertFalse(launched_before_log_open)
                self.assertEqual(measured["exit_code"], 0)
                self.assertTrue(marker.is_file())
            finally:
                if process.poll() is None:
                    process.kill()
                    process.communicate()
                if reader is not None:
                    os.close(reader)

    def test_unavailable_diagnostic_log_prevents_launch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            marker = root / "launched"
            source = "from pathlib import Path; import sys; Path(sys.argv[1]).touch()"
            for failed_stream in ("stdout", "stderr"):
                with self.subTest(failed_stream=failed_stream):
                    out = root if failed_stream == "stdout" else root / "out"
                    err = root if failed_stream == "stderr" else root / "err"
                    p = subprocess.run([str(bench.COLLECTOR), "3", str(out), str(err),
                                        sys.executable, "-c", source, str(marker)], capture_output=True, timeout=5)
                    self.assertNotEqual(p.returncode, 0)
                    self.assertFalse(marker.exists())

    def collect(self, source, timeout=3):
        with tempfile.TemporaryDirectory() as directory:
            out, err = Path(directory) / "out", Path(directory) / "err"
            raw = subprocess.check_output([str(bench.COLLECTOR), str(timeout), str(out), str(err), sys.executable, "-c", source], timeout=timeout + 10)
            return json.loads(raw), out.stat().st_size, err.stat().st_size

    def test_launch_exit_and_bounded_drain(self):
        value, out, err = self.collect("import os; os.write(1,b'x'*1000000); os.write(2,b'y'*1000000)")
        self.assertEqual(value["exit_code"], 0)
        self.assertEqual((out, err), (65536, 65536))
        self.assertEqual(value["stdout_bytes"], 1000000)
        self.assertGreater(value["elapsed_ns"], 0)
        self.assertTrue(value["reaped"])
        self.assertGreater(value["peak_rss_raw"], 0)
        multiplier = 1 if sys.platform == "darwin" else 1024
        self.assertEqual(value["peak_rss_bytes"], value["peak_rss_raw"] * multiplier)

    def test_os_peak_is_not_previous_child_or_large_parent(self):
        large, _, _ = self.collect("x=bytearray(192*1024*1024); print(len(x))")
        small, _, _ = self.collect("pass")
        self.assertGreater(large["peak_rss_bytes"], small["peak_rss_bytes"] + 100 * 1024 ** 2)
        parent_allocation = bytearray(192 * 1024 ** 2)
        small_with_large_parent, _, _ = self.collect("pass")
        self.assertLess(small_with_large_parent["peak_rss_bytes"], 80 * 1024 ** 2)
        self.assertEqual(len(parent_allocation), 192 * 1024 ** 2)

    def test_timeout_and_leftover_descendants(self):
        value, _, _ = self.collect("import time; time.sleep(20)", timeout=1)
        self.assertEqual(value["interrupted_signal"], signal.SIGALRM)
        self.assertNotEqual(value["exit_code"], 0)
        value, _, _ = self.collect("import subprocess,sys; subprocess.Popen([sys.executable,'-c','import time; time.sleep(20)'])")
        self.assertTrue(value["leftover_descendants"])

    def test_nonzero_and_missing_output_are_not_success(self):
        with tempfile.TemporaryDirectory() as directory:
            run = Path(directory)
            adapter = [{"id": "fake", "command": [sys.executable, "-c", "raise SystemExit(7)"]}]
            item = {"id": "exit", "role": "timing", "size": 200, "adapter": "fake", "round": 1}
            self.assertEqual(bench.run_attempt(item, run, adapter)["status"], "adapter_failure")
            adapter[0]["command"] = [sys.executable, "-c", "pass"]
            item["id"] = "missing"
            self.assertEqual(bench.run_attempt(item, run, adapter)["status"], "missing_output")
            with self.assertRaises(FileExistsError):
                bench.run_attempt(item, run, adapter)


class PhysicalVerifierTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        workload.generate()
        cls.doc = json.loads((workload.ROOT / "inputs/basic-200.json").read_text())
        cls.directory = tempfile.TemporaryDirectory()
        cls.path = Path(cls.directory.name) / "baseline.apkg"
        subprocess.run([sys.executable, str(workload.ROOT / "adapters/genanki/export.py"),
                        str(workload.ROOT / "inputs/basic-200.json"), str(cls.path)], check=True,
                       env={**os.environ, "TMPDIR": cls.directory.name})
        cls.raw, _ = verify.read_package(cls.path)

    @classmethod
    def tearDownClass(cls):
        cls.directory.cleanup()

    def database(self):
        db = sqlite3.connect(":memory:")
        db.deserialize(self.raw)
        self.addCleanup(db.close)
        return db

    def test_valid_legacy_with_whitespace_tags(self):
        db = self.database()
        self.assertEqual(db.execute("SELECT tags FROM notes LIMIT 1").fetchone()[0], "  ")
        physical, _, _, semantic = verify.check_rows(db, self.doc)
        self.assertEqual(physical["notes"], 200)
        self.assertEqual(semantic["status"], "passed")

    def test_collection_row_cardinality_preserves_deferred_verification(self):
        for count in (0, 2):
            with self.subTest(collection_rows=count), tempfile.TemporaryDirectory() as directory:
                run = Path(directory)
                db = self.database()
                if count == 0:
                    db.execute("DELETE FROM col")
                else:
                    row = list(db.execute("SELECT * FROM col").fetchone())
                    row[0] += 1
                    db.execute("INSERT INTO col VALUES (" + ",".join("?" for _ in row) + ")", row)
                self.assertEqual(db.execute("SELECT COUNT(*) FROM col").fetchone()[0], count)
                db.commit()
                with zipfile.ZipFile(run / "malformed.apkg", "w") as archive:
                    archive.writestr("collection.anki2", db.serialize())
                    archive.writestr("media", "{}")
                (run / "valid.apkg").write_bytes(self.path.read_bytes())
                records = [{"id": name, "status": "success", "size": 200, "artifact": f"{name}.apkg"}
                           for name in ("malformed", "valid")]
                results = bench.verify_records(run, records)
                self.assertEqual(results["malformed"]["status"], "invalid_artifact")
                self.assertIn("exactly one collection row", results["malformed"]["reason"])
                self.assertEqual(results["valid"]["status"], "passed")
                bench.save(run / "verification.json", results)
                self.assertEqual(json.loads((run / "verification.json").read_text()), results)

    def test_raw_multiplicity_and_reference_counterexamples(self):
        mutations = [
            "UPDATE cards SET nid=(SELECT min(id) FROM notes) WHERE id=(SELECT max(id) FROM cards)",
            "UPDATE cards SET nid=-1 WHERE id=(SELECT max(id) FROM cards)",
            "UPDATE cards SET did=-1 WHERE id=(SELECT max(id) FROM cards)",
            "UPDATE notes SET mid=-1 WHERE id=(SELECT max(id) FROM notes)",
            "UPDATE notes SET flds=flds || char(31) || 'extra' WHERE id=(SELECT min(id) FROM notes)",
            "UPDATE notes SET guid=(SELECT guid FROM notes ORDER BY id LIMIT 1) WHERE id=(SELECT max(id) FROM notes)",
            "UPDATE notes SET guid='' WHERE id=(SELECT max(id) FROM notes)",
            "UPDATE notes SET flds=(SELECT flds FROM notes ORDER BY id LIMIT 1) WHERE id=(SELECT max(id) FROM notes)",
            "UPDATE cards SET ord=1 WHERE id=(SELECT max(id) FROM cards)",
            "UPDATE notes SET flds='<b>changed</b>' || char(31) || 'answer' WHERE id=(SELECT max(id) FROM notes)",
        ]
        for sql in mutations:
            with self.subTest(sql=sql):
                db = self.database()
                db.execute(sql)
                with self.assertRaises(verify.InvalidArtifact):
                    verify.check_rows(db, self.doc)

    def test_malformed_legacy_metadata_is_invalid_evidence(self):
        for field, mutation in (("models", "alias"), ("decks", "alias"),
                                ("models", "missing"), ("decks", "missing"),
                                ("models", "wrong_type"), ("models", "nonnumeric")):
            with self.subTest(field=field, mutation=mutation), tempfile.TemporaryDirectory() as directory:
                db = self.database()
                metadata = json.loads(db.execute(f"SELECT {field} FROM col").fetchone()[0])
                used = db.execute("SELECT mid FROM notes LIMIT 1" if field == "models" else
                                  "SELECT did FROM cards LIMIT 1").fetchone()[0]
                key = str(used)
                if mutation == "alias":
                    metadata["0" + key] = metadata.pop(key)
                elif mutation == "missing":
                    del metadata[key]["flds" if field == "models" else "name"]
                elif mutation == "wrong_type":
                    metadata[key]["flds"] = None
                else:
                    metadata["not-an-id"] = metadata.pop(key)
                db.execute(f"UPDATE col SET {field}=?", (json.dumps(metadata),))
                db.commit()
                run = Path(directory)
                with zipfile.ZipFile(run / "malformed.apkg", "w") as archive:
                    archive.writestr("collection.anki2", db.serialize())
                    archive.writestr("media", "{}")
                (run / "valid.apkg").write_bytes(self.path.read_bytes())
                records = [{"id": name, "status": "success", "size": 200, "artifact": f"{name}.apkg"}
                           for name in ("malformed", "valid")]
                results = bench.verify_records(run, records)
                self.assertEqual(results["malformed"]["status"], "invalid_artifact")
                self.assertEqual(results["valid"]["status"], "passed")

    def test_wrong_template_and_original_corrupt_database(self):
        db = self.database()
        models = json.loads(db.execute("SELECT models FROM col").fetchone()[0])
        next(iter(models.values()))["tmpls"][0]["qfmt"] = "{{Back}}"
        db.execute("UPDATE col SET models=?", (json.dumps(models),))
        with self.assertRaisesRegex(verify.InvalidArtifact, "templates"):
            verify.check_rows(db, self.doc)
        with self.assertRaises(verify.InvalidArtifact):
            verify.literal("<b>literal</b>")
        self.assertNotEqual(verify.literal("&amp;amp;"), "&")


class EvidenceTests(unittest.TestCase):
    def test_missing_deferred_artifact_preserves_other_checks_and_summary(self):
        workload.generate()
        rows, checks, anki = self.records()
        with tempfile.TemporaryDirectory() as directory:
            run = Path(directory)
            missing = {**rows[0], "artifact": "missing.apkg"}
            valid = {**rows[1], "artifact": "valid.apkg"}
            subprocess.run([sys.executable, str(workload.ROOT / "adapters/genanki/export.py"),
                            str(workload.ROOT / "inputs/basic-10000.json"), str(run / "valid.apkg")], check=True)
            results = bench.verify_records(run, [missing, valid])
            self.assertEqual(results[missing["id"]]["status"], "verification_unavailable")
            self.assertNotIn("artifact_bytes", results[missing["id"]])
            self.assertEqual(results[valid["id"]]["status"], "passed")
            checks.update(results)
            bench.save(run / "verification.json", checks)
            cell = report.cell_summary(rows, checks, anki, 10000, "rust")
            self.assertEqual(cell["status"], "unverified")
            self.assertIsNone(cell["apkg_bytes"])

    def test_unreadable_artifact_metadata_is_failed_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "output.apkg"
            path.write_bytes(b"retained artifact")
            for operation in (patch.object(verify, "sha256", side_effect=PermissionError("unreadable")),
                              patch.object(Path, "stat", side_effect=OSError("stat unavailable"))):
                with operation:
                    result = verify.verify_artifact(path, {}, None)
                self.assertEqual(result["status"], "verification_unavailable")
                self.assertIn("reason", result)

    def test_oracle_timeout_is_persisted_and_next_cell_continues(self):
        with tempfile.TemporaryDirectory() as directory:
            run = Path(directory)
            oracle = run / "oracle"
            oracle.write_text("pinned oracle executable")
            frozen = verify.sha256(oracle)
            rows = [{"id": f"timing-200-{adapter}-01", "role": "timing", "round": 1,
                     "size": 200, "adapter": adapter,
                     "artifact": f"artifacts/timing-200-{adapter}-01/output.apkg"}
                    for adapter in ("rust", "genanki")]
            checks = {}
            for row in rows:
                artifact = run / row["artifact"]
                artifact.parent.mkdir(parents=True)
                artifact.write_bytes(b"original measured bytes")
                checks[row["id"]] = {"status": "passed", "artifact_sha256": verify.sha256(artifact),
                                     "physical": {"logical_sha256": "logical"}}
                bench.append(run / "attempts.jsonl", row)
            bench.save(run / "verification.json", checks)
            bench.save(run / "manifest.json", {"oracle": {"available_before_measurement": True},
                       "identity_before": {"executables": {str(oracle): frozen}, "upstream_revision": "pinned"}})
            def invoke(argv, **kwargs):
                self.assertEqual(kwargs["timeout"], 120)
                if "rust" in argv[2]:
                    raise subprocess.TimeoutExpired(argv, 120, output=b"partial output", stderr=b"partial error")
                bench.save(Path(argv[3]), {"status": "passed", "notes": 200, "cards": 200})
                return subprocess.CompletedProcess(argv, 0, stdout=b"", stderr=b"")
            with patch.object(bench, "ORACLE", oracle), patch.object(subprocess, "run", side_effect=invoke) as launch:
                evidence = bench.complete_oracle(run)
            self.assertEqual(launch.call_count, 2)
            failure = evidence[rows[0]["id"]]
            self.assertEqual(failure["status"], "oracle_failed")
            self.assertEqual(failure["failure_kind"], "timeout")
            self.assertEqual(failure["timeout_seconds"], 120)
            self.assertEqual(failure["stderr"], "partial error")
            self.assertEqual(evidence[rows[1]["id"]]["status"], "passed")
            self.assertEqual(json.loads((run / "anki.json").read_text()), evidence)
            bench.cleanup(run)
            self.assertTrue((run / rows[0]["artifact"]).is_file())
            self.assertFalse((run / rows[1]["artifact"]).exists())

    def test_oracle_identity_io_failures_preserve_remaining_evidence(self):
        for failed_path in ("oracle", "artifact", "after_import"):
            with self.subTest(failed_path=failed_path), tempfile.TemporaryDirectory() as directory:
                run = Path(directory)
                oracle = run / "oracle"
                oracle.write_bytes(b"pinned executable")
                frozen = verify.sha256(oracle)
                rows, checks = [], {}
                for adapter in ("rust", "genanki"):
                    key = f"timing-200-{adapter}-01"
                    artifact = run / "artifacts" / key / "output.apkg"
                    artifact.parent.mkdir(parents=True)
                    artifact.write_bytes(b"original artifact")
                    rows.append({"id": key, "role": "timing", "round": 1, "size": 200,
                                 "adapter": adapter, "artifact": str(artifact.relative_to(run))})
                    checks[key] = {"status": "passed", "artifact_sha256": verify.sha256(artifact),
                                   "physical": {"logical_sha256": "logical"}}
                    bench.append(run / "attempts.jsonl", rows[-1])
                bench.save(run / "verification.json", checks)
                bench.save(run / "manifest.json", {"oracle": {"available_before_measurement": True},
                           "identity_before": {"executables": {str(oracle): frozen}, "upstream_revision": "pinned"}})
                original_sha = verify.sha256
                artifact_reads = 0
                def sha(path):
                    nonlocal artifact_reads
                    if path == oracle and failed_path == "oracle":
                        raise PermissionError("oracle unreadable")
                    if path == run / rows[0]["artifact"]:
                        artifact_reads += 1
                        if failed_path == "artifact" or (failed_path == "after_import" and artifact_reads == 2):
                            raise FileNotFoundError("artifact disappeared after is_file")
                    return original_sha(path)
                def invoke(argv, **kwargs):
                    bench.save(Path(argv[3]), {"status": "passed", "notes": 200, "cards": 200})
                    return subprocess.CompletedProcess(argv, 0, stdout=b"", stderr=b"")
                with patch.object(bench, "ORACLE", oracle), patch.object(verify, "sha256", side_effect=sha), \
                     patch.object(subprocess, "run", side_effect=invoke):
                    evidence = bench.complete_oracle(run)
                    if failed_path == "oracle":
                        with patch.object(bench, "source_paths", return_value=[]), \
                             patch.object(bench.importlib.metadata, "distributions", return_value=[]), \
                             patch.object(workload, "SIZES", ()), patch.object(bench, "command", return_value=""):
                            snapshot = bench.identity_snapshot([])
                        self.assertIn("oracle unreadable", snapshot["executables"][str(oracle)])
                expected = {"oracle": "missing_oracle", "artifact": "missing_or_changed_artifact",
                            "after_import": "changed_during_oracle"}[failed_path]
                self.assertEqual(evidence[rows[0]["id"]]["status"], expected)
                self.assertIn("reason", evidence[rows[0]["id"]])
                self.assertEqual(evidence[rows[1]["id"]]["status"], "missing_oracle" if failed_path == "oracle" else "passed")
                self.assertEqual(json.loads((run / "anki.json").read_text()), evidence)
                bench.cleanup(run)
                self.assertTrue((run / rows[0]["artifact"]).is_file())

    def test_captured_adapter_version_is_rendered(self):
        adapters = [{"id": "rust", "command": ["rust-adapter"]},
                    {"id": "genanki", "command": ["genanki-adapter"]}]
        bundle = next((bench.REPO / "anki_forge/assets").rglob("anki-forge-contract-bundle-*.tar.gz")).name.removeprefix(
            "anki-forge-contract-bundle-").removesuffix(".tar.gz")
        reported_bundle = bundle
        allocator = "system"
        feature_commands = []
        def command(argv, **kwargs):
            if argv == ["rust-adapter", "--metadata"]:
                return json.dumps({"crate_version": "0.2.3", "bundle_version": reported_bundle,
                                   "features": "default", "allocator": allocator,
                                   "allocator_version": "0.1.52" if allocator == "mimalloc" else None,
                                   "adapter_features": ["mimalloc"] if allocator == "mimalloc" else []})
            if argv == ["genanki-adapter", "--metadata"]:
                return json.dumps({"genanki": "0.13.1", "architecture": bench.platform.machine()})
            if argv[:2] == ["cargo", "tree"]:
                feature_commands.append(argv)
            return ""
        identity = {"source_files": {}, "upstream_revision": "pinned", "upstream_dirty": ""}
        with patch.object(bench, "command", side_effect=command), \
             patch.object(bench, "identity_snapshot", return_value=identity), patch.object(bench, "host_state", return_value={}):
            m = bench.manifest(adapters, workload.generate(), "full", [], 1024)
            allocator = "mimalloc"
            mi_manifest = bench.manifest(adapters, m["fixture_evidence"], "full", [], 1024)
            reported_bundle = "stale-bundle"
            with self.assertRaisesRegex(RuntimeError, "bundle.*mismatch"):
                bench.manifest(adapters, {}, "full", [], 1024)
        self.assertEqual(feature_commands[0][-1], "--no-default-features")
        self.assertEqual(feature_commands[1][-3:], ["--no-default-features", "--features", "mimalloc"])
        legacy = copy.deepcopy(m)
        legacy["adapter_metadata"]["rust"] = {"crate_version": "0.2.3", "bundle_version": bundle}
        for captured, label in ((m, "system"), (mi_manifest, "mimalloc 0.1.52"), (legacy, "unrecorded")):
            with self.subTest(allocator=label), tempfile.TemporaryDirectory() as directory:
                run = Path(directory)
                captured.update(run_id="version-test", identity_unchanged=True)
                bench.save(run / "manifest.json", captured)
                for filename in ("verification.json", "anki.json"):
                    bench.save(run / filename, {})
                (run / "attempts.jsonl").write_text("")
                with patch.object(report, "plot") as plot:
                    summary = report.render(run)
                rendered = (run / "report.md").read_text()
                self.assertIn("anki-forge 0.2.3", rendered)
                self.assertIn(f"**Rust allocator: {label}.**", rendered.split("| Notes/cards")[0])
                self.assertEqual(report.rust_allocator_label(plot.call_args.args[0]["rust_configuration"]), label)
                if label.startswith("mimalloc"):
                    self.assertIn("not the default system-allocator result", rendered)
                self.assertEqual(summary["rust_configuration"], report.rust_configuration(captured))
        self.assertEqual(m["toolchain"]["crate_version"], "0.2.3")
        self.assertEqual(m["toolchain"]["bundle_version"], bundle)

    def test_provenance_includes_untracked_runtime_sources_and_embedded_bundle(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            included = ["anki_forge/src/new.rs", "anki_forge/assets/contracts/new.tar.gz",
                        "contracts/versioning/changes/new.yaml", "contract_tools/src/new.rs",
                        "contracts/fixtures/inputs/new.json"]
            excluded = ["anki_forge/target/cache", "contracts/artifacts/package.apkg",
                        "benchmarks/.work/run/output.apkg"]
            for name in included + excluded:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("new untracked file")
            with patch.object(bench, "REPO", root), patch.object(bench, "SUITE", root / "benchmarks"), \
                 patch.object(bench, "command", return_value="deleted-tracked-file.rs\0"):
                paths = set(bench.source_paths())
            self.assertTrue({root / name for name in included} <= paths)
            self.assertFalse({root / name for name in excluded} & paths)
            self.assertIn(root / "deleted-tracked-file.rs", paths)

    def records(self, size=10000, adapter="rust"):
        records, verification, anki = [], {}, {}
        for role, count in (("timing", 10), ("memory", 5)):
            for i in range(count):
                key = f"{role}-{size}-{adapter}-{i + 1:02d}"
                records.append({"id": key, "role": role, "round": i + 1, "size": size, "adapter": adapter,
                    "status": "success", "measurement": {"elapsed_ns": (i + 1) * 1000000, "peak_rss_bytes": (i + 1) * 1000000},
                    "memory": {"metric": bench.METRIC, "scope": "single_process", "status": "available"}})
                verification[key] = {"status": "passed", "artifact_sha256": key, "artifact_bytes": 100 if role == "timing" else 99999}
        selected = f"timing-{size}-{adapter}-01"
        anki[selected] = {"status": "passed", "artifact_sha256": selected}
        return records, verification, anki

    def test_hand_calculated_statistics_and_timing_only_sizes(self):
        self.assertEqual(report.stats([1, 2, 3, 4]), {"n": 4, "median": 2.5, "q1": 1.75, "q3": 3.25, "min": 1, "max": 4})
        rows, checks, anki = self.records()
        cell = report.cell_summary(rows, checks, anki, 10000, "rust")
        self.assertEqual(cell["time_ns"]["median"], 5500000)
        self.assertEqual(cell["apkg_bytes"]["median"], 100)
        self.assertEqual(cell["peak_rss_bytes"]["n"], 5)
        self.assertEqual(cell["status"], "verified")

    def test_failed_cell_does_not_hide_other_sizes_or_use_survivors(self):
        rows, checks, anki = self.records()
        rows[0]["status"] = "timeout"
        cell = report.cell_summary(rows, checks, anki, 10000, "rust")
        self.assertIsNone(cell["time_ns"])
        self.assertEqual(cell["timing_succeeded"], 9)
        rows[0]["status"] = "success"
        checks[rows[1]["id"]]["status"] = "invalid_artifact"
        self.assertIsNone(report.cell_summary(rows, checks, anki, 10000, "rust")["time_ns"])

    def test_unmatched_memory_and_missing_anki_are_distinct(self):
        rows, checks, anki = self.records()
        rows[-1]["memory"]["metric"] = "sampled_tree"
        cell = report.cell_summary(rows, checks, anki, 10000, "rust")
        self.assertEqual(cell["status"], "verified")
        self.assertIsNone(cell["peak_rss_bytes"])
        cell = report.cell_summary(rows, checks, {}, 10000, "rust")
        self.assertEqual(cell["status"], "unverified")
        self.assertIsNotNone(cell["time_ns"])

    def test_ratio_direction_dirty_provenance_and_no_headline(self):
        rows, checks, anki = self.records()
        other, c2, a2 = self.records(adapter="genanki")
        for r in other:
            r["measurement"]["elapsed_ns"] *= .8
        m = {"run_id": "test", "git_status": "dirty", "identity_unchanged": False, "identity_before": {"upstream_dirty": ""}}
        result = report.summarize(m, rows + other, checks | c2, anki | a2)
        pair = result["comparisons"][-1]
        self.assertAlmostEqual(pair["genanki_over_rust"], .8)
        self.assertAlmostEqual(pair["genanki_minus_rust_ms"], -1.1)
        self.assertEqual(len(result["cells"]), 8)
        self.assertEqual(len(result["comparisons"]), 4)
        self.assertFalse(result["headline_eligible"])
        self.assertEqual(pair["status"], "draft")

    def test_measuring_defers_verification_and_cancellation_stops_schedule(self):
        for cancel in (None, "timing", "preflight"):
            with self.subTest(cancel=cancel), tempfile.TemporaryDirectory() as directory:
                events = []
                def attempt(item, run, adapters):
                    events.append("launch:" + item["role"])
                    return {**item, "status": "cancelled" if item["role"] == cancel else "success"}
                def verification(run, records):
                    events.append("verify")
                    return {r["id"]: {"status": "passed"} for r in records if r["status"] == "success"}
                m = {"host": {}, "interruptions": [], "identity_before": {}}
                with patch.object(bench, "SUITE", Path(directory)), patch.object(sys, "prefix", str(Path(directory) / ".venv")), \
                     patch.object(bench, "manifest", return_value=copy.deepcopy(m)), patch.object(bench.workload, "generate", return_value={}), \
                     patch.object(bench, "run_attempt", side_effect=attempt), patch.object(bench, "verify_records", side_effect=verification), \
                     patch.object(bench, "registry", return_value=[{"id": "rust"}, {"id": "genanki"}]), \
                     patch.object(bench, "artifact_size", return_value=0), patch.object(bench, "complete_oracle"), \
                     patch.object(bench, "identity_snapshot", return_value={}), patch.object(bench, "host_state", return_value={}), \
                     patch.object(bench, "cleanup"), patch.object(report, "render"), patch("builtins.print"):
                    code = bench.execute("full", "test")
                preflight_count = 1 if cancel == "preflight" else 2
                self.assertEqual(events[:preflight_count + 1], ["launch:preflight"] * preflight_count + ["verify"])
                self.assertEqual(events[-1], "verify")
                self.assertEqual(events.count("verify"), 2)
                if cancel:
                    self.assertEqual(code, 1)
                    self.assertEqual(events.count("launch:timing"), 1 if cancel == "timing" else 0)
                    self.assertNotIn("launch:memory", events)
                    run = Path(directory) / ".work/runs/test"
                    saved = json.loads((run / "manifest.json").read_text())
                    self.assertEqual(len(saved["interruptions"]), 1)
                    self.assertTrue(saved["interruptions"][0]["attempt"].startswith(cancel))
                    rows = [json.loads(line) for line in (run / "attempts.jsonl").read_text().splitlines()]
                    cancelled = next(i for i, r in enumerate(rows) if r["status"] == "cancelled")
                    self.assertTrue(all(r["status"] == "not_run" and r["reason"] == "cancelled" for r in rows[cancelled + 1:]))
                else:
                    self.assertEqual(code, 0)
                    self.assertEqual(events.count("launch:timing"), 80)
                    self.assertEqual(events.count("launch:memory"), 40)


if __name__ == "__main__":
    unittest.main()
