"""Private candidate: bounded dynamic work allocation with ordered consumption."""
from pathlib import Path

WORK = Path(__file__).resolve().parent
path = WORK / 'profile-source/anki_forge/src/prepared_media.rs'
text = path.read_text()
text = text.replace('use std::collections::{BTreeMap, BTreeSet};',
                    'use std::collections::{BTreeMap, BTreeSet, VecDeque};')
text = text.replace('use std::sync::Mutex;', 'use std::sync::{Arc, Mutex};\nuse std::sync::mpsc::{sync_channel, SyncSender};')
text = text.replace('// Each worker can hold one queued result plus one being prepared. The consumer\n// holds one more.',
                    '// The work window holds at most two payloads per worker plus one for\n// the consumer.')
start = text.index('                let mut receivers = Vec::new();')
end = text.index('\n            });', start)
text = text[:start] + '''                // Bound all submitted, unfinished and completed payloads together.
                // Per-job reply channels preserve order and disconnect if a worker
                // stops; no reply sender remains in a different waiting worker.
                type Preparation = Result<(IngestedMediaBytes, EncodedPayload), MediaIngestError>;
                let window = workers * 2 + 1;
                let (sender, receiver) = sync_channel::<(usize, SyncSender<Preparation>)>(window);
                let receiver = Arc::new(Mutex::new(receiver));
                let mut handles = Vec::new();
                for _ in 0..workers {
                    let receiver = Arc::clone(&receiver);
                    let jobs = &jobs;
                    let prepared = &*self;
                    handles.push(
                        std::thread::Builder::new()
                            .name("anki-forge-media".into())
                            .spawn_scoped(scope, move || {
                                let mut context = zstd::zstd_safe::CCtx::create();
                                loop {
                                    let job = receiver.lock().expect("media job queue lock").recv();
                                    let Ok((position, reply)) = job else { break };
                                    if reply.send(prepared.prepare_one(jobs[position].1, options, &mut context)).is_err() {
                                        break;
                                    }
                                }
                            }),
                    );
                }
                drop(receiver);
                let stopped = |item| diagnostic(
                    item,
                    "MEDIA.CAS_WRITE_FAILED",
                    "media preparation worker could not complete".into(),
                );
                if handles.iter().any(Result::is_err) {
                    drop(sender);
                    for &(index, item) in &jobs {
                        accept(index, Err(stopped(item)));
                    }
                } else {
                    let mut pending = VecDeque::with_capacity(window);
                    let mut next = 0;
                    for _ in 0..jobs.len() {
                        while next < jobs.len() && pending.len() < window {
                            let (reply, receiver) = sync_channel(1);
                            // Failed submission drops the reply sender, which is
                            // reported through the same ordered receive below.
                            let _ = sender.send((next, reply));
                            pending.push_back((next, receiver));
                            next += 1;
                        }
                        let (position, receiver) = pending.pop_front().expect("pending media job");
                        let (index, item) = jobs[position];
                        let result = receiver.recv().unwrap_or_else(|_| Err(stopped(item)));
                        accept(index, result);
                    }
                    drop(sender);
                }
                for handle in handles.into_iter().flatten() {
                    let _ = handle.join();
                }''' + text[end:]
path.write_text(text)
print('Dynamic preparation candidate ready')
