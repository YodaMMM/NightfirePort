//! fix-decryptor-sounds: the decryptor's per-digit and unlock chimes.
//!
//! [M `Lock_Update` 0x1ae3d8: `Sound_Play(0x42c80000 = 100.0, 0x5db, 0, 0)` on every solved digit
//! (:368) and `Sound_Play(100.0, 0x5da, 0, 0)` on the unlock (:374)]. The port's scan solved the
//! digits silently. The sim now banks the ids in `Lock::pending_sounds` and the viewer plays them at
//! the lock's position; this test drives the lock's own tick (no ISO) and pins the exact sequence.
//!
//! Fail-first: on the pre-fix tree `pending_sounds` does not exist, so this test does not compile —
//! the repo's accepted shape for a new field. On the patched tree it passes.

use nf_game::lock::Lock;
use nf_game::triggers::Channels;

fn lock() -> Lock {
    Lock {
        placement: 0,
        pos: [1.0, 2.0, 3.0],
        radius: 0.5,
        out: 0,
        key_channels: [0; 2],
        burn: 0,
        burned: false,
        unlock_on_operative: false,
        unlocked: false,
        kind: 0, // a decryptor lock [M 0x1ae3d8's case-0 decryptor arm]
        code: 0,
        burn_a: 0xff,
        burn_b: 0xff,
        grace: 0,
        scan_progress: 0,
        scan_solved: 0,
        pending_sounds: Vec::new(),
        scanning: false,
        lod_next: false,
        facing: [0.0, 0.0, 1.0],
    }
}

/// Run a held scan until the lock opens, collecting every sound id it raises, tick by tick.
fn scan_until_unlocked(upgraded: bool) -> (Vec<u32>, usize) {
    let mut l = lock();
    let mut ch = Channels::default();
    let mut ids = Vec::new();
    let mut ticks = 0;
    while !l.unlocked && ticks < 100_000 {
        ticks += 1;
        let _ = l.decrypt_tick(1, upgraded, &mut ch, ticks as u32);
        ids.extend(l.pending_sounds.drain(..));
    }
    (ids, ticks)
}

#[test]
fn a_solved_scan_chimes_once_per_digit_and_once_on_the_unlock() {
    let (ids, ticks) = scan_until_unlocked(false);
    assert!(ticks < 100_000, "the scan never unlocked");
    let want = vec![0x5db, 0x5db, 0x5db, 0x5db, 0x5da];
    assert_eq!(ids, want,
        "one 0x5db per solved digit then one 0x5da on the unlock [M 0x1ae3d8:368/:374], got {ids:?}");
}

/// Sister: the chimes are per SOLVE, not per tick — a scan that is broken before the first digit
/// (stopped early) raises nothing, and a scan reset by the decay raises nothing for the lost progress.
#[test]
fn only_solved_digits_chime() {
    let mut l = lock();
    let mut ch = Channels::default();
    // 30 ticks of scan: the first digit needs progress > 160 at 1/tick (plus the state-0 flip), so
    // nothing is solved yet and no sound may be raised.
    for f in 0..30u32 { let _ = l.decrypt_tick(1, false, &mut ch, f); }
    assert_eq!(l.scan_solved, 0, "no digit before the 160-progress threshold [M 0x1ae3d8:347-364]");
    assert!(l.pending_sounds.is_empty(), "no chime without a solved digit, got {:?}", l.pending_sounds);
    // The decay reset throws the partial progress away, silently [M 0x1ae3d8:340-344].
    for f in 0..60u32 { let _ = l.decrypt_tick(0, false, &mut ch, f); }
    assert_eq!(l.scan_solved, 0);
    assert!(l.pending_sounds.is_empty(), "a reset is silent, got {:?}", l.pending_sounds);
}
