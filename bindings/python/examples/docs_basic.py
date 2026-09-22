"""python docs_basic.py [OUTPUT_DIRECTORY]"""
from pathlib import Path
import sys

from anki_forge import Note, Project

output = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
output.mkdir(parents=True, exist_ok=True)
project = Project("Spanish", stable_id="docs-spanish", default_deck="Spanish", base_dir=output)
project.add_note(Note.basic("hola", "hello", stable_id="es:hola"))
project.validate().ensure_success()
report = project.write_apkg("spanish.apkg")
report.ensure_success()
assert report.counts["notes"] == 1 and report.counts["cards"] == 1
print(output / "spanish.apkg")
