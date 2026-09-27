"""Create and update an original distribution using owned media and stable keys."""
from pathlib import Path
import sys
from anki_forge import BuildOptions, CompareOptions, Content, Media, Note, Project

root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('output')
root.mkdir(parents=True, exist_ok=True)
style = Media.bytes(b'.card { color: navy; }', 'text/css').with_export_name('theme.css')
first = Project('biology-course', name='Biology', default_deck='Science::Biology')
first.add_asset(style).add('cell', Note.basic('What is a cell?', 'A unit of life'))
first.build(BuildOptions.to(root/'v1.apkg'))
next_project = Project('biology-course', name='Biology', default_deck='Science::Biology')
next_project.add_asset(style).add('cell', Note.basic('What is a cell?', Content.html('<b>The basic unit of life</b>')))
comparison = next_project.compare(CompareOptions.against(root/'v1.apkg'))
assert comparison.allows_publication
output = next_project.build(BuildOptions.to(root/'v2.apkg').update_from(root/'v1.apkg'))
print(output.artifact.path)
