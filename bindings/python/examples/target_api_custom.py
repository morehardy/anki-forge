from anki_forge import Field, Template, NoteType, Project, BuildOptions
model = (NoteType.builder('vocab').name('词汇')
    .field(Field('front', name='正面', required=True))
    .field(Field('back', name='背面'))
    .template(Template('recognition', '{{front}}', '{{FrontSide}}<hr>{{back}}', name='识别'))
    .build())
project = Project('vocab-course')
project.add('cell', model.note().field('front', 'cell').field('back', '细胞'))
output = project.build(BuildOptions.temporary())
assert output.report.counts.cards == 1
