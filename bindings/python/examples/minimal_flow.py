from ankiforge import Project, Note, BuildOptions
project = Project('biology-course', default_deck='Biology')
project.add('cell', Note.basic('What is a cell?', 'A unit of life'))
output = project.build(BuildOptions.temporary())
assert output.report.counts.notes == 1
