#!/usr/bin/env python3
"""Build private instrumented adapters and compare retained Deck allocations.

Reuses the repository's existing diagnostic System allocator counter, only in a
scratch crate. Production native code, core, and platform packages are untouched.
The control changes only BuildTask back to its prior persistent Project snapshot.
Rust live bytes exclude V8 and libraries allocating through their own C allocator.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

repo = Path(__file__).resolve().parents[3]
node_root = repo / 'bindings/node'
parser = argparse.ArgumentParser()
parser.add_argument('--node', required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
patch = (repo / 'benchmarks/results/20260907-post-streaming-audit/probes/memory.patch').read_text()
section = patch.split('+++ b/benchmarks/adapters/rust/src/memory.rs\n', 1)[1]
counter = '\n'.join(line[1:] for line in section.splitlines() if line.startswith('+'))
counter = counter[:counter.index('pub fn observe')]
counter += '''
#[napi_derive::napi]
pub fn allocation_stats() -> String {
    serde_json::json!({"liveRustBytes": LIVE.load(Relaxed),
        "peakRustBytes": PEAK.load(Relaxed), "allocationCalls": COUNT.load(Relaxed)}).to_string()
}
'''
source = (node_root / 'native/src/state.rs').read_text()
start = source.index('            let result = match &context.deck {', source.index('impl Task for BuildTask'))
end = source.index('            crate::artifact::BuildOutcome', start)
control = source[:start] + '''            if let Some(deck) = &context.deck {
                context.project = Project::from(deck.clone());
            }
            let result = context.project.build(self.options.clone());
''' + source[end:]
manifest = (node_root / 'native/Cargo.toml').read_text().replace('edition.workspace = true', 'edition = "2021"').replace('rust-version.workspace = true', 'rust-version = "1.92"')
manifest = manifest.replace('path = "../../../anki_forge"', f'path = {json.dumps(str(repo / "anki_forge"))}')
manifest += '\n[workspace]\n'
env = {**os.environ, 'CARGO_TARGET_DIR': str(repo / 'target'), 'ANKI_FORGE_NODE_TARGET': subprocess.check_output(['rustc', '-vV'], text=True).split('host: ')[1].splitlines()[0]}
with tempfile.TemporaryDirectory(prefix='anki-forge-node-probe-') as directory:
    scratch = Path(directory)
    shutil.copytree(node_root / 'native', scratch / 'native')
    crate = scratch / 'native'
    (crate / 'Cargo.toml').write_text(manifest)
    shutil.copyfile(repo / 'Cargo.lock', crate / 'Cargo.lock')
    (crate / 'src/memory_probe.rs').write_text('#![allow(unsafe_code)]\n' + counter)
    with (crate / 'src/lib.rs').open('a') as file:
        file.write('\nmod memory_probe;\n')
    for name, state in [('persistent-project-control', control), ('direct-deck', source)]:
        (crate / 'src/state.rs').write_text(state)
        build = subprocess.run(['cargo', 'build', '--manifest-path', str(crate / 'Cargo.toml'), '--offline', '--message-format=json'], env=env, text=True, stdout=subprocess.PIPE, check=True)
        records = [json.loads(line) for line in build.stdout.splitlines()]
        artifact = next(item for item in records if item.get('target', {}).get('name') == 'anki_forge_node_native' and item.get('reason') == 'compiler-artifact')
        binary = next(Path(file) for file in artifact['filenames'] if Path(file).suffix in ['.dylib', '.so', '.dll'])
        addon = scratch / f'{name}.node'
        shutil.copyfile(binary, addon)
        run = subprocess.run([args.node, '--expose-gc', str(node_root / 'scripts/profile-resources.mjs'), 'deck'], env={**env, 'ANKI_FORGE_NATIVE_PATH': str(addon)}, text=True, stdout=subprocess.PIPE, check=True)
        data = json.loads(run.stdout)
        data['comparison'] = name
        data['instrumentedStateSha256'] = hashlib.sha256(state.encode()).hexdigest()
        data['counterSource'] = 'benchmarks/results/20260907-post-streaming-audit/probes/memory.patch'
        (args.output / f'{name}.json').write_text(json.dumps(data, indent=2) + '\n')
        print(name, [(sample['phase'], sample['liveRustBytes']) for sample in data['samples']])
