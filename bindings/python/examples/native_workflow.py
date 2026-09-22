"""Run against an installed wheel: python native_workflow.py OUTPUT_DIRECTORY."""
from __future__ import annotations

import base64
import io
import math
import struct
import wave
from pathlib import Path
import sys

from anki_forge import BuildOptions, Deck, Field, Note, NoteType, Project, Template, versions


def tone_wav() -> bytes:
    stream = io.BytesIO()
    with wave.open(stream, "wb") as audio:
        audio.setnchannels(1)
        audio.setsampwidth(2)
        audio.setframerate(8000)
        audio.writeframes(b"".join(
            struct.pack("<h", int(4000 * math.sin(2 * math.pi * 440 * i / 8000)))
            for i in range(8000)
        ))
    return stream.getvalue()


def main(root: Path) -> None:
    root.mkdir(parents=True, exist_ok=True)
    project = Project("Vocabulary", stable_id="vocabulary", base_dir=root)
    project.add_notetype(NoteType.custom("word", name="Word")
        .field(Field("Prompt", key="prompt", identity=True))
        .field(Field("Answer", key="answer"))
        .template(Template("Forward", key="forward", front="{{Prompt}}", back="{{FrontSide}}<hr>{{Answer}}")))
    sound = project.media.add_bytes(source_label="hello", data=tone_wav(), export_as="hello.wav")
    project.add_note(Note("word").text("prompt", "hola").text("answer", "hello"))
    project.add_note(Note.basic("Audio", "", stable_id="audio").sound("back", sound))
    project.validate().ensure_success()
    project.build(BuildOptions(output="baseline.apkg").first_update_safe_build("identity.lock.json")).ensure_success()
    project.diff_against_apkg("baseline.apkg").ensure_success()
    project.build(BuildOptions(output="updated.apkg", compare_to="baseline.apkg", fail_on="high").update_safe("identity.lock.json")).ensure_success()
    with project.build() as report:
        report.ensure_success()
        assert report.artifact is not None
        with report.artifact.persist_to("saved.apkg") as saved:
            assert saved.path.is_file()
    sink = io.BytesIO()
    assert project.write_to(sink) == len(sink.getvalue())
    assert not sink.closed and project.to_apkg_bytes().startswith(b"PK")

    bundle = root / "bundle"
    bundle.mkdir(exist_ok=True)
    (bundle / "anki-template.yaml").write_text("""format_version: template-bundle-v1
note_type:
  id: bundle-card
  fields:
    - {key: prompt, name: Prompt, identity: true}
  templates:
    - {key: card, name: Card, front_file: front.html, back_file: back.html}
""", encoding="utf-8")
    (bundle / "front.html").write_text("{{Prompt}}", encoding="utf-8")
    (bundle / "back.html").write_text("{{FrontSide}}", encoding="utf-8")
    project.import_template_bundle("bundle").add_note(Note("bundle-card").text("prompt", "bundled"))
    project.build().ensure_success()

    deck = Deck("Diagram", stable_id="diagram", base_dir=root)
    png = base64.b64decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScLttAAAAABJRU5ErkJggg==")
    image = deck.media.add_bytes("image.png", png)
    deck.add_image_occlusion(image, rects=[(0, 0, 1, 1)])
    deck.add_cloze("{{c1::Hello}} from Deck")
    Project.from_deck(deck).add_note(Note.basic("Extra", "card")).write_apkg("diagram.apkg").ensure_success()
    print(f"Installed native workflow passed: {versions()}")


if __name__ == "__main__":
    main(Path(sys.argv[1]).absolute())
