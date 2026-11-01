//! perf-240g: a small persistent worker pool for the palette build's per-character posing.
//!
//! `run(n, f)` calls `f(i, slot)` once for every item `i` in `0..n`, spread over the calling
//! thread (slot 0) and up to `workers()` pool threads (slots 1..). Items are claimed one at a
//! time from a shared counter, so the caller works the list itself from the first item: a
//! worker that wakes late only takes what is left, and with no workers (or `NF_POSE_THREADS=0`)
//! this is a plain loop. `run` returns only after every thread has left `f`, so `f` may borrow
//! the caller's data. Each item must be independent of the others and of which thread runs it —
//! the caller gives each slot its own scratch.
//!
//! The pool threads park on a condvar between jobs (no spinning while the game is idle).

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering::SeqCst};
use std::sync::{Condvar, Mutex, OnceLock};

/// The most slots a caller must provide scratch for (the caller's own slot 0 included).
pub const MAX_SLOTS: usize = 8;

type Job = dyn Fn(usize, usize) + Sync;

struct Pool {
    workers: usize,
    /// Bumped per job; a worker runs each generation at most once.
    gen: AtomicU64,
    /// The job's item count and the next unclaimed item.
    n: AtomicUsize,
    next: AtomicUsize,
    /// Set by the caller once it has no more items to claim; a worker that enters after it does
    /// not touch the job.
    closed: AtomicBool,
    /// Workers inside the current job (between their entry check and their exit).
    active: AtomicUsize,
    /// The current job, lifetime-erased; valid while `closed` is false.
    job: Mutex<Option<&'static Job>>,
    park: Mutex<u64>,
    wake: Condvar,
}

fn pool() -> &'static Pool {
    static P: OnceLock<&'static Pool> = OnceLock::new();
    P.get_or_init(|| {
        let workers = std::env::var("NF_POSE_THREADS").ok().and_then(|v| v.parse::<usize>().ok())
            .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()) / 6)
            .min(MAX_SLOTS - 1).min(3);
        let p: &'static Pool = Box::leak(Box::new(Pool {
            workers, gen: AtomicU64::new(0), n: AtomicUsize::new(0), next: AtomicUsize::new(0),
            closed: AtomicBool::new(true), active: AtomicUsize::new(0), job: Mutex::new(None),
            park: Mutex::new(0), wake: Condvar::new(),
        }));
        for slot in 1..=workers {
            std::thread::Builder::new().name(format!("nf-pose-{slot}")).spawn(move || worker(p, slot)).expect("pose worker");
        }
        p
    })
}

/// The pool threads in use (0 = `run` is a plain loop on the caller).
pub fn workers() -> usize { pool().workers }

fn worker(p: &'static Pool, slot: usize) {
    let mut seen = 0u64;
    loop {
        {
            let mut g = p.park.lock().unwrap();
            while *g == seen { g = p.wake.wait(g).unwrap(); }
            seen = *g;
        }
        p.active.fetch_add(1, SeqCst);
        if p.closed.load(SeqCst) || p.gen.load(SeqCst) != seen { p.active.fetch_sub(1, SeqCst); continue; }
        let job = *p.job.lock().unwrap();
        if let Some(f) = job {
            let n = p.n.load(SeqCst);
            loop {
                let i = p.next.fetch_add(1, SeqCst);
                if i >= n { break; }
                f(i, slot);
            }
        }
        p.active.fetch_sub(1, SeqCst);
    }
}

/// Run `f(i, slot)` for every `i` in `0..n` (see the module doc). Not re-entrant: `f` must not
/// call `run`.
pub fn run<'a>(n: usize, f: &'a (dyn Fn(usize, usize) + Sync + 'a)) {
    let p = pool();
    if p.workers == 0 || n < 2 {
        for i in 0..n { f(i, 0); }
        return;
    }
    // SAFETY: the reference is published only between here and the wait below; `closed` turns
    // true before the wait, a worker checks it (SeqCst, after announcing itself in `active`)
    // before reading the job, and the wait holds until every announced worker has left `f`.
    let erased: &'static Job = unsafe { std::mem::transmute::<&'a (dyn Fn(usize, usize) + Sync + 'a), &'static Job>(f) };
    *p.job.lock().unwrap() = Some(erased);
    p.n.store(n, SeqCst);
    p.next.store(0, SeqCst);
    p.closed.store(false, SeqCst);
    let gen = p.gen.fetch_add(1, SeqCst) + 1;
    { let mut g = p.park.lock().unwrap(); *g = gen; }
    p.wake.notify_all();
    loop {
        let i = p.next.fetch_add(1, SeqCst);
        if i >= n { break; }
        f(i, 0);
    }
    p.closed.store(true, SeqCst);
    while p.active.load(SeqCst) != 0 { std::hint::spin_loop(); }
    *p.job.lock().unwrap() = None;
}
