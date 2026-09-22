"""Generate real image/audio assets: python docs_media.py [OUTPUT_DIRECTORY]."""
from pathlib import Path
import math
import struct
import sys
import wave

from anki_forge import Note, Project

output = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
output.mkdir(parents=True, exist_ok=True)
(output / "diagram.svg").write_text(
    '<svg xmlns="http://www.w3.org/2000/svg" width="240" height="120">'
    '<rect width="240" height="120" fill="#edf3f1"/>'
    '<circle cx="120" cy="60" r="35" fill="#116e60"/></svg>',
    encoding="utf-8",
)
with wave.open(str(output / "tone.wav"), "wb") as audio:
    audio.setnchannels(1)
    audio.setsampwidth(2)
    audio.setframerate(8000)
    audio.writeframes(b"".join(
        struct.pack("<h", int(4000 * math.sin(2 * math.pi * 440 * i / 8000)))
        for i in range(8000)
    ))

project = Project("Media", stable_id="docs-media", base_dir=output)
picture = project.media.add_file("diagram.svg", export_as="diagram.svg")
sound = project.media.add_file("tone.wav", export_as="tone.wav")
project.add_note(Note.basic("Reveal a green circle", "", stable_id="media:circle").image("back", picture))
project.add_note(Note.basic("Play A4 (440 Hz)", "", stable_id="media:tone").sound("back", sound))
report = project.write_apkg("media.apkg")
report.ensure_success()
assert report.counts["notes"] == 2 and report.counts["cards"] == 2 and report.counts["media"] == 2
print(output / "media.apkg")
