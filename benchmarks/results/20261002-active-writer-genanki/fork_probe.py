"""Real fork lifecycle check through freshly built Python bindings; not timed."""
import gc
import hashlib
import io
import json
import os
from pathlib import Path
import signal
import traceback
import zipfile

import zstandard
from ankiforge import BuildOptions, Media, Note, Project

ROOT = Path(os.environ['TMPDIR'])
assert os.fork


def payload(index):
    return hashlib.shake_256(f'round4-fork-{index}'.encode()).digest(512 * 1024)


def create(index):
    return Media.bytes(payload(index), 'application/octet-stream').with_export_name(f'{index}.bin')


def verify(owners, indices, namespace):
    project = Project(namespace).add('note', Note.basic('question', 'answer'))
    for owner in owners:
        project.add_asset(owner)
    output = project.build(BuildOptions.temporary())
    assert output.report.counts.media == len(indices)
    with zipfile.ZipFile(output.artifact.path) as archive:
        identity = json.loads(archive.read('ankiforge-identity.json'))
        assert {int(name.removesuffix('.bin')) for name in identity['media']} == set(indices)
        actual = []
        for name in archive.namelist():
            if name.isdecimal():
                with zstandard.ZstdDecompressor().stream_reader(io.BytesIO(archive.read(name))) as stream:
                    actual.append(hashlib.sha256(stream.read()).digest())
        assert sorted(actual) == sorted(hashlib.sha256(payload(index)).digest() for index in indices)
    output.artifact.close()


def main():
    assert not list(ROOT.iterdir())
    owners = [create(index) for index in range(9)]
    gc.collect()
    before = set(ROOT.iterdir())
    assert len(before) == 1, before
    block = next(iter(before))
    assert block.stat().st_size == 512 * 1024, 'ninth snapshot must use the active spool'
    ready_read, ready_write = os.pipe()
    done_read, done_write = os.pipe()
    pid = os.fork()
    if pid == 0:
        signal.alarm(25)
        os.close(ready_read)
        os.close(done_write)
        try:
            try:
                owners[-1].filename
            except RuntimeError as error:
                assert 'BINDING.FORKED_OBJECT' in str(error)
            else:
                raise AssertionError('inherited native owner was not fenced')
            owners.clear()
            gc.collect()
            assert set(ROOT.iterdir()) == before
            fresh = [create(index) for index in range(100, 109)]
            verify(fresh, range(100, 109), 'round4-child')
            del fresh
            gc.collect()
            assert set(ROOT.iterdir()) == before, 'child snapshots or writer were retained'
            os.write(ready_write, b'1')
            assert os.read(done_read, 1) == b'1'
            assert block.exists(), 'parent storage disappeared while owner remained live'
        except BaseException:
            traceback.print_exc()
            os._exit(1)
        os._exit(0)
    os.close(ready_write)
    os.close(done_read)
    assert os.read(ready_read, 1) == b'1', 'child failed before completing its independent build'
    owners.append(create(9))
    assert block.stat().st_size == 1024 * 1024, 'parent append did not reuse its active writer'
    verify(owners, range(10), 'round4-parent')
    os.write(done_write, b'1')
    _, status = os.waitpid(pid, 0)
    assert os.waitstatus_to_exitcode(status) == 0
    os.close(ready_read)
    os.close(done_write)
    owners.clear()
    gc.collect()
    assert not list(ROOT.iterdir()), 'parent last-owner cleanup retained a block or artifact'
    print(json.dumps({'status':'passed', 'real_fork':True, 'child_media':9, 'parent_media':10,
                      'raw_exported_payloads_checked':19, 'parent_append_after_child_cleanup':True}))


if __name__ == '__main__':
    main()
