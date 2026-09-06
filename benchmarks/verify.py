"""Read original canonical rows before any Anki import or schema upgrade."""
import hashlib
import html
import io
import json
import sqlite3
import subprocess
import tempfile
import zipfile
from pathlib import Path

import zstandard
from workload import QFMT, AFMT, serialize

MAX_BYTES = 128 * 1024 * 1024


class InvalidArtifact(ValueError):
    pass


class UnsupportedVerifier(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise InvalidArtifact(message)


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def literal(raw):
    # The workload is plain text. Check raw markup BEFORE decoding exactly once.
    require("<" not in raw and ">" not in raw, "active/unescaped markup in plain-text field")
    return html.unescape(raw)


def zstd_decode(raw):
    with zstandard.ZstdDecompressor(max_window_size=MAX_BYTES).stream_reader(io.BytesIO(raw)) as reader:
        data = reader.read(MAX_BYTES + 1)
    require(len(data) <= MAX_BYTES, "decoded data exceeds benchmark verifier budget")
    return data


def read_package(path):
    require(path.stat().st_size <= MAX_BYTES, "archive exceeds verifier budget")
    with zipfile.ZipFile(path) as archive:
        info = archive.infolist()
        names = [entry.filename for entry in info]
        require(len(names) == len(set(names)), "duplicate archive entry")
        require(len(info) <= 8 and sum(e.file_size for e in info) <= MAX_BYTES, "archive expansion budget")
        meta = archive.read("meta") if "meta" in names else None
        if meta is None or meta == b"\x08\x01":
            canonical, version = "collection.anki2", 1
        elif meta == b"\x08\x02":
            canonical, version = "collection.anki21", 2
        elif meta == b"\x08\x03":
            canonical, version = "collection.anki21b", 3
        else:
            raise UnsupportedVerifier(f"unknown APKG metadata: {meta!r}")
        require(canonical in names and "media" in names, "missing canonical collection or media map")
        allowed = {canonical, "media", "meta"}
        if version > 1:
            allowed.add("collection.anki2")
        require(set(names) <= allowed, "unexpected media or unaccounted archive payload")
        raw = archive.read(canonical)
        media = archive.read("media")
        if version == 3:
            raw, media = zstd_decode(raw), zstd_decode(media)
            require(media == b"", "nonempty modern media protobuf")
        else:
            require(json.loads(media) == {}, "nonempty legacy media map")
        require(raw.startswith(b"SQLite format 3\x00"), "canonical entry is not SQLite")
        return raw, {"version": version, "canonical_entry": canonical,
                     "nested_compression": "zstd" if version == 3 else "none",
                     "payloads": [{"name": e.filename, "zip_method": e.compress_type,
                                   "stored_bytes": e.compress_size, "decoded_zip_bytes": e.file_size,
                                   "role": "canonical" if e.filename == canonical else
                                           "compatibility_placeholder" if e.filename == "collection.anki2" else "metadata"}
                                  for e in info], "media_count": 0}


def check_rows(db, expected):
    """Do not join/project before checking physical multiplicity and references."""
    require(db.execute("PRAGMA integrity_check").fetchall() == [("ok",)], "SQLite integrity check")
    tables = {r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table'")}
    require({"col", "notes", "cards", "revlog"} <= tables, "missing required tables")
    schema = db.execute("SELECT ver FROM col").fetchone()[0]
    if schema == 11:
        model_raw, deck_raw = db.execute("SELECT models,decks FROM col").fetchone()
        models, decks = json.loads(model_raw), json.loads(deck_raw)
        model_ids, deck_ids = {int(k) for k in models}, {int(k) for k in decks}
    elif schema in (15, 16, 17, 18):
        model_ids = {r[0] for r in db.execute("SELECT id FROM notetypes")}
        deck_ids = {r[0] for r in db.execute("SELECT id FROM decks")}
        models = decks = None
    else:
        raise UnsupportedVerifier(f"unsupported original SQLite schema {schema}")
    notes = db.execute("SELECT id,guid,mid,flds,tags FROM notes ORDER BY id").fetchall()
    cards = db.execute("SELECT id,nid,did,ord,type,queue,reps,lapses FROM cards ORDER BY id").fetchall()
    count = expected["note_count"]
    require(len(notes) == len(cards) == count, "physical note/card count")
    require(len({n[0] for n in notes}) == count, "duplicate raw note id")
    require(len({c[0] for c in cards}) == count, "duplicate raw card id")
    require(all(n[1] for n in notes) and len({n[1] for n in notes}) == count, "nonunique/empty GUID")
    require(all(n[2] in model_ids for n in notes), "dangling note type")
    note_ids = {n[0] for n in notes}
    require(all(c[1] in note_ids for c in cards), "orphan card")
    require(all(c[2] in deck_ids for c in cards), "dangling deck")
    require(len({c[1] for c in cards}) == count, "duplicate same-note card")
    require(all(c[3] == 0 for c in cards), "wrong card ordinal")
    require(all(c[4:] == (0, 0, 0, 0) for c in cards), "unexpected scheduling history")
    require(db.execute("SELECT COUNT(*) FROM revlog").fetchone()[0] == 0, "unexpected review log")
    require(len({n[2] for n in notes}) == 1, "multiple used note types")
    require(len({c[2] for c in cards}) == 1, "multiple populated decks")
    required = {n["front"]: n["back"] for n in expected["notes"]}
    observed = {}
    by_guid = {}
    for _, guid, _, fields, tags in notes:
        parts = fields.split("\x1f")
        require(len(parts) == 2, "not exactly two stored field segments")
        require(not tags.split(), "nonempty logical tags")
        front, back = map(literal, parts)
        require(front not in observed, "duplicate fixture Front")
        require(front in required and required[front] == back, "field content mismatch")
        observed[front] = back
        by_guid[guid] = (front, back)
    require(observed == required, "full content mismatch")
    logical_digest = hashlib.sha256(serialize(sorted(observed.items()))).hexdigest()
    used_mid, used_did = notes[0][2], cards[0][2]
    if models is not None:
        model, deck = models[str(used_mid)], decks[str(used_did)]
        require(model["type"] == 0, "non-Basic model")
        fields = sorted(model["flds"], key=lambda f: f["ord"])
        require([(f["ord"], f["name"]) for f in fields] == [(0, "Front"), (1, "Back")], "field schema")
        require(len(model["tmpls"]) == 1, "template count")
        template = model["tmpls"][0]
        require(template["ord"] == 0 and template["qfmt"] == QFMT and template["afmt"] == AFMT, "Basic templates")
        require(deck["name"] == expected["deck_name"], "deck name")
        require(template.get("did") in (None, 0, used_did), "template deck override")
        semantic = {"status": "passed", "reader": "benchmark-legacy-schema11-v1",
                    "stock_model_name": model["name"], "css": model.get("css", ""),
                    "repository_inspector": "unsupported: requires modern decks table"}
    else:
        require(db.execute("SELECT name FROM decks WHERE id=?", (used_did,)).fetchone()[0] == expected["deck_name"], "deck name")
        require(db.execute("SELECT ord,name FROM fields WHERE ntid=? ORDER BY ord", (used_mid,)).fetchall()
                == [(0, "Front"), (1, "Back")], "field schema")
        require(db.execute("SELECT ord FROM templates WHERE ntid=?", (used_mid,)).fetchall() == [(0,)], "template count")
        semantic = None
    return {"status": "passed", "notes": count, "cards": count, "schema": schema,
            "used_note_types": 1, "populated_decks": 1, "logical_sha256": logical_digest}, by_guid, used_mid, semantic


def check_projection(path, inspector, expected, by_guid, used_mid):
    process = subprocess.run([str(inspector), "inspect", "--apkg", str(path), "--output", "contract-json"],
                             capture_output=True, timeout=120)
    if process.returncode:
        raise UnsupportedVerifier("repository inspector: " + process.stderr.decode(errors="replace")[:1500])
    try:
        report = json.loads(process.stdout)
        o = report["observations"]
        require(report["observation_status"] == "complete", "incomplete semantic inspection")
        models = [m for m in o["notetypes"] if m["anki_model_id"] == used_mid]
        require(len(models) == 1 and models[0]["kind"] == "normal", "used normal model")
        mid = models[0]["id"]
        fields = sorted((f["ord"], f["name"]) for f in o["fields"] if f["notetype_id"] == mid)
        require(fields == [(0, "Front"), (1, "Back")], "semantic fields")
        templates = [t for t in o["templates"] if t["notetype_id"] == mid]
        require(len(templates) == 1 and templates[0]["ord"] == 0
                and templates[0]["question_format"] == QFMT and templates[0]["answer_format"] == AFMT, "semantic template")
        notes = [r for r in o["references"] if "fields" in r]
        cards = [r for r in o["references"] if "note_id" in r and "ord" in r]
        require(len(notes) == len(cards) == expected["note_count"], "projected counts")
        require({c["note_id"] for c in cards} == set(by_guid), "projected card associations")
        require(all(c["ord"] == 0 and c["deck_name"] == expected["deck_name"] for c in cards), "projected cards")
        require(not o["media"], "projected media")
        for note in notes:
            require(note["notetype_id"] == mid and note["deck_name"] == expected["deck_name"], "projected association")
            require(not any(tag.strip() for tag in note["tags"]), "projected tags")
            require(set(note["fields"]) == {"Front", "Back"}, "projected fields")
            require(tuple(literal(note["fields"][key]) for key in ("Front", "Back")) == by_guid[note["id"]], "projected content")
        return {"status": "passed", "reader": "repository-inspector-modern-v1",
                "observation_model_version": report["observation_model_version"],
                "stock_model_name": models[0]["name"], "css": models[0]["css"],
                "inspection_sha256": hashlib.sha256(process.stdout).hexdigest()}
    except (KeyError, TypeError) as error:
        raise UnsupportedVerifier(f"unsupported inspector observation: {error}") from error


def verify_artifact(path, expected, inspector):
    result = {"artifact_sha256": sha256(path), "artifact_bytes": path.stat().st_size}
    try:
        raw, package = read_package(path)
        result["package"] = package
        with tempfile.TemporaryDirectory(prefix="verify-") as directory:
            database = Path(directory) / "original.sqlite"
            database.write_bytes(raw)
            db = sqlite3.connect(database.as_uri() + "?mode=ro&immutable=1", uri=True)
            try:
                physical, by_guid, used_mid, semantic = check_rows(db, expected)
            finally:
                db.close()
        result["physical"] = physical
        result["semantic"] = semantic or check_projection(path, inspector, expected, by_guid, used_mid)
        result["status"] = "passed"
    except UnsupportedVerifier as error:
        result.update(status="unsupported_verifier", reason=str(error))
    except (InvalidArtifact, sqlite3.DatabaseError, zipfile.BadZipFile, zstandard.ZstdError,
            json.JSONDecodeError, UnicodeDecodeError) as error:
        result.update(status="invalid_artifact", reason=str(error))
    except (OSError, subprocess.TimeoutExpired) as error:
        result.update(status="verification_unavailable", reason=str(error))
    return result
