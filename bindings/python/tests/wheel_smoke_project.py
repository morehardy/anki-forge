from pathlib import Path
import sys
from ankiforge import BuildOptions, Note, Project
out = Path(sys.argv[1])
out.parent.mkdir(parents=True, exist_ok=True)
Project('wheel').add('front', Note.basic('Front', 'Back')).build(BuildOptions.to(out))
assert out.is_file()

prepared = Project('wheel-prepared').add('one', Note.basic('q', 'a')).prepare_publication(BuildOptions.temporary())
assert prepared.report.counts.notes == 1
published = prepared.publish()
owned = published.artifact.path
assert owned.is_file()
published.artifact.close()
assert not owned.exists()
prepared.close()
