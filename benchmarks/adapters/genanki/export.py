"""Public genanki workflow. All preparation below is inside each fresh process."""
import html
import json
import sys
from pathlib import Path

import genanki


def main():
    if sys.argv[1:] == ["--metadata"]:
        import importlib.metadata
        import platform
        print(json.dumps({
            "protocol": "basic-apkg-v1", "adapter": "genanki/python",
            "protocols": ["basic-apkg-v1", "basic-media-apkg-v1"],
            "genanki": importlib.metadata.version("genanki"),
            "python": platform.python_version(), "architecture": platform.machine(),
            "process_scope": "single_process", "model": "genanki.BASIC_MODEL",
        }))
        return
    input_path, output_path = sys.argv[1:]
    with open(input_path, encoding="utf-8") as stream:
        workload = json.load(stream)
    if workload["schema"] not in ("basic-apkg-v1", "basic-media-apkg-v1") or len(workload["notes"]) != workload["note_count"]:
        raise ValueError("invalid workload")
    deck = genanki.Deck(workload["genanki_deck_id"], workload["deck_name"])
    media = {item["id"]: item for item in workload.get("media", [])}
    if len(media) != len(workload.get("media", [])):
        raise ValueError("duplicate media id")

    def field(text, refs):
        result = html.escape(text)
        for ref in refs:
            item = media[ref]
            if item["kind"] == "image":
                result += '\n<img src="' + html.escape(item["filename"], quote=True) + '">'
            elif item["kind"] == "audio":
                result += '\n[sound:' + item["filename"] + ']'
            else:
                raise ValueError("unsupported media kind")
        return result
    for note in workload["notes"]:
        deck.add_note(genanki.Note(
            model=genanki.BASIC_MODEL,
            fields=[field(note["front"], note.get("front_media", [])),
                    field(note["back"], note.get("back_media", []))],
        ))
    package = genanki.Package(deck)
    package.media_files = []
    for item in media.values():
        path = Path(input_path).parent / item["path"]
        if path.name != item["filename"]:
            raise ValueError("media filename mismatch")
        package.media_files.append(str(path))
    package.write_to_file(output_path)


if __name__ == "__main__":
    main()
