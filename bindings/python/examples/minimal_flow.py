from __future__ import annotations

from pathlib import Path

from anki_forge_python import build, diff, inspect, normalize, resolve_runtime


BINDINGS_PYTHON_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = BINDINGS_PYTHON_ROOT.parents[1]


def main() -> None:
    runtime = resolve_runtime(cwd=BINDINGS_PYTHON_ROOT)
    print("resolved runtime =>", runtime)

    normalized = normalize(
        {"input_path": str(REPO_ROOT / "contracts/fixtures/valid/minimal-authoring-ir.json")},
        cwd=BINDINGS_PYTHON_ROOT,
    )
    print(
        "normalize status =>",
        normalized["result_status"],
        "warnings =>",
        normalized["helper"]["warning_count"],
    )

    artifacts_dir = REPO_ROOT / "tmp/phase4-python-example/basic"
    build_result = build(
        {
            "input_path": str(REPO_ROOT / "contracts/fixtures/phase3/inputs/basic-normalized-ir.json"),
            "artifacts_dir": str(artifacts_dir),
        },
        cwd=BINDINGS_PYTHON_ROOT,
    )
    print("build status =>", build_result["result_status"])

    staging_report = inspect(
        {"staging_path": str(artifacts_dir / "staging/manifest.json")},
        cwd=BINDINGS_PYTHON_ROOT,
    )
    apkg_report = inspect(
        {"apkg_path": str(artifacts_dir / "package.apkg")},
        cwd=BINDINGS_PYTHON_ROOT,
    )
    print(
        "inspect statuses =>",
        staging_report["observation_status"],
        apkg_report["observation_status"],
    )

    diff_result = diff(
        {
            "left_path": str(REPO_ROOT / "contracts/fixtures/phase3/expected/basic.inspect.json"),
            "right_path": str(REPO_ROOT / "contracts/fixtures/phase3/expected/basic.inspect.json"),
        },
        cwd=BINDINGS_PYTHON_ROOT,
    )
    print("diff status =>", diff_result["comparison_status"])


if __name__ == "__main__":
    main()
