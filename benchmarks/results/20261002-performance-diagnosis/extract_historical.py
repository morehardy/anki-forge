from pathlib import Path
import hashlib,json,tarfile
W=Path(__file__).resolve().parent;R=W.parents[2];S=W/'historical-source';S.mkdir()
reference=R/'benchmarks/results/20260921-readme-genanki'
frozen=json.loads((reference/'source-snapshot.json').read_text());identities={}
with tarfile.open(reference/'source-and-inputs.tar.gz') as archive:
    for member in archive.getmembers():
        if not member.isfile(): continue
        if not (member.name.startswith(('source/anki_forge/','source/benchmarks/adapters/rust/')) or member.name in ('source/Cargo.lock','source/Cargo.toml','source/rust-toolchain.toml')): continue
        relative=Path(member.name).relative_to('source');assert '..' not in relative.parts
        data=archive.extractfile(member).read();digest=hashlib.sha256(data).hexdigest();assert frozen['source_files'][str(relative)]==digest
        target=S/relative;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data);identities[str(relative)]=digest
(W/'historical-source-check.json').write_text(json.dumps({'archive_sha256':hashlib.sha256((reference/'source-and-inputs.tar.gz').read_bytes()).hexdigest(),'source_commit':json.loads((reference/'summary.json').read_text())['source_commit'],'files_match_archived_snapshot':identities},indent=2)+'\n')
