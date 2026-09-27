"""Fork probes run out of process with a deadline, including native destructors."""
import os
from pathlib import Path
import subprocess
import sys

import pytest

PROBE = r'''
import gc, os, signal, struct, sys, traceback, zlib
from pathlib import Path
from anki_forge import BuildOptions, Content, Field, Media, Note, NoteType, Project, Template
root = Path(os.environ['TMPDIR'])
kind = sys.argv[1]
def chunk(tag, data):
    return struct.pack('>I', len(data)) + tag + data + struct.pack('>I', zlib.crc32(tag + data))
data = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', 1024, 1024, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(b''.join(b'\0' + os.urandom(3072) for _ in range(1024)))) + chunk(b'IEND', b'')
assert len(data) > 1 << 20

def make_owner():
    image = Media.bytes(data, 'image/png')
    if kind == 'media': return image
    content = image.image()
    if kind == 'content': return content
    if kind == 'note': return Note.basic(content, 'answer')
    model = (NoteType.builder('model').field(Field('front')).template(Template('card', '{{front}}', '{{FrontSide}}')).asset(image).build())
    if kind == 'model': return model
    return Project('fork-project').add('n', model.note().field('front', content))

def build(owner):
    if kind == 'media': project = Project('fork-media').add('n', Note.basic(owner.image(), 'answer'))
    elif kind == 'content': project = Project('fork-content').add('n', Note.basic(owner, 'answer'))
    elif kind == 'note': project = Project('fork-note').add('n', owner)
    elif kind == 'model': project = Project('fork-model').add('n', owner.note().field('front', 'question'))
    else: project = owner
    out = project.build(BuildOptions.temporary())
    assert out.report.counts.media == 1

owners = [make_owner()]
gc.collect()
before = set(root.iterdir())
assert len(before) == 1, before
pid = os.fork()
if pid == 0:
    signal.alarm(15)
    try:
        try:
            build(owners[0])
        except RuntimeError as e:
            assert 'BINDING.FORKED_OBJECT' in str(e), str(e)
        else:
            raise AssertionError('inherited owner unexpectedly usable')
        owners.clear()
        gc.collect()
        assert all(p.exists() for p in before), 'child removed parent snapshot'
        fresh = Media.bytes(data, 'image/png')
        assert set(root.iterdir()) - before, 'child reused inherited snapshot cache'
        out = Project('child-created').add('n', Note.basic(fresh.image(), 'answer')).build(BuildOptions.temporary())
        assert out.report.counts.media == 1
        del out, fresh
        gc.collect()
        assert set(root.iterdir()) == before, 'child-owned files were not cleaned'
    except BaseException:
        traceback.print_exc()
        os._exit(1)
    os._exit(0)
_, status = os.waitpid(pid, 0)
assert os.waitstatus_to_exitcode(status) == 0, status
assert all(p.exists() for p in before), 'parent snapshot missing'
build(owners[0])
owners.clear()
gc.collect()
assert not set(root.iterdir()), 'parent-owned files were not cleaned'
'''

@pytest.mark.skipif(not hasattr(os, 'fork'), reason='fork is unavailable')
@pytest.mark.parametrize('kind', ['media', 'content', 'note', 'model', 'project'])
def test_fork_owned_values_do_not_remove_parent_snapshots(tmp_path: Path, kind: str) -> None:
    environment = {**os.environ, 'TMPDIR': str(tmp_path)}
    subprocess.run([sys.executable, '-c', PROBE, kind], env=environment, check=True, timeout=30)
