//! The in-game keypad mini-game (docs/systems/keypad-menu.md): `Locks_Init` 0x1b0548's
//! `KeyCodes` table, `C_KEYPAD_Handler` 0x216810's digit append/display, and `Lock_Update`
//! 0x1ae3d8 case 4's match — the port's `KeyCodes`/`Keypad` in nf-game lock.rs. Every
//! assertion cites its decompile line; the sisters are the negatives that can fail.

use nf_game::drone::Rng;
use nf_game::lock::{KeyCodes, Keypad};

/// `Locks_Init` 0x1b0548 [M :18-33]: 51 entries; four digits per entry, each
/// `(Rand_Random() % 9) + '0'` — digits '0'..'8', '9' impossible — then the display
/// flag byte 0. The warmup draws `warmup·10 & 0x1ff` [M :17]. Sister: the digit domain
/// (a `% 10` regression or an off-by-one in the entry count flips it).
#[test]
fn locks_init_table_shape_and_digit_domain() {
    let mut rng = Rng::new(0x1b0548);
    let kc = KeyCodes::init(&mut rng, 0x2a);
    assert_eq!(kc.0.len(), 0x33, "51 entries [M 0x1b0548:31]");
    let mut draws = 0;
    let mut rng2 = Rng::new(0x1b0548);
    let _ = KeyCodes::init(&mut rng2, 0x2a);
    assert_eq!(rng2.draws, (0x2a * 10 & 0x1ff) as u64 + 0x33 * 4, "warmup + 51·4 draws [M :17-30]");
    for e in kc.0.iter() {
        for d in e.iter().take(4) {
            assert!((b'0'..=b'8').contains(d), "digit {d} outside '0'..'8' [M (rand % 9) + '0']");
        }
        assert_eq!(e[4], 0, "display flag starts 0 [M :32]");
    }
}

/// `C_KEYPAD_Handler` 0x216810 msg 0x4b [M :8-23]: the count wraps to 0 at the cap (4), the
/// digit lands at typed[count], and the display string is typed[0..=count] INCLUSIVE (the
/// just-typed digit included, the rest still '*' masks) [M the copy loop `uVar2 + 1 <= uVar3`].
#[test]
fn keypad_digit_append_wraps_and_display_is_inclusive() {
    let mut kp = Keypad::arm([b'1', b'2', b'3', b'4'], false);
    assert_eq!(kp.display(), "", "the armed display: nothing typed yet");
    kp.digit(7);
    assert_eq!(kp.display(), "7", "typed[0..=0] — the just-typed digit only [M the inclusive copy]");
    kp.digit(0);
    assert_eq!(kp.display(), "70");
    kp.digit(2);
    kp.digit(5);
    assert_eq!(kp.count, 4, "the cap [M 0x2a37e2 = 4]");
    assert_eq!(kp.display(), "7025");
    // The wrap [M 0x216810:8-10: `if (cap <= count) count = 0`].
    kp.digit(9);
    assert_eq!(kp.count, 1, "the count wrapped to 0 then incremented");
    assert_eq!(kp.display(), "9", "the display restarted from the wrapped buffer");
}

/// `Lock_Update` case 4 [M 0x1ae3d8:262-296]: a FULL buffer equal to the code unlocks
/// (`Lock_Unlock` — the port's `solved` + the caller's out channel); anything short stays open.
/// Sisters: a wrong digit, and a partial buffer, both fail to unlock.
#[test]
fn keypad_match_unlocks_only_a_full_equal_buffer() {
    let mut kp = Keypad::arm([b'4', b'4', b'7', b'2'], false);
    for d in [4u32, 4, 7, 2] { kp.digit(d); }
    // The match runs on the next Lock_Update tick after the last digit [M 0x1ae3d8:262-282]:
    // full + equal → Lock_Unlock [M :283-289] on the first tick that sees the full buffer.
    assert!(kp.tick(), "full + equal unlocks");
    assert!(kp.solved);
    assert!(!kp.open, "MenuManager_Delete closes the page [M :288]");

    let mut wrong = Keypad::arm([b'1', b'2', b'3', b'4'], false);
    for d in [1u32, 2, 3, 5] { wrong.digit(d); }
    assert!(!wrong.tick(), "a wrong digit must NOT unlock");
    assert!(!wrong.solved && wrong.open, "the page stays up (the game re-enables it per frame [M :273-276])");

    let mut partial = Keypad::arm([b'1', b'2', b'3', b'4'], false);
    for d in [1u32, 2, 3] { partial.digit(d); }
    assert!(!partial.tick(), "count < cap must NOT unlock even when the prefix matches");
}

/// The Q-Worm reveal [M 0x1aee20:28-31 + 0x1ae3d8:249-258]: `hint` prints the code only when
/// the entry's display flag is set. Sister: the unrevealed default.
#[test]
fn keypad_hint_prints_only_after_the_q_worm_flag() {
    let mut kp = Keypad::arm([b'0', b'2', b'7', b'1'], false);
    assert_eq!(kp.hint(), "", "no reveal, no hint");
    let mut shown = Keypad::arm([b'0', b'2', b'7', b'1'], true);
    assert_eq!(shown.hint(), "0271", "the sprintf'd code [M 0x1ae3d8:252-257]");
    shown.close();
    assert!(!shown.open);
}
