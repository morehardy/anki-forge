"""Real Anki oracle; only opens collections under a temporary directory."""

import json
import sys
import tempfile
from importlib.metadata import version
from pathlib import Path

from anki.collection import Collection
from anki import import_export_pb2 as pb


def card_state(collection):
    return collection.db.all(
        "select n.guid, n.id, n.mid, c.id, c.type, c.queue, c.due, c.ivl, c.factor, c.reps, c.lapses "
        "from notes n join cards c on c.nid = n.id order by n.guid, c.ord"
    )


results = []
for condition in (
    pb.IMPORT_ANKI_PACKAGE_UPDATE_CONDITION_IF_NEWER,
    pb.IMPORT_ANKI_PACKAGE_UPDATE_CONDITION_ALWAYS,
):
    for merge in (False, True):
        with tempfile.TemporaryDirectory(prefix="anki-note-revisions-") as directory:
            collection = Collection(str(Path(directory) / "oracle.anki2"))
            try:
                options = pb.ImportAnkiPackageOptions(
                    merge_notetypes=merge,
                    update_notes=condition,
                    update_notetypes=condition,
                )
                collection.import_anki_package(pb.ImportAnkiPackageRequest(package_path=sys.argv[1], options=options))
                for card_id in collection.db.list("select id from cards"):
                    card = collection.get_card(card_id)
                    card.type, card.queue, card.due = 2, 2, 123
                    card.ivl, card.factor, card.reps, card.lapses = 42, 2500, 7, 1
                    collection.update_card(card)
                before = card_state(collection)
                models = collection.db.all("select name, id from notetypes order by id")
                result = collection.import_anki_package(pb.ImportAnkiPackageRequest(package_path=sys.argv[2], options=options))
                assert not result.log.conflicting, result
                assert len(result.log.updated) == 1, result
                assert len(result.log.duplicate) == 1, result
                assert collection.note_count() == 2
                assert card_state(collection) == before
                assert collection.db.all("select name, id from notetypes order by id") == models
                fields, tags, mtime = collection.db.first("select flds, tags, mod from notes where guid = 'changed'")
                assert fields.split("\x1f")[1] == "B", fields
                assert "new-tag" in tags.split(), tags
                assert mtime == 2, mtime
                assert collection.db.scalar("select mod from notes where guid = 'unchanged'") == 1
                repeat = collection.import_anki_package(pb.ImportAnkiPackageRequest(package_path=sys.argv[2], options=options))
                assert not repeat.log.updated and len(repeat.log.duplicate) == 2, repeat
                assert card_state(collection) == before
                results.append({"condition": "if_newer" if condition == 0 else "always",
                                "merge_notetypes": merge, "updated": 1, "unchanged": 1,
                                "repeat_updated": 0, "identity_and_review_state_preserved": True})
            finally:
                collection.close()

print(json.dumps({"anki_version": version("anki"), "results": results}, indent=2))
