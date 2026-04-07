from __future__ import annotations

from pathlib import Path


def warning_count(result: dict) -> int:
    diagnostics = result.get("diagnostics", {})
    items = diagnostics.get("items", [])
    return sum(1 for item in items if item.get("level") == "warning")


def _artifact_path_from_ref(artifacts_dir: str | None, ref: str | None) -> str | None:
    if not artifacts_dir or not ref:
        return None
    normalized_ref = ref.removeprefix("artifacts/")
    return str(Path(artifacts_dir) / Path(normalized_ref))


def helper_view(command: str, result: dict, request: dict) -> dict:
    return {
        "is_invalid": result.get("result_status") == "invalid",
        "is_degraded": result.get("observation_status") == "degraded",
        "is_partial": result.get("comparison_status") == "partial",
        "warning_count": warning_count(result),
        "artifact_paths": (
            {
                "staging_manifest": _artifact_path_from_ref(
                    request.get("artifacts_dir"),
                    result.get("staging_ref"),
                ),
                "apkg": _artifact_path_from_ref(
                    request.get("artifacts_dir"),
                    result.get("apkg_ref"),
                ),
            }
            if command == "build"
            else None
        ),
    }
