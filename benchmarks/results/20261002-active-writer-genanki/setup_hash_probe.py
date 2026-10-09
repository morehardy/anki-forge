from pathlib import Path
import json,shutil
W=Path(__file__).resolve().parent;S=W/'hash-source'
shutil.copytree(W/'budget-source',S)
def edit(name,before,after):
 p=S/name;text=p.read_text();assert text.count(before)==1,(name,before,text.count(before));p.write_text(text.replace(before,after))
name='anki_forge/src/round4_diag.rs'
with (S/name).open('a') as stream:
 stream.write('pub fn direct() -> bool { static VALUE: LazyLock<bool> = LazyLock::new(|| std::env::var_os("ANKIFORGE_ROUND4_DIRECT_HASH").is_some()); *VALUE }\n')
name='anki_forge/src/media/snapshot.rs'
edit(name,'            { let _timer = crate::round4_diag::span(10); digest.update(&buffer[..count]); }','            if !crate::round4_diag::direct() || spool.is_some() { let _timer = crate::round4_diag::span(10); digest.update(&buffer[..count]); }')
edit(name,'                file.write_all(&memory)','                if crate::round4_diag::direct() { let _timer = crate::round4_diag::span(10); digest.update(&memory); digest.update(&buffer[..count]); }\n                file.write_all(&memory)')
edit(name,'        let storage = match spool {','        let digest = if crate::round4_diag::direct() && spool.is_none() { let _timer = crate::round4_diag::span(10); blake3::hash(&memory) } else { digest.finalize() };\n        let storage = match spool {')
edit(name,'digest: digest.finalize(),','digest,')
config=json.loads((W/'import-config.json').read_text());config['purpose']='All-at-once BLAKE3 for snapshots already retained in memory, with streaming promotion for larger files; fixed 4 MiB budget and active writer'
config['modes']={name:dict(binary='hash',env=dict(ANKIFORGE_ROUND4_WRITER='1',**({'ANKIFORGE_ROUND4_DIRECT_HASH':'1'} if name=='direct' else {}))) for name in ['streaming','direct']}
(W/'hash-config.json').write_text(json.dumps(config,indent=2)+'\n')
