from copy import deepcopy
import pytest
from anki_forge import Diagnostic, DiagnosticsError, ProtocolError
from anki_forge.report import BuildReport, _report_to_json

def build_report_payload(**overrides):
    payload = {
        "kind": "anki-forge-build-report",
        "schema_version": "phase4-build-report-v2",
        "tool_version": "test",
        "status": "success",
        "comparison": "not_requested",
        "artifact": {"path": "deck.apkg"},
        "counts": {"notes": 1, "cards": 1, "media": 0},
        "media": {
            "objects": 0,
            "bindings": 0,
            "references": 0,
            "missing_references": 0,
            "unsafe_references": 0,
            "unused_bindings": 0,
            "unique_bytes": 0,
            "entries": [],
        },
        "diagnostics": [],
        "metrics": {"duration_ms": 1},
        "policy": {
            "status": "not_evaluated",
            "threshold": None,
            "highest_risk": None,
            "blocking_findings": [],
        },
    }
    payload.update(deepcopy(overrides))
    return payload


def diagnostic_payload(
    *,
    code="E",
    severity="error",
    message="bad",
    domain="project",
    stage="build",
    path=None,
    suggested_fix=None,
):
    return {
        "code": code,
        "severity": severity,
        "domain": domain,
        "stage": stage,
        "path": path,
        "message": message,
        "suggested_fix": suggested_fix,
    }


def test_report_preserves_metrics_policy_and_unknown_fields_losslessly():
    payload = build_report_payload(
        status="blocked", artifact=None,
        metrics={"duration_ms": 173, "future_count": 2**60 + 1},
        policy={"status": "blocked", "threshold": "high", "highest_risk": "critical", "blocking_findings": ["risk:1"]},
        future={"nested": ["kept"]},
    )
    expected = deepcopy(payload)
    report = BuildReport.from_json(payload)
    payload["future"]["nested"].append("changed input")

    assert report.metrics == expected["metrics"]
    assert report.policy == expected["policy"]
    assert report.tool_version == "test"
    assert report.schema_version == "phase4-build-report-v2"
    assert report.raw == expected
    assert report.to_json() == expected
    assert _report_to_json(report) == expected


def test_report_success_warning_does_not_raise():
    report = BuildReport.from_json(build_report_payload(
        diagnostics=[diagnostic_payload(code="W", severity="warning", message="warn")],
    ))
    report.ensure_success()


def test_report_ensure_success_raises_for_invalid_report():
    payload = build_report_payload(
        status="invalid",
        artifact=None,
        counts={"notes": 0, "cards": 0, "media": 0},
        diagnostics=[diagnostic_payload()],
    )
    with pytest.raises(DiagnosticsError):
        BuildReport.from_json(payload).ensure_success()


def test_report_rejects_unknown_comparison():
    payload = build_report_payload(comparison="garbage")
    with pytest.raises(ProtocolError):
        BuildReport.from_json(payload)


def test_report_rejects_wrong_kind_and_schema_version():
    base = build_report_payload()
    with pytest.raises(ProtocolError):
        BuildReport.from_json({**base, "kind": "wrong"})
    with pytest.raises(ProtocolError):
        BuildReport.from_json({**base, "schema_version": "future"})


def test_report_rejects_v2_diagnostic_missing_required_fields():
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(
            diagnostics=[{"code": "E", "severity": "error", "message": "bad"}],
        ))
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(
            diagnostics=[diagnostic_payload(domain="")],
        ))


@pytest.mark.parametrize(
    ("code", "domain", "stage"),
    [
        ("AFID.BLANK", "identity", "validate"),
        ("COMPARE.MISSING_BASELINE", "comparison", "compare"),
        ("DECK.STABLE_ID_BLANK", "deck", "validate"),
        ("MEDIA.SOURCE_MISSING", "media", "normalize"),
        ("NOTETYPE.FIELD_MISSING", "notetype", "validate"),
        ("TEMPLATE.INVALID", "notetype", "validate"),
        ("PRODUCT.CLOZE_MARKER_MISSING", "product", "validate"),
        ("PROJECT.BUILD_INTERNAL", "project", "build"),
        ("RISK.THRESHOLD_EXCEEDED", "risk", "risk"),
        ("UPDATE.BASELINE_APKG_UNREADABLE", "update_safety", "update_safety"),
        ("UNKNOWN", "unknown", "unknown"),
    ],
)
def test_python_report_json_infers_required_diagnostic_domain_and_stage(code, domain, stage):
    report = BuildReport(
        status="invalid",
        comparison="not_requested",
        artifact=None,
        counts={"notes": 0, "cards": 0, "media": 0},
        media={
            "objects": 0,
            "bindings": 0,
            "references": 0,
            "missing_references": 0,
            "unsafe_references": 0,
            "unused_bindings": 0,
            "unique_bytes": 0,
        },
        diagnostics=(
            Diagnostic(
                code=code,
                severity="error",
                message="missing media",
            ),
        ),
    )

    payload = _report_to_json(report)

    assert payload["diagnostics"][0]["domain"] == domain
    assert payload["diagnostics"][0]["stage"] == stage
    parsed = BuildReport.from_json(payload)
    assert parsed.diagnostics[0].domain == domain
    assert parsed.diagnostics[0].stage == stage


def test_report_parses_schema_media_summary_unique_bytes():
    report = BuildReport.from_json(build_report_payload(
        media={
            "objects": 2,
            "bindings": 3,
            "references": 4,
            "missing_references": 0,
            "unsafe_references": 1,
            "unused_bindings": 5,
            "unique_bytes": 987,
            "entries": [
                {
                    "id": "media:audio.wav",
                    "filename": "audio.wav",
                    "source_mode": "path_backed",
                    "size_bytes": 987,
                }
            ],
        },
    ))

    assert report.media["unique_bytes"] == 987
    assert report.media["entries"][0]["source_mode"] == "path_backed"


def test_report_accepts_v1_media_summary_without_entries():
    payload = build_report_payload()
    del payload["media"]["entries"]

    report = BuildReport.from_json(payload)

    assert report.media["entries"] == []


def test_report_projects_update_safety_summary_dict():
    report = BuildReport.from_json(build_report_payload(update_safety={
        "mode": "strict",
        "baseline_sources": [
            {
                "source_kind": "lockfile",
                "source_ref": "baseline.identity_lockfile.primary",
                "display_path": "anki-forge.lock.json",
                "status": "loaded",
                "used_for_reconcile": True,
                "limitations": [],
                "diagnostic_codes": [],
            }
        ],
        "notes_preserved": 1,
        "notes_derived": 0,
        "notes_failed": 0,
        "baseline_conflicts": 0,
        "blocking_diagnostics": [],
        "lockfile_written": False,
    }))

    assert report.update_safety["mode"] == "strict"
    assert report.update_safety["baseline_sources"][0]["source_kind"] == "lockfile"
    assert report.update_safety["lockfile_written"] is False


def test_report_rejects_legacy_media_bytes_payload():
    payload = build_report_payload(media={"objects": 0, "bindings": 0, "bytes": 0})

    with pytest.raises(ProtocolError):
        BuildReport.from_json(payload)


def test_report_rejects_unknown_status():
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(status="warning"))


def test_report_rejects_missing_or_invalid_artifact_path():
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(artifact={}))
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(artifact={"path": ""}))

    report = BuildReport.from_json(build_report_payload(artifact={"path": "deck.apkg", "extra": True}))
    assert report.artifact == {"path": "deck.apkg", "extra": True}


def test_report_rejects_bool_integer_field():
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(counts={"notes": True, "cards": 1, "media": 0}))
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(metrics={"duration_ms": True}))


def test_report_rejects_invalid_policy_shape_and_status():
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(policy={"status": "not_evaluated"}))
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(policy={
            "status": "not_applicable",
            "threshold": None,
            "highest_risk": None,
            "blocking_findings": [],
        }))

    BuildReport.from_json(build_report_payload(policy={
        "status": "not_evaluated",
        "threshold": None,
        "highest_risk": None,
        "blocking_findings": [],
        "extra": "ok",
    }))


def test_report_accepts_forward_compatible_summary_fields():
    report = BuildReport.from_json(build_report_payload(
        counts={"notes": 1, "cards": 1, "media": 0, "future": 99},
        media={
            "objects": 0,
            "bindings": 0,
            "references": 0,
            "missing_references": 0,
            "unsafe_references": 0,
            "unused_bindings": 0,
            "unique_bytes": 0,
            "entries": [],
            "future": 99,
        },
        metrics={"duration_ms": 1, "future": True},
    ))

    assert report.counts == {"notes": 1, "cards": 1, "media": 0}
    assert report.media == {
        "objects": 0,
        "bindings": 0,
        "references": 0,
        "missing_references": 0,
        "unsafe_references": 0,
        "unused_bindings": 0,
        "unique_bytes": 0,
        "entries": [],
    }

