//! frame-hitches: a per-frame WORK counter (`NF_WORKCOUNT=1`), the count-not-time companion of
//! `nf_game::prof`. Each hooked site adds one event (and its bytes) under a static name: wgpu object
//! creations and queue writes on the draw side, first-use decodes and loads on the sim/sound side. A
//! probe takes the counters after every frame and compares frames by the work they did, which
//! holds on a loaded machine where wall-clock times do not.
//!
//! Off by default: `add` is one relaxed atomic load and a branch. Never feeds back into the game, so
//! the simulation and the random stream are unaffected.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU8, Ordering};

/// 0 = not read yet, 1 = off, 2 = on.
static STATE: AtomicU8 = AtomicU8::new(0);

thread_local! {
    /// Per thread: a frame's work is the work its own thread did (the window's frame runs on one
    /// thread; a worker's decode is not that frame's), and parallel probes do not mix.
    static COUNTS: RefCell<Vec<(&'static str, u64, u64)>> = const { RefCell::new(Vec::new()) };
}

#[inline]
pub fn enabled() -> bool {
    match STATE.load(Ordering::Relaxed) {
        0 => {
            let on = std::env::var_os("NF_WORKCOUNT").is_some();
            STATE.store(if on { 2 } else { 1 }, Ordering::Relaxed);
            on
        }
        s => s == 2,
    }
}

/// Test hook: counting on (or off) without the env var.
pub fn force(on: bool) { STATE.store(if on { 2 } else { 1 }, Ordering::Relaxed); }

/// One event of `name` carrying `bytes` (0 when the event has no size).
#[inline]
pub fn add(name: &'static str, bytes: u64) {
    if !enabled() { return; }
    add_slow(name, bytes);
}

#[cold]
fn add_slow(name: &'static str, bytes: u64) {
    let _ = COUNTS.try_with(|c| {
        let Ok(mut v) = c.try_borrow_mut() else { return };
        match v.iter_mut().find(|e| std::ptr::eq(e.0, name) || e.0 == name) {
            Some(e) => { e.1 += 1; e.2 += bytes; }
            None => v.push((name, 1, bytes)),
        }
    });
}

/// This thread's counters since the last call: `(name, events, bytes)`, in first-seen order; clears them.
pub fn take() -> Vec<(&'static str, u64, u64)> {
    COUNTS.try_with(|c| std::mem::take(&mut *c.borrow_mut())).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    #[test]
    fn counts_only_while_on() {
        super::force(false);
        super::add("x", 5);
        assert!(super::take().is_empty(), "off: nothing is counted");
        super::force(true);
        super::add("x", 5);
        super::add("x", 7);
        super::add("y", 0);
        let t = super::take();
        super::force(false);
        assert_eq!(t, vec![("x", 2, 12), ("y", 1, 0)]);
        assert!(super::take().is_empty(), "take clears");
    }
}
