from ankiforge import BuildOptions, InspectLimits, Project, Note, Media
InspectLimits(max_entries='unbounded')
Note.basic(7, 'back')
Project('p').add(Note.basic('q', 'a'))
Project('p').add_notetype('model')
Media.bytes('text', 'text/plain')
BuildOptions.temporary().inspect(False)
Project('p', name='removed')
Project('p').name
