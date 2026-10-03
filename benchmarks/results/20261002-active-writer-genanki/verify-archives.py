"""Check every archive member against the retained SHA-256 manifest."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import tarfile

ROOT = Path(__file__).resolve().parent


def verify():
    expected = json.loads((ROOT / "archive-manifest.json").read_text())["archives"]
    counts = {}
    for name, members in expected.items():
        with tarfile.open(ROOT / name) as archive:
            entries = archive.getmembers()
            assert len(entries) == len(members), name
            assert len({entry.name for entry in entries}) == len(entries), name
            assert {entry.name for entry in entries} == set(members), name
            for entry in entries:
                path = PurePosixPath(entry.name)
                assert entry.isfile() and not path.is_absolute() and ".." not in path.parts, entry.name
                with archive.extractfile(entry) as stream:
                    assert hashlib.file_digest(stream, "sha256").hexdigest() == members[entry.name], entry.name
            counts[name] = len(entries)
    return {"status": "passed", "archive_members": counts}


if __name__ == "__main__":
    print(json.dumps(verify(), indent=2))
