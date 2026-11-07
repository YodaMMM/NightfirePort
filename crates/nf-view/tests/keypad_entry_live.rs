//! keypad-entry: the keypad page's input, pinned through real input on two census levels.
//!
//! The census [D `keypad_census.rs`]: 33 levels carry the identical keypad page
//! (`MenuManager_Create(0x80000003, 0x40000021, …)` [M `Lock_Update` 0x1ae3d8:242-243]) whose
//! ten digit controls (id 0x100000ba) sit at the port's `KEYPAD_DIGIT_RECTS` with the digits
//! 1..9,0; 15 kind-0 keypad locks across 8 levels. This file pins, on 0700000a (locks #286
//! code-index 2 / out ch 0x31 and #288 code-index 1 / out ch 0x30) and 07000011 (lock #0b1
//! code-index 1 / out ch 0x12) [D the census]:
//! - the use press arms the page (the game's `Lock_Activate` 0x1aed00 kind-0 state-3 arm);
//! - the stick moves the page cursor, ✕ types the digit under it (`C_KEYPAD_Handler`
//!   0x216810:8-22 via the manager's msg-0x4b dispatch [M 0x218cb8]); the display rests as the
//!   `****` masks and grows per digit [M 0x216810:13-29 — the count-0 sprintf's literal
//!   "****", then typed[0..=count]];
//! - the right code unlocks the lock and writes its out channel (`Lock_Unlock` [M
//!   0x1ae3d8:283-289]; the case-1 hold writes `switch_channels[rec+8]` every tick);
//! - a wrong code never unlocks; the 5th press wraps the count [M 0x216810:8-10];
//! - the back edge closes the page and the lock re-arms (the case-4 Exists==0 arm [M
//!   0x1ae3d8:292-295]).
//!
//! The teleports are the documented stand-in transit (the use press, the stick, the select and
//! the back edges all run through the real `step_bond` input path). The typed code is the
//! port's own `KeyCodes` table entry — the game's table is boot-RNG (`Locks_Init` 0x1b0548
//! [M]; the real digits are a live check in the report), the port generates the same shape
//! deterministically.

use nf_view::scene::Sim;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn digit_centre(d: u32) -> (f32, f32) {
    for (i, &v) in nf_game::lock::KEYPAD_DIGIT_VALUES.iter().enumerate() {
        if v == d {
            let (x, y, w, h) = nf_game::lock::KEYPAD_DIGIT_RECTS[i];
            return (x as f32 + w as f32 / 2.0, y as f32 + h as f32 / 2.0);
        }
    }
    unreachable!("digit {d}")
}

/// Hold the stick toward `want`'s rect centre until the cursor is over it, then press ✕ —
/// the whole steer through real `step_bond` input (the sim's page routing moves the cursor
/// [S] 3.0 units/tick per full stick, the same constant the steer compensates for).
fn steer_and_confirm(sim: &mut Sim, want: u32) {
    let (tx, ty) = digit_centre(want);
    let mut reached = false;
    for _ in 0..400 {
        if sim.drones.keypad_digit_under() == Some(want) { reached = true; break; }
        let cur = sim.drones.keypad_cursor().expect("the page stands");
        let dx = (tx - cur[0]).clamp(-3.0, 3.0);
        let dy = (ty - cur[1]).clamp(-3.0, 3.0);
        // keypad_page_input: cursor += (strafe, −forward)·3 — the forward lane inverts.
        let input = nf_game::player::Input { strafe: dx / 3.0, forward: -dy / 3.0, ..Default::default() };
        sim.step_bond(input, false, false, false);
    }
    assert!(reached, "the cursor never reached digit {want}");
    sim.step_bond(nf_game::player::Input { keypad_confirm: true, ..Default::default() }, false, false, false);
}

fn type_code(sim: &mut Sim, digits: &[u8]) {
    for &d in digits {
        steer_and_confirm(sim, (d - b'0') as u32);
    }
}

/// Stand Bond in front of the lock (inside the use sphere, on the facing's positive side
/// [M 0x1a5170:158-166's `facing · (body − lock) < 0` refusal]) and press the use key —
/// the teleport is the documented stand-in transit, the press is the real edge.
fn use_lock(sim: &mut Sim, li: usize) {
    let (pos, facing, radius) = {
        let l = &sim.drones.locks[li];
        (l.pos, l.facing, l.radius)
    };
    let at = [pos[0] + facing[0] * 1.2, pos[1] + facing[1] * 1.2, pos[2] + facing[2] * 1.2];
    let yaw = (pos[0] - at[0]).atan2(pos[2] - at[2]);
    {
        let sim = &mut *sim;
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, at, yaw);
        // Aim the look at the lock's centre: the use pick is the head + look·1.0 sphere
        // [M 0x1a5170:100-108] — the pitch carries the height difference.
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        let (dx, dy, dz) = (pos[0] - eye[0], pos[1] - eye[1], pos[2] - eye[2]);
        sim.bond.pitch = (dy / (dx * dx + dz * dz).sqrt().max(0.5)).atan();
        sim.bond.vel = [0.0; 3];
        eprintln!("[use_lock {li}] lock at ({:.1},{:.1},{:.1}) r {radius:.2}, stand ({:.1},{:.1},{:.1}), yaw {:.2} pitch {:.2}",
            pos[0], pos[1], pos[2], sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], yaw, sim.bond.pitch);
    }
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); } // settle on the floor
    sim.step_bond(Default::default(), false, false, true); // the use edge
}

/// The kind-0 lock with the census's (code index, out channel) pair [D the census].
fn find_lock(sim: &Sim, code: u16, out: u16) -> usize {
    sim.drones.locks.iter().position(|l| l.kind == 0 && l.code == code && l.out == out)
        .unwrap_or_else(|| panic!("the census's kind-0 lock (code {code}, out {out:#04x}) exists"))
}

/// The port's own table code for the lock's index — the four digits the test types.
fn port_code(sim: &Sim, li: usize) -> [u8; 4] {
    sim.drones.key_codes.code(sim.drones.locks[li].code)
}

#[test]
fn the_right_code_opens_0700000a_s_keypad_and_writes_its_channel() {
    let mut scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a");
    let sim = scene.sim.as_mut().unwrap();
    // The census [D]: 0700000a carries four kind-0 locks; #286 (code index 2, out 0x31) first.
    let li = find_lock(sim, 2, 0x31);
    assert!(!sim.drones.locks[li].unlocked);
    use_lock(sim, li);
    assert!(sim.drones.keypad.is_some(), "the use press arms the keypad page [M 0x1aed00's kind-0 state-3 arm]");
    assert_eq!(sim.drones.keypad_display().as_deref(), Some("****"),
        "the page's resting display is the four masks [M 0x216810:24-29]");
    let code = port_code(sim, li);
    eprintln!("0700000a lock {li}: typing the port's table code {:?}", std::str::from_utf8(&code).unwrap());
    // The first three digits: the display grows and can't match yet.
    type_code(sim, &code[..3]);
    assert_eq!(sim.drones.keypad_display().as_deref(), Some(std::str::from_utf8(&code[..3]).unwrap()),
        "the display reads the typed digits [M 0x216810:13-22]");
    assert!(!sim.drones.locks[li].unlocked, "an incomplete code never unlocks [M the count==cap gate]");
    // The 4th confirm: the same tick's case-4 matches and solves — the page closes and the
    // lock unlocks in ONE step_bond (the game's case 4 runs every tick of the open page
    // [M 0x1ae3d8:262-289]).
    steer_and_confirm(sim, (code[3] - b'0') as u32);
    for _ in 0..5 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.locks[li].unlocked, "the full matching code unlocks [M 0x1ae3d8:283-289]");
    assert!(sim.drones.keypad.is_none(), "the manager is deleted on the solve [M 0x1ae3d8:288]");
    assert!(sim.drones.channels.on(0x31), "the lock's out channel writes [M 0x1ae3d8:283-289, the case-1 hold]");
}

#[test]
fn the_wrap_and_the_cap_on_0700000a() {
    let mut scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a");
    let sim = scene.sim.as_mut().unwrap();
    // The second keypad (#288: code index 1, out 0x30 [D the census]) takes the wrap pin.
    let li = find_lock(sim, 1, 0x30);
    use_lock(sim, li);
    assert!(sim.drones.keypad.is_some());
    // Five presses: the fifth wraps the count to 0 [M 0x216810:8-10] — the display shows one digit.
    for d in [1u32, 2, 3, 4, 5] { steer_and_confirm(sim, d); }
    let disp = sim.drones.keypad_display().unwrap();
    assert_eq!(disp.len(), 1, "the 5th press wrapped the count — the display shows the wrapped single digit ({disp})");
    assert!(!sim.drones.locks[li].unlocked, "an incomplete code never unlocks [M 0x1ae3d8's count==cap gate]");
    // Complete it with the real code: three more digits (positions 1..3 — the wrap left "5" at 0,
    // which cannot match the code's first digit, so type to a REFUSAL first — the pin here is the
    // wrap, not the solve).
    for _ in 0..5 { sim.step_bond(Default::default(), false, false, false); }
    assert!(!sim.drones.locks[li].unlocked);
}

#[test]
fn a_wrong_code_never_opens_and_closing_leaves_07000011_s_lock_usable() {
    let mut scene = nf_view::load_with_carry_weapons_difficulty(&iso(), "07000011.bin", false, None, &[], 2)
        .expect("load 07000011 at difficulty 2");
    let sim = scene.sim.as_mut().unwrap();
    // The census [D]: 07000011 carries one kind-0 lock (#0b1: code index 1, out ch 0x12). Its
    // param-6 tag starts it UNLOCKED on Operative [M 0x1ae1f8:67-69] — the plain load's
    // difficulty 1 refuses the arm (the game's own behavior); the test plays at difficulty 2.
    let li = find_lock(sim, 1, 0x12);
    eprintln!("07000011 lock {li}: kind {} unlocked {} unlock_on_operative {}", sim.drones.locks[li].kind, sim.drones.locks[li].unlocked, sim.drones.locks[li].unlock_on_operative);
    use_lock(sim, li);
    assert!(sim.drones.keypad.is_some(), "the page arms");
    // A wrong code: every digit shifted +1 mod 9 — differs from the code in EVERY position
    // (the table's digits are '0'..'8' [M 0x1b0548: (Rand % 9) + '0']).
    let code = port_code(sim, li);
    let wrong: Vec<u8> = code.iter().map(|&d| b'0' + (d - b'0' + 1) % 9).collect();
    assert_ne!(wrong, code);
    type_code(sim, &wrong);
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    assert!(!sim.drones.locks[li].unlocked, "a wrong code never unlocks [M 0x1ae3d8's match gate]");
    assert!(sim.drones.keypad.is_some(), "the page stands on a wrong code");
    assert!(!sim.drones.channels.on(0x12), "the out channel stays off");
    // The back edge closes the page; the lock's case-4 reads the manager gone → state 0.
    sim.step_bond(nf_game::player::Input { keypad_back: true, ..Default::default() }, false, false, false);
    for _ in 0..5 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.keypad.is_none(), "the back edge closes the page [M 0x218cb8's edge-2 arm]");
    // Re-arm: the lock is usable again — the case-4 Exists==0 arm returned it to state 0
    // [M 0x1ae3d8:292-295].
    use_lock(sim, li);
    assert!(sim.drones.keypad.is_some(), "the closed page leaves the lock re-armable [M 0x1ae3d8:292-295]");
    // And the right code now opens it — the 4th confirm's tick solves (the same-tick case-4).
    let code = port_code(sim, li);
    type_code(sim, &code[..3]);
    steer_and_confirm(sim, (code[3] - b'0') as u32);
    for _ in 0..5 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.locks[li].unlocked, "the right code unlocks after the re-arm");
    assert!(sim.drones.channels.on(0x12), "the out channel writes [M the case-1 hold]");
}
