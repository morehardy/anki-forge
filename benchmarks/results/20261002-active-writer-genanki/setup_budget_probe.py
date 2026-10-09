from pathlib import Path
import json,shutil
W=Path(__file__).resolve().parent;S=W/'budget-source'
shutil.copytree(W/'import-probe-source',S)
def edit(name,before,after):
 p=S/name;text=p.read_text();assert text.count(before)==1,(name,before,text.count(before));p.write_text(text.replace(before,after))
name='anki_forge/src/round4_diag.rs'
edit(name,'(0..10)','(0..13)')
edit(name,'"source_open"];','"source_open", "blake3", "memory_copy", "source_close"];')
with (S/name).open('a') as stream:
 stream.write('pub fn budget() -> usize { static VALUE: LazyLock<usize> = LazyLock::new(|| std::env::var("ANKIFORGE_ROUND4_BUDGET_MIB").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(4) * 1024 * 1024); *VALUE }\n')
name='anki_forge/src/media/snapshot.rs'
edit(name,'> MEMORY_BUDGET','> crate::round4_diag::budget()')
edit(name,'            digest.update(&buffer[..count]);','            { let _timer = crate::round4_diag::span(10); digest.update(&buffer[..count]); }')
edit(name,'                memory.extend_from_slice(&buffer[..count]);','                let _timer = crate::round4_diag::span(11);\n                memory.extend_from_slice(&buffer[..count]);')
edit(name,'        let storage = match spool {','        { let _timer = crate::round4_diag::span(12); drop(source); }\n        let storage = match spool {')
config=json.loads((W/'import-config.json').read_text());config['purpose']='Active writer resident budget tradeoff; 4/12/16 MiB, additional source hash/copy/close spans; all groups close source after completing its read'
config['modes']={f'writer{budget}':dict(binary='budget',env={'ANKIFORGE_ROUND4_WRITER':'1','ANKIFORGE_ROUND4_BUDGET_MIB':str(budget)}) for budget in [4,12,16]}
(W/'budget-config.json').write_text(json.dumps(config,indent=2)+'\n')
