from __future__ import annotations

import sys
from pathlib import Path

from anki_forge import Note, Project


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: wheel_smoke_project.py <output.apkg>")
    out = Path(sys.argv[1])
    out.parent.mkdir(parents=True, exist_ok=True)
    project = Project("Wheel")
    project.add_note(Note.basic("Front", "Back"))
    report = project.write_apkg(out)
    report.ensure_success()
    if not out.is_file():
        raise SystemExit(f"APKG was not written: {out}")


if __name__ == "__main__":
    main()
