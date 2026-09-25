"""End-to-end contracts through the default Rust API, with real APKG inspection."""
from __future__ import annotations
import copy
import gc
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import zipfile
import zlib
import struct
import pytest
from anki_forge import (
    Project, Note, Content, NoteType, Field, Template, GenerationRule, Media, MediaLimits,
    BuildOptions, CompareOptions, UpdatePolicy, InspectLimits, RiskLevel, Mask, OcclusionMode,
    SchemaError, AddError, MediaError, ImageOcclusionError, BuildError, CompareError,
    PolicyError, TemplateBundleError, PersistError,
)

def png(width=10, height=10):
    def chunk(kind, data):
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
    pixels = b''.join(b'\0' + b'\xff\0\0' * width for _ in range(height))
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress(pixels)) + chunk(b'IEND', b''))

ROOT = Path(__file__).resolve().parents[3]
OBSERVER = Path(os.environ.get("ANKI_FORGE_PYTHON_OBSERVER", str(ROOT/'target/debug/examples'/('python_parity.exe' if os.name == 'nt' else 'python_parity'))))

def protobuf(data):
    def varint(pos):
        value = shift = 0
        while True:
            byte = data[pos]; pos += 1
            value |= (byte & 127) << shift
            if byte < 128: return value, pos
            shift += 7
    pos = 0
    result = {}
    while pos < len(data):
        tag, pos = varint(pos)
        if tag & 7 == 0: value, pos = varint(pos)
        elif tag & 7 == 2:
            size, pos = varint(pos)
            value = data[pos:pos+size]; pos += size
        else: raise AssertionError(f'unsupported wire type {tag & 7}')
        result.setdefault(tag >> 3, []).append(value)
    return result

def unpack(path, tmp_path):
    root = tmp_path / ('inspect-' + str(len(list(tmp_path.glob('inspect-*')))))
    subprocess.run([str(OBSERVER), 'extract', str(path), str(root)], check=True)
    entries = protobuf((root/'media').read_bytes()).get(1, [])
    assets = {protobuf(entry)[1][0].decode(): (root/str(i)).read_bytes() for i, entry in enumerate(entries)}
    with zipfile.ZipFile(path) as z:
        identity = json.loads(z.read('ankiforge-identity.json'))['identity']
    connection = sqlite3.connect(root/'collection.anki21b')
    fields = connection.execute('select flds from notes order by guid').fetchall()
    models = {}
    for ntid, name in connection.execute('select id,name from notetypes'):
        templates = [dict(qfmt=protobuf(config)[1][0].decode()) for (config,) in connection.execute('select config from templates where ntid=? order by ord', (ntid,))]
        models[str(ntid)] = dict(name=name, tmpls=templates)
    cards = connection.execute('select nid,ord from cards order by nid,ord').fetchall()
    connection.close()
    return fields, models, assets, identity, cards

def custom(name='词汇'):
    return (NoteType.builder('vocab').name(name)
        .field(Field('front', name='正面'))
        .field(Field('back', name='背面'))
        .template(Template('recognition', '{{front}}', '{{FrontSide}}<hr>{{back}}', name='识别'))
        .build())

def test_basic_content_and_artifact_lifetime(tmp_path):
    output = Project('basic').add('term', Note.basic('<front>', Content.html('<b>back</b>'))).build(BuildOptions.temporary())
    assert output.report.counts.notes == output.report.counts.cards == 1
    assert output.snapshot()['result']['status'] == 'success'
    assert 'result' not in output.report.snapshot()
    fields, _, _, _, _ = unpack(output.artifact.path, tmp_path)
    assert fields == [('&lt;front&gt;\x1f<b>back</b>',)]
    snapshot = output.snapshot()
    path = output.artifact.path
    clone = copy.copy(output.artifact)
    output.artifact.close()
    assert path.exists()
    clone.close()
    assert not path.exists()
    assert snapshot['result']['artifact'] == str(path)

def test_snapshot_and_report_do_not_retain_temporary_file():
    output = Project('lifetime').add('n', Note.basic('q', 'a')).build(BuildOptions.temporary())
    report, snapshot, path = output.report, output.snapshot(), output.artifact.path
    del output
    gc.collect()
    assert not path.exists()
    assert report.counts.notes == 1
    assert snapshot['result']['status'] == 'success'

def test_custom_model_keys_compile_to_labels_and_values_are_immutable(tmp_path):
    model = custom()
    first = model.note().field('front', '<front>').field('back', Content.html('<b>back</b>'))
    second = first.field('back', 'other')
    project = Project('custom').add('n', first)
    fields, models, _, _, _ = unpack(project.build(BuildOptions.to(tmp_path/'custom.apkg')).artifact.path, tmp_path)
    assert fields[0][0] == '&lt;front&gt;\x1f<b>back</b>'
    assert next(iter(models.values()))['tmpls'][0]['qfmt'] == '{{正面}}'
    assert second.note_type.key == 'vocab'
    with pytest.raises(AttributeError):
        model._handle = first._handle
    with pytest.raises(AddError) as error:
        project.add('other', custom('Different').note().field('front', 'x'))
    assert error.value.code == 'NOTE.MODEL_CONFLICT'
    assert len(project) == 1

@pytest.mark.parametrize('key', ['', ' ', ' bad '])
def test_explicit_schema_and_note_keys_reject_invalid_values(key):
    with pytest.raises(SchemaError):
        NoteType.builder('vocab').field(Field(key)).template(Template('t', 'q', 'a')).build()
    project = Project('keys')
    with pytest.raises(AddError):
        project.add(key, Note.basic('q', 'a'))
    assert len(project) == 0

def test_invalid_namespace_and_duplicate_note_key_are_structured():
    with pytest.raises(SchemaError) as error:
        Project('')
    assert error.value.code == 'SCHEMA.NAMESPACE_INVALID'
    project = Project('keys').add('n', Note.basic('q', 'a'))
    with pytest.raises(AddError) as error:
        project.add('n', Note.basic('changed', 'a'))
    assert error.value.code == 'NOTE.KEY_DUPLICATE'
    assert len(project) == 1

def test_field_lookup_only_accepts_stable_key():
    project = Project('field')
    with pytest.raises(AddError) as error:
        project.add('n', custom().note().field('正面', 'q'))
    assert error.value.code == 'NOTE.FIELD_UNKNOWN'
    assert len(project) == 0

def test_cloze_strings_are_text(tmp_path):
    output = Project('cloze').add('n', Note.cloze('{{c1::<b>word</b>}}')).build(BuildOptions.temporary())
    fields, _, _, _, cards = unpack(output.artifact.path, tmp_path)
    assert '{{c1::&lt;b&gt;word&lt;/b&gt;}}' in fields[0][0]
    assert len(cards) == 1

def test_media_snapshot_sequence_and_rename_ownership(tmp_path):
    source = tmp_path/'image.png'
    original = png()
    source.write_bytes(original)
    media = Media.file(source)
    first = media.image()
    named = media.with_export_name('cell.png')
    source.write_bytes(b'changed')
    source.unlink()
    project = Project('media').add('n', Note.basic(Content.sequence(['<x>', first, named.image()]), 'a'))
    output = project.build(BuildOptions.temporary())
    fields, _, assets, _, _ = unpack(output.artifact.path, tmp_path)
    assert assets == {media.filename: original, 'cell.png': original}
    assert '&lt;x&gt;' in fields[0][0]
    assert output.report.counts.media == 2
    assert Project('reuse').add('n', Note.basic(named.image(), 'a')).build(BuildOptions.temporary()).report.counts.media == 1

def test_large_bytes_asset_and_media_budget(tmp_path):
    data = b'/* ' + b'x' * 70000 + b' */'
    asset = Media.bytes(data, 'text/css').with_export_name('style.css')
    project = Project('bytes').add('n', Note.basic('q', 'a')).add_asset(asset)
    output = project.build(BuildOptions.temporary())
    _, _, assets, _, _ = unpack(output.artifact.path, tmp_path)
    assert assets['style.css'] == data
    with pytest.raises(MediaError) as error:
        Media.bytes(data, 'text/css', limits=MediaLimits(max_bytes=100))
    assert error.value.code == 'MEDIA.RESOURCE_LIMIT_EXCEEDED'
    assert error.value.details['limit_exceeded']['observed'] == len(data)

@pytest.mark.parametrize('budget', ['small', 'default'])
def test_media_budget_rejects_before_binding_copy(budget):
    result = subprocess.run(
        [sys.executable, '-I', '-X', 'utf8', str(Path(__file__).with_name('media_budget_probe.py')), budget],
        text=True, capture_output=True, timeout=30,
    )
    assert result.returncode == 0, result.stdout + result.stderr

def test_media_byte_budget_boundaries_and_input_ownership(tmp_path):
    data = b'body { color: navy; }'
    asset = Media.bytes(data, 'text/css', limits=MediaLimits(len(data))).with_export_name('style.css')
    del data
    gc.collect()
    output = Project('bytes-owner').add_asset(asset).add('n', Note.basic('q', 'a')).build(BuildOptions.temporary())
    _, _, assets, _, _ = unpack(output.artifact.path, tmp_path)
    assert assets['style.css'] == b'body { color: navy; }'
    assert len(Media.bytes(b'', 'application/octet-stream', limits=MediaLimits(0))) == 0
    with pytest.raises(MediaError) as error:
        Media.bytes(b'x', 'application/octet-stream', limits=MediaLimits(0))
    assert error.value.details['limit_exceeded'] == {'resource': 'media_bytes', 'limit': 0, 'observed': 1}

@pytest.mark.parametrize('limit,error_type', [(-1, OverflowError), (1 << 64, OverflowError), (1.5, TypeError), ('1', TypeError)])
def test_media_byte_budget_validates_native_u64(limit, error_type):
    with pytest.raises(error_type):
        Media.bytes(b'x', 'application/octet-stream', limits=MediaLimits(limit))

@pytest.mark.parametrize('name', ['../x.png', 'CON', 'bad/name', 'trailing.'])
def test_media_names_fail_at_rename(name):
    with pytest.raises(MediaError) as error:
        Media.bytes(png(), 'image/png').with_export_name(name)
    assert error.value.code == 'MEDIA.EXPORT_NAME_INVALID'

def test_media_mime_and_name_conflicts_are_atomic():
    with pytest.raises(MediaError) as error:
        Media.bytes(png(), 'audio/wav')
    assert error.value.code == 'MEDIA.TYPE_MISMATCH'
    p = Project('assets').add_asset(Media.bytes(png(), 'image/png').with_export_name('Cell.png'))
    with pytest.raises(AddError):
        p.add('n', Note.basic(Media.bytes(png(), 'image/png').with_export_name('cell.png').image(), 'a'))
    assert len(p) == 0
    p.add('n', Note.basic('q', 'a'))
    assert p.build(BuildOptions.temporary()).report.counts.media == 1

def test_media_bytes_share_container_matching_and_canonical_names(tmp_path):
    cases = [
        (png(), 'IMAGE/PNG', 'image/png', '.png'),
        (bytes.fromhex('1a45dfa37765626d'), 'AUDIO/WEBM; codecs=Opus', 'audio/webm; codecs=Opus', '.webm'),
        (b'OggSOpusHead', 'AUDIO/OPUS', 'audio/opus', '.opus'),
        (bytes.fromhex('000000186674797069736f6d'), 'audio/mp4', 'audio/mp4', '.m4a'),
    ]
    project = Project('mime-containers').add('one', Note.basic('q', 'a'))
    expected = {}
    for data, declared, canonical, extension in cases:
        media = Media.bytes(data, declared)
        lower = Media.bytes(data, canonical)
        assert media.media_type == canonical
        assert media.filename == lower.filename
        assert media.filename.endswith(extension)
        expected[media.filename] = data
        project.add_asset(media)
    output = project.build(BuildOptions.temporary())
    _, _, assets, _, _ = unpack(output.artifact.path, tmp_path)
    assert assets == expected
    assert output.report.counts.media == len(cases)

@pytest.mark.parametrize('mode', list(OcclusionMode))
def test_io_masks_create_distinct_cards_and_preserve_structure(tmp_path, mode):
    note = (Note.image_occlusion(Media.bytes(png(), 'image/png')).mode(mode)
            .mask(Mask.rect('left', 0, 0, 3, 3)).mask(Mask.rect('right', 4, 4, 3, 3)).build()
            .field('header', '<heading>'))
    output = Project('io').add('n', note).build(BuildOptions.temporary())
    fields, _, _, _, cards = unpack(output.artifact.path, tmp_path)
    assert len(cards) == output.report.counts.cards == 2
    assert 'c1::image-occlusion' in fields[0][0]
    assert 'c2::image-occlusion' in fields[0][0]
    assert ('oi=1' in fields[0][0]) == (mode == OcclusionMode.HIDE_ALL_GUESS_ONE)
    with pytest.raises(AddError) as error:
        Project('reserved').add('n', note.field('image', 'oops'))
    assert error.value.code == 'NOTE.IO_FIELD_RESERVED'

def test_io_validation_happens_at_builder_completion():
    image = Media.bytes(png(), 'image/png')
    with pytest.raises(ImageOcclusionError) as error:
        Note.image_occlusion(image).mask(Mask.rect('n', 8, 8, 4, 4)).build()
    assert error.value.code == 'NOTE.IO_RECT_INVALID'
    with pytest.raises(ImageOcclusionError) as error:
        Note.image_occlusion(image).mask(Mask.rect('n', 0, 0, 1, 1)).mask(Mask.rect('n', 2, 2, 1, 1)).build()
    assert error.value.code == 'NOTE.IO_MASK_KEY_DUPLICATE'

def test_update_comparison_policy_and_identity(tmp_path):
    baseline = tmp_path/'v1.apkg'
    Project('updates').add('keep', Note.basic('q', 'a')).add('remove', Note.basic('r', 'a')).build(BuildOptions.to(baseline))
    candidate = Project('updates').add('keep', Note.basic('changed', 'a'))
    comparison = candidate.compare(CompareOptions.against(baseline))
    assert not comparison.allows_publication
    assert any(f['code'] == 'RISK.NOTE_REMOVED' for f in comparison.findings)
    target = tmp_path/'v2.apkg'
    with pytest.raises(BuildError) as error:
        candidate.build(BuildOptions.to(target).update_from(baseline))
    assert not target.exists()
    assert error.value.snapshot()['result']['status'] == 'failure'
    assert error.value.report.comparison is not None
    policy = UpdatePolicy().allow('RISK.NOTE_REMOVED')
    allowed = candidate.compare(CompareOptions.against(baseline).update_policy(policy))
    assert allowed.allows_publication
    output = candidate.build(BuildOptions.to(target).update_policy(policy).update_from(baseline))
    assert output.report.comparison.policy == allowed.policy
    before = unpack(baseline, tmp_path)[3]
    after = unpack(target, tmp_path)[3]
    assert before['notes']['keep']['guid'] == after['notes']['keep']['guid']

def test_update_configuration_unknown_risk_and_hard_errors(tmp_path):
    with pytest.raises(PolicyError) as error:
        UpdatePolicy().allow('RISK.MADE_UP')
    assert error.value.code == 'UPDATE.RISK_CODE_INVALID'
    p = Project('config').add('n', Note.basic('q', 'a'))
    with pytest.raises(BuildError) as error:
        p.build(BuildOptions.temporary().update_policy(UpdatePolicy()))
    assert error.value.kind == 'Configuration'
    with pytest.raises(CompareError):
        p.compare(CompareOptions.against(tmp_path/'missing.apkg').update_policy(UpdatePolicy().fail_on(RiskLevel.CRITICAL)))

def test_warning_does_not_change_success(tmp_path):
    p = Project('warning').add('n', Note.basic('q', 'a'))
    baseline = tmp_path/'v1.apkg'
    p.build(BuildOptions.to(baseline))
    output = p.build(BuildOptions.temporary().update_from(baseline).update_policy(UpdatePolicy().allow('RISK.MASK_REMOVED')))
    assert output.snapshot()['result']['status'] == 'success'
    assert output.report.diagnostics

def test_inspection_budget_applies_to_candidate_and_baseline(tmp_path):
    p = Project('limits').add('n', Note.basic('q', 'a'))
    with pytest.raises(BuildError) as error:
        p.build(BuildOptions.temporary().inspect_limits(InspectLimits(max_archive_bytes=1)))
    assert error.value.kind == 'ResourceLimit'
    baseline = tmp_path/'v1.apkg'
    p.build(BuildOptions.to(baseline))
    with pytest.raises(CompareError) as error:
        p.compare(CompareOptions.against(baseline).inspect_limits(InspectLimits(max_archive_bytes=1)))
    assert error.value.kind == 'ResourceLimit'

def test_persist_failure_keeps_source_and_publication_facts(tmp_path):
    output = Project('persist').add('n', Note.basic('q', 'a')).build(BuildOptions.temporary())
    path = output.artifact.path
    with pytest.raises(PersistError) as error:
        output.artifact.persist_to(tmp_path)
    assert error.value.details['publication']['stage'] == 'not_published'
    assert path.exists()
    permanent = output.artifact.persist_to(tmp_path/'saved.apkg')
    output.artifact.close()
    assert permanent.path.exists()

def test_removed_api_is_absent():
    import anki_forge
    for name in ['Deck', 'MediaRegistry', 'MediaRef', 'IdentityRecipe', 'UpdateSafetyMode']:
        assert not hasattr(anki_forge, name)
    assert not hasattr(Project, 'add_notetype')
    assert not hasattr(Project, 'add_note')

@pytest.mark.parametrize('scenario', ['basic', 'custom'])
def test_python_matches_independent_default_rust_producer(tmp_path, scenario):
    rust = tmp_path/'rust.apkg'
    subprocess.run([str(OBSERVER), scenario, str(rust)], check=True, capture_output=True)
    if scenario == 'basic':
        note = Note.basic('<front>', Content.html('<b>back</b>'))
    else:
        note = custom().note().field('front', '<front>').field('back', Content.html('<b>back</b>'))
    python = Project('python-parity').add('term', note).build(BuildOptions.to(tmp_path/'python.apkg'))
    actual = unpack(python.artifact.path, tmp_path)
    expected = unpack(rust, tmp_path)
    assert actual[:3] == expected[:3]
    # Build time is intentionally observational rather than part of identity.
    for value in [actual[3], expected[3]]:
        for model in value['models'].values(): model.pop('mtime_secs')
        for note in value['notes'].values(): note.pop('mtime_secs')
    assert actual[3] == expected[3]
    assert [ordinal for _, ordinal in actual[4]] == [ordinal for _, ordinal in expected[4]]

def bundle(root):
    root.mkdir()
    (root/'front.html').write_text('{{front}}', encoding='utf-8')
    (root/'back.html').write_text('{{FrontSide}}<link rel="stylesheet" href="theme.css">', encoding='utf-8')
    (root/'theme.css').write_bytes(b'.card {color: navy;}')
    (root/'anki-template.yaml').write_text('''format_version: template-bundle-v2
note_type:
  key: bundled
  name: 词汇
  fields:
    - key: front
      name: 正面
  templates:
    - key: card
      front_file: front.html
      back_file: back.html
assets:
  - path: theme.css
    export_as: theme.css
''', encoding='utf-8')

def test_bundle_owns_complete_asset_closure(tmp_path):
    import shutil
    root = tmp_path/'bundle'
    bundle(root)
    model = NoteType.from_bundle(root)
    shutil.rmtree(root)
    output = Project('bundle').add('n', model.note().field('front', '<q>')).build(BuildOptions.temporary())
    fields, models, assets, _, _ = unpack(output.artifact.path, tmp_path)
    assert fields == [('&lt;q&gt;',)]
    assert assets == {'theme.css': b'.card {color: navy;}'}
    assert next(iter(models.values()))['tmpls'][0]['qfmt'] == '{{正面}}'

def test_bundle_errors_keep_path_and_source(tmp_path):
    root = tmp_path/'bundle'
    bundle(root)
    (root/'front.html').write_bytes(b'\xff')
    with pytest.raises(TemplateBundleError) as error:
        NoteType.from_bundle(root)
    assert error.value.details['path'].endswith('front.html')
    assert error.value.causes

def test_required_fields_generation_and_custom_cloze(tmp_path):
    model = (NoteType.builder('generation').field(Field('front', required=True, sort=True))
             .field(Field('back')).template(Template('forward', '{{front}}', '{{back}}',
                 generation=GenerationRule.all(['front'])))
             .template(Template('reverse', '{{back}}', '{{front}}', generation=GenerationRule.any(['back'])))
             .build())
    p = Project('generation')
    with pytest.raises(AddError) as error:
        p.add('n', model.note().field('back', 'a'))
    assert error.value.code == 'NOTE.FIELD_REQUIRED'
    p.add('n', model.note().field('front', 'q'))
    assert p.build(BuildOptions.temporary()).report.counts.cards == 1
    cloze = (NoteType.builder('custom-cloze').field(Field('text', name='文本'))
             .cloze_field('text').template(Template('cloze', '{{cloze:text}}', '{{cloze:text}}')).build())
    output = Project('cloze-model').add('n', cloze.note().field('text', '{{c2::word}}')).build(BuildOptions.temporary())
    assert output.report.counts.cards == 1

def test_model_assets_and_failed_add_do_not_register_partial_assets():
    a = Media.bytes(png(), 'image/png').with_export_name('asset.png')
    b = Media.bytes(png(12, 12), 'image/png').with_export_name('asset.png')
    builder = NoteType.builder('assets').field(Field('front')).template(Template('t', '{{front}}', '{{FrontSide}}'))
    with pytest.raises(SchemaError):
        builder.asset(a).asset(b).build()
    model = builder.asset(a).build()
    p = Project('atomic').add_asset(b)
    with pytest.raises(AddError):
        p.add('n', model.note().field('front', 'q'))
    p.add('n', Note.basic('q', 'a'))
    assert p.build(BuildOptions.temporary()).report.counts.media == 1

def test_compare_missing_evidence_is_hard_error(tmp_path):
    p = Project('evidence').add('n', Note.basic('q', 'a'))
    source = p.build(BuildOptions.temporary())
    damaged = tmp_path/'damaged.apkg'
    with zipfile.ZipFile(source.artifact.path) as z, zipfile.ZipFile(damaged, 'w') as out:
        for name in z.namelist():
            if name != 'ankiforge-identity.json': out.writestr(name, z.read(name))
    with pytest.raises(CompareError) as error:
        p.compare(CompareOptions.against(damaged).update_policy(UpdatePolicy().fail_on(RiskLevel.CRITICAL)))
    assert error.value.kind == 'Validation'
    assert 'EVIDENCE' in error.value.code

@pytest.mark.skipif(not hasattr(os, 'fork'), reason='requires fork')
def test_inherited_project_and_artifact_fail_without_locking():
    p = Project('fork').add('n', Note.basic('q', 'a'))
    output = p.build(BuildOptions.temporary())
    pid = os.fork()
    if pid == 0:
        try:
            for operation in [lambda: len(p), lambda: output.artifact.path]:
                try: operation()
                except RuntimeError as error: assert 'BINDING.FORKED_OBJECT' in str(error)
                else: os._exit(2)
            output.artifact = None
            gc.collect()
            os._exit(0)
        except BaseException:
            os._exit(3)
    _, status = os.waitpid(pid, 0)
    assert os.waitstatus_to_exitcode(status) == 0
    assert output.artifact.path.exists()

def test_concurrent_independent_projects_build_with_gil_released():
    from concurrent.futures import ThreadPoolExecutor
    def build(number):
        return Project(f'thread-{number}').add('n', Note.basic('q', 'a')).build(BuildOptions.temporary())
    with ThreadPoolExecutor(max_workers=4) as pool:
        outputs = list(pool.map(build, range(8)))
    assert len({o.artifact.path for o in outputs}) == 8
    assert all(o.report.counts.notes == 1 for o in outputs)

@pytest.mark.parametrize('coordinate', [float('nan'), float('inf'), -float('inf'), -1.0, 11.0])
def test_io_nonfinite_and_out_of_bounds_use_core_build_error(coordinate):
    builder = Note.image_occlusion(Media.bytes(png(), 'image/png')).mask(Mask.rect('n', coordinate, 0, 1, 1))
    with pytest.raises(ImageOcclusionError) as error:
        builder.build()
    assert error.value.code == 'NOTE.IO_RECT_INVALID'
    assert error.value.details['error_kind'] == 'InvalidMask'

@pytest.mark.parametrize('input_name,limit', [('anki-template.yaml', 256 << 10), ('front.html', 2 << 20)])
def test_bundle_text_budgets_keep_typed_source(tmp_path, input_name, limit):
    root = tmp_path / 'bundle'
    bundle(root)
    (root / input_name).write_bytes(b' ' * (limit + 1))
    with pytest.raises(TemplateBundleError) as error:
        NoteType.from_bundle(root)
    assert error.value.details['error_kind'] == 'ResourceLimit'
    assert {'type': 'bundle_limit', 'limit': limit, 'observed': limit + 1} in error.value.source_details

def test_bundle_media_budget_can_be_raised_and_keeps_typed_source(tmp_path):
    root = tmp_path / 'bundle'
    bundle(root)
    data = png()
    (root / 'picture.png').write_bytes(data)
    with (root / 'anki-template.yaml').open('a') as f:
        f.write('  - path: picture.png\n    export_as: picture.png\n')
    with pytest.raises(TemplateBundleError) as error:
        NoteType.from_bundle(root, limits=MediaLimits(len(data) - 1))
    sources = [s for s in error.value.source_details if s['type'] == 'media']
    assert sources[0]['code'] == 'MEDIA.RESOURCE_LIMIT_EXCEEDED'
    assert sources[0]['limit_exceeded'] == {'resource': 'media_bytes', 'limit': len(data) - 1, 'observed': len(data)}
    model = NoteType.from_bundle(root, limits=MediaLimits(len(data)))
    out = Project('raised-bundle-budget').add('n', model.note().field('front', 'q')).build(BuildOptions.temporary())
    assert unpack(out.artifact.path, tmp_path)[2]['picture.png'] == data
