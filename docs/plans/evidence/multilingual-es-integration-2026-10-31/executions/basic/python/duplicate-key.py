from ankiforge import Project, Note, BuildOptions

project = Project('controlled-duplicate', default_deck='Recovery')
project.add('source:1', Note.basic('one', 'first'))
try:
    project.add('source:1', Note.basic('two', 'second'))
except Exception as error:
    print(type(error).__name__, str(error))
    assert 'NOTE.KEY_DUPLICATE' in str(error)
else:
    raise AssertionError('The duplicate key must fail')
project.add('source:2', Note.basic('two', 'second'))
output = project.build(BuildOptions.to('recovered.apkg'))
print(output.artifact.path)
