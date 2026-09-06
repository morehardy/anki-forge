"""Public genanki workflow. All preparation below is inside each fresh process."""
import html
import json
import sys

import genanki


def main():
    if sys.argv[1:] == ["--metadata"]:
        import importlib.metadata
        import platform
        print(json.dumps({
            "protocol": "basic-apkg-v1", "adapter": "genanki/python",
            "genanki": importlib.metadata.version("genanki"),
            "python": platform.python_version(), "architecture": platform.machine(),
            "process_scope": "single_process", "model": "genanki.BASIC_MODEL",
        }))
        return
    input_path, output_path = sys.argv[1:]
    with open(input_path, encoding="utf-8") as stream:
        workload = json.load(stream)
    if workload["schema"] != "basic-apkg-v1" or len(workload["notes"]) != workload["note_count"]:
        raise ValueError("invalid workload")
    deck = genanki.Deck(workload["genanki_deck_id"], workload["deck_name"])
    for note in workload["notes"]:
        deck.add_note(genanki.Note(
            model=genanki.BASIC_MODEL,
            fields=[html.escape(note["front"]), html.escape(note["back"])],
        ))
    genanki.Package(deck).write_to_file(output_path)


if __name__ == "__main__":
    main()
