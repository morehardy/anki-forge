from pathlib import Path
import sys
from anki_forge import BuildOptions, Note, Project
out = Path(sys.argv[1])
out.parent.mkdir(parents=True, exist_ok=True)
Project('wheel').add('front', Note.basic('Front', 'Back')).build(BuildOptions.to(out))
assert out.is_file()
