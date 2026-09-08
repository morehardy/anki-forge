//! Scoped I/O jobs with at most four active operations and ordered results.
//! Results must be metadata or owned temporary files, not buffered payloads.
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub(crate) fn ordered<T: Send>(
    count: usize,
    operation: impl Fn(usize) -> T + Sync,
) -> Vec<Result<T, ()>> {
    let workers = if count < 16 {
        1
    } else {
        std::thread::available_parallelism()
            .map_or(1, usize::from)
            .min(4)
    };
    run(count, workers, operation)
}

fn run<T: Send>(
    count: usize,
    workers: usize,
    operation: impl Fn(usize) -> T + Sync,
) -> Vec<Result<T, ()>> {
    let next = AtomicUsize::new(0);
    let results = Mutex::new((0..count).map(|_| None).collect::<Vec<_>>());
    let work = || loop {
        let index = next.fetch_add(1, Ordering::Relaxed);
        if index >= count {
            break;
        }
        let result = catch_unwind(AssertUnwindSafe(|| operation(index))).map_err(|_| ());
        results.lock().unwrap_or_else(|error| error.into_inner())[index] = Some(result);
    };
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        // The calling thread also works; failed spawns reduce concurrency and
        // never lose queued jobs. No task can outlive the caller's sources.
        for _ in 1..workers {
            if let Ok(handle) = std::thread::Builder::new()
                .name("anki-forge-io".into())
                .spawn_scoped(scope, work)
            {
                handles.push(handle);
            }
        }
        work();
        for handle in handles {
            let _ = handle.join();
        }
    });
    results
        .into_inner()
        .unwrap_or_else(|error| error.into_inner())
        .into_iter()
        .map(|result| result.unwrap_or(Err(())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jobs_are_bounded_joined_and_returned_in_input_order_even_after_a_panic() {
        let active = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        let completed = AtomicUsize::new(0);
        let results = run(40, 4, |index| {
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(current, Ordering::SeqCst);
            std::thread::sleep(std::time::Duration::from_millis((index % 3 + 1) as u64));
            active.fetch_sub(1, Ordering::SeqCst);
            completed.fetch_add(1, Ordering::SeqCst);
            assert_ne!(index, 7, "injected I/O job panic");
            index
        });
        assert!(peak.load(Ordering::SeqCst) <= 4);
        assert_eq!(active.load(Ordering::SeqCst), 0);
        assert_eq!(completed.load(Ordering::SeqCst), 40);
        assert_eq!(
            results,
            (0..40)
                .map(|index| if index == 7 { Err(()) } else { Ok(index) })
                .collect::<Vec<_>>()
        );
        assert_eq!(run(4, 1, |index| index), vec![Ok(0), Ok(1), Ok(2), Ok(3)]);
    }
}
