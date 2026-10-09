from ankiforge import (Project, Note, NoteType, Field, Template, Content, Media,
                        BuildOptions, CompareOptions, UpdatePolicy, BuildOutput)
model = NoteType.builder('vocab').field(Field('front')).template(Template('card', '{{front}}', '{{FrontSide}}')).build()
media = Media.bytes(b'.card{}', 'text/css').with_export_name('style.css')
project = Project('typed').add_asset(media).add('n', model.note().field('front', Content.html('<b>front</b>')))
output: BuildOutput = project.build(BuildOptions.temporary())
output.artifact.persist_to('out.apkg')
project.compare(CompareOptions.against('out.apkg').update_policy(UpdatePolicy().allow('RISK.NOTE_CHANGED')))
Note.cloze('{{c1::word}}').field('back_extra', 'extra')

from ankiforge import MediaLimits
budgeted_model = NoteType.from_bundle("bundle", limits=MediaLimits(max_bytes=512 << 20))

from ankiforge import AddError, AddContext, AddDetail

def read_add_error(error: AddError) -> str | None:
    context: AddContext = error.details['context']
    target = context['target']
    if target['type'] == 'field':
        path: list[int] | None = target['content_path']
        key: str = target['field_key']
    detail: AddDetail | None = error.details['detail']
    if detail is not None and detail['type'] == 'media_usage':
        return detail['media_type']
    return context['note_key']

from ankiforge import PreparedPublication, BuildReport
prepared: PreparedPublication = project.prepare_publication(BuildOptions.temporary())
reviewed: BuildReport = prepared.report
prepared.close()
