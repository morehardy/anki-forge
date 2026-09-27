from anki_forge import Media, Content, Note, Project, BuildOptions
# Assets referenced by raw HTML/CSS are declared explicitly; no registry is needed.
style = Media.bytes(b'.card { color: navy; }', 'text/css').with_export_name('theme.css')
project = Project('styled-course').add_asset(style)
project.add('cell', Note.basic(Content.html('<link rel="stylesheet" href="theme.css">Cell'), '细胞'))
output = project.build(BuildOptions.temporary())
assert output.report.counts.media == 1
