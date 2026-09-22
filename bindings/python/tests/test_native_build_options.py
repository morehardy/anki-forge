import gc
import json
import pytest

from anki_forge import Note, Project


def test_artifacts_directory_and_report_use_captured_base_dir(tmp_path, monkeypatch):
    from anki_forge import BuildOptions

    root = tmp_path / "项目 files"
    root.mkdir()
    project = Project("Options", base_dir=root).add_note(Note.basic("Question", "Answer"))
    monkeypatch.chdir(tmp_path)
    report = project.build(BuildOptions(artifacts_dir="artifacts", report_json="report.json", inspect=False))
    report.ensure_success()
    assert report.inspect is None
    assert report.counts == {"notes": 1, "cards": 1, "media": 0}
    artifact_path = report.artifact.path
    assert artifact_path.is_relative_to(root / "artifacts")
    saved = json.loads((root / "report.json").read_text())
    assert saved["metrics"] == report.metrics
    assert saved["policy"] == report.policy
    assert saved["artifact"]["path"] == str(artifact_path)
    del report
    gc.collect()
    assert artifact_path.is_file()


@pytest.mark.parametrize("budget,resource", [
    ("max_archive_bytes", "archive_bytes"), ("max_entries", "entries"),
    ("max_central_directory_bytes", "central_directory_bytes"),
    ("max_zip_entry_bytes", "zip_entry_bytes"), ("max_zip_total_bytes", "zip_total_bytes"),
    ("max_meta_bytes", "meta_bytes"), ("max_media_map_bytes", "media_map_bytes"),
    ("max_collection_bytes", "collection_bytes"), ("max_media_bytes", "media_bytes"),
    ("max_decoded_total_bytes", "decoded_total_bytes"), ("max_zstd_window_bytes", "zstd_window_bytes"),
])
def test_each_inspect_budget_protects_existing_output_even_when_inspect_is_false(tmp_path, budget, resource):
    from anki_forge import BuildError, BuildOptions, InspectLimits

    project = Project("Budgets")
    media = project.media.add_bytes(source_label="sound", data=b"RIFF sound", export_as="sound.wav")
    project.add_note(Note.basic("Question", "Answer").sound("back", media))
    output = tmp_path / "previous.apkg"
    output.write_bytes(b"old output must remain")
    report = project.build(BuildOptions(output=output, inspect=False, inspect_limits=InspectLimits(**{budget: 0})))
    assert report.status != "success"
    assert report.inspect is None
    diagnostics = [d for d in report.diagnostics if d.code == "INSPECT.RESOURCE_LIMIT_EXCEEDED"]
    assert len(diagnostics) == 1
    assert resource in diagnostics[0].message
    assert output.read_bytes() == b"old output must remain"
    with pytest.raises(BuildError) as caught:
        report.ensure_success()
    assert caught.value.report is report
    assert caught.value.code == "INSPECT.RESOURCE_LIMIT_EXCEEDED"
    assert caught.value.failure_cause == report.failure_cause


def test_budget_values_and_conflicting_output_are_rejected_before_publication(tmp_path):
    from anki_forge import BuildOptions, InspectLimits, ValidationError

    for value in [True, False, -1, 1.5, "1024", 2**64]:
        with pytest.raises(ValidationError):
            InspectLimits(max_archive_bytes=value)
    limits = InspectLimits(max_archive_bytes=2**64 - 1)
    assert limits.max_archive_bytes == 18446744073709551615
    project = Project("Options", base_dir=tmp_path).add_note(Note.basic("Question", "Answer"))
    with pytest.raises(ValidationError, match="conflicting output"):
        project.write_apkg("second.apkg", options=BuildOptions(output="first.apkg"))
    assert list(tmp_path.iterdir()) == []
    project.write_apkg("valid.apkg", options=BuildOptions(output=tmp_path / "valid.apkg", inspect_limits=limits)).ensure_success()


def test_media_modes_and_policy_are_applied_by_core_build(tmp_path):
    from anki_forge import BuildOptions, MediaPolicy, ValidationError

    source = tmp_path / "source.wav"
    source.write_bytes(b"RIFF content")
    project = Project("Media options", base_dir=tmp_path)
    reference = project.media.add_file(source, export_as="sound.wav")
    project.add_note(Note.basic("Question", "Answer").sound("back", reference))
    path_backed = project.build(BuildOptions(media_store_dir="store"))
    path_backed.ensure_success()
    assert path_backed.media["entries"][0]["source_mode"] == "path_backed"
    assert (tmp_path / "store").is_dir()
    inline = project.build(BuildOptions(self_contained=True))
    inline.ensure_success()
    assert inline.media["entries"][0]["source_mode"] == "inline"
    with pytest.raises(ValidationError, match="conflict"):
        BuildOptions(self_contained=True, media_mode="path_backed")
    project.media.add_bytes(source_label="unused", data=b"RIFF unused", export_as="unused.wav")
    strict = project.build(BuildOptions(media_policy=MediaPolicy(unused_binding="error")))
    assert strict.status == "invalid"
    assert any(d.code == "MEDIA.UNUSED_BINDING" and d.severity == "error" for d in strict.diagnostics)


def test_report_and_staging_collisions_are_rejected_without_overwriting_files(tmp_path):
    from anki_forge import BuildOptions

    project = Project("Paths", stable_id="paths", base_dir=tmp_path).add_note(Note.basic("Question", "Answer", stable_id="one"))
    baseline = tmp_path / "baseline.apkg"
    project.write_apkg(baseline).ensure_success()
    original = baseline.read_bytes()
    alias = tmp_path / "alias.apkg"
    alias.symlink_to(baseline)
    report = project.write_apkg(alias, compare_to=baseline)
    assert report.status == "invalid"
    assert any(d.code == "PROJECT.PATH_COLLISION" for d in report.diagnostics)
    assert alias.is_symlink() and baseline.read_bytes() == original
    report = project.write_apkg(baseline, report_json=baseline)
    assert report.status == "invalid"
    assert baseline.read_bytes() == original
    staging = tmp_path / "artifacts/staging"
    staging.mkdir(parents=True)
    staged_baseline = staging / "previous.apkg"
    staged_baseline.write_bytes(original)
    report = project.build(BuildOptions(output="output.apkg", artifacts_dir="artifacts", compare_to=staged_baseline))
    assert report.status == "invalid"
    assert any(d.code == "PROJECT.PATH_COLLISION" for d in report.diagnostics)
    assert staged_baseline.read_bytes() == original
    assert not (tmp_path / "output.apkg").exists()


@pytest.mark.parametrize("mode", ["strict", "report_only", "report-only", "disabled"])
def test_inspect_budget_also_applies_to_baseline_and_respects_update_mode(tmp_path, mode):
    from anki_forge import BuildOptions, InspectLimits

    baseline_project = Project("Baseline", stable_id="baseline")
    media = baseline_project.media.add_bytes(source_label="sound", data=b"RIFF more than ten bytes", export_as="sound.wav")
    baseline_project.add_note(Note.basic("Question", "Answer", stable_id="one").sound("back", media))
    baseline = tmp_path / "baseline.apkg"
    baseline_project.write_apkg(baseline).ensure_success()
    original = baseline.read_bytes()
    project = Project("Baseline", stable_id="baseline").add_note(Note.basic("Question", "Answer", stable_id="one"))
    output = tmp_path / "output.apkg"
    output.write_bytes(b"previous output")
    report = project.build(BuildOptions(output=output, compare_to=baseline, update_safety=mode,
        inspect_limits=InspectLimits(max_media_bytes=10)))
    assert report.comparison == "unavailable"
    assert any(d.code == "INSPECT.RESOURCE_LIMIT_EXCEEDED" for d in report.diagnostics)
    assert baseline.read_bytes() == original
    if mode == "strict":
        assert report.status != "success"
        assert output.read_bytes() == b"previous output"
    else:
        report.ensure_success()


def test_read_only_output_directory_keeps_previous_publication(tmp_path):
    import os
    if os.name == "nt":
        pytest.skip("POSIX permission bits do not make a Windows directory read-only")
    directory = tmp_path / "read-only"
    directory.mkdir()
    target = directory / "deck.apkg"
    target.write_bytes(b"previous publication")
    directory.chmod(0o555)
    try:
        if os.access(directory, os.W_OK):
            pytest.skip("current account bypasses POSIX directory permissions")
        report = Project("Permissions").add_note(Note.basic("Front", "Back")).write_apkg(target)
        assert report.status != "success"
        assert report.diagnostics
        assert target.read_bytes() == b"previous publication"
    finally:
        directory.chmod(0o755)
