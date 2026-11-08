//! The keypad page's input, pinned through real `step_bond` input on two census levels.
//!
//! The census [D `keypad_census.rs`]: 33 levels carry the identical keypad page
//! (`MenuManager_Create(0x80000003, 0x40000021, …)` [M `Lock_Update` 0x1ae3d8:242-243]) whose ten
//! digit buttons (id 0x100000ba) sit at the port's `KEYPAD_DIGIT_RECTS` with the digits 1..9,0;
//! 15 kind-0 keypad locks across 8 levels. Pinned here:
//! - with no draw the page holds nothing: Bond walks on after the press;
//! - with a draw the page owns the pad, the cursor steps control to control
//!   (`Menu_FindControl` 0x1f16d8) from its home on digit 1, ✕ types the focused digit
//!   (`Button_Update` 0x1ef948 → `C_KEYPAD_Handler` 0x216810), △ closes it (0x21c3d8, 0x21d508);
//! - the sounds: a step and a press 0x1da, the close 0x1d9, a blocked step nothing;
//! - the right code unlocks and writes the out channel; a wrong code never does; the 5th press
//!   wraps; a closed page leaves the lock re-armable [M 0x1ae3d8:292-295];
//! - a solve equips the code's key, so a sister keypad opens on the press [M 0x1b0650:15, 0x1aed00].
//!
//! The teleports are the documented stand-in transit. The typed code is the port's own `KeyCodes`
//! entry: the game's table is boot-RNG (`Locks_Init` 0x1b0548 [M]).

use nf_game::player::Input;
use nf_view::menu::{keypad_step, pad_keypad_lanes};
use nf_view::scene::Sim;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn tick(sim: &mut Sim, input: Input) { sim.step_bond(input, false, false, false); }
fn idle(sim: &mut Sim) { tick(sim, Input::default()); }

/// Press and release one direction.
fn press_nav(sim: &mut Sim, bit: u8) {
    tick(sim, Input { keypad_nav: bit, ..Default::default() });
    idle(sim);
}

fn press_confirm(sim: &mut Sim) {
    tick(sim, Input { keypad_confirm: true, ..Default::default() });
    idle(sim);
}

fn index_of(d: u32) -> usize {
    nf_game::lock::KEYPAD_DIGIT_VALUES.iter().position(|&v| v == d).expect("a digit")
}

/// The direction presses from one control to another, found over the game's own step.
fn path(from: usize, to: usize) -> Vec<u8> {
    let mut seen = vec![None; 10];
    seen[from] = Some(Vec::new());
    let mut queue = std::collections::VecDeque::from([from]);
    while let Some(c) = queue.pop_front() {
        for bit in [1u8, 2, 4, 8] {
            let n = keypad_step(c, bit);
            if seen[n].is_none() {
                let mut p: Vec<u8> = seen[c].clone().unwrap();
                p.push(bit);
                seen[n] = Some(p);
                queue.push_back(n);
            }
        }
    }
    seen[to].clone().expect("every digit is reachable")
}

/// Step the cursor onto a digit with real presses, then press ✕.
fn steer_and_confirm(sim: &mut Sim, want: u32) {
    let from = index_of(sim.drones.keypad_focus_digit().expect("the page stands"));
    for bit in path(from, index_of(want)) { press_nav(sim, bit); }
    assert_eq!(sim.drones.keypad_focus_digit(), Some(want), "the cursor stands on the digit");
    press_confirm(sim);
}

fn type_code(sim: &mut Sim, digits: &[u8]) {
    for &d in digits { steer_and_confirm(sim, (d - b'0') as u32); }
}

/// Stand Bond in front of the lock (inside the use sphere, on the facing's positive side
/// [M 0x1a5170:158-166's `facing · (body − lock) < 0` refusal]) and press the use key: the
/// teleport is the documented stand-in transit, the press is the real edge.
fn use_lock(sim: &mut Sim, li: usize) {
    let (pos, facing) = {
        let l = &sim.drones.locks[li];
        (l.pos, l.facing)
    };
    let at = [pos[0] + facing[0] * 1.2, pos[1] + facing[1] * 1.2, pos[2] + facing[2] * 1.2];
    let yaw = (pos[0] - at[0]).atan2(pos[2] - at[2]);
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, at, yaw);
    // Aim the look at the lock's centre: the use pick is the head + look·1.0 sphere
    // [M 0x1a5170:100-108]; the pitch carries the height difference.
    let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
    let (dx, dy, dz) = (pos[0] - eye[0], pos[1] - eye[1], pos[2] - eye[2]);
    sim.bond.pitch = (dy / (dx * dx + dz * dz).sqrt().max(0.5)).atan();
    sim.bond.vel = [0.0; 3];
    for _ in 0..10 { idle(sim); } // settle on the floor
    sim.step_bond(Default::default(), false, false, true); // the use edge
    idle(sim); // the release
}

/// The kind-0 lock with the census's (code index, out channel) pair [D the census].
fn find_lock(sim: &Sim, code: u16, out: u16) -> usize {
    sim.drones.locks.iter().position(|l| l.kind == 0 && l.code == code && l.out == out)
        .unwrap_or_else(|| panic!("the census's kind-0 lock (code {code}, out {out:#04x}) exists"))
}

fn port_code(sim: &Sim, li: usize) -> [u8; 4] {
    sim.drones.key_codes.code(sim.drones.locks[li].code)
}

/// The voices of `id` the pool holds; a press is read as the rise (the headless tick stops none).
fn voices(sim: &Sim, id: u32) -> usize {
    sim.drones.audio.live_ids().iter().filter(|&&x| x == id).count()
}

/// The page at lock (code 2, out 0x31) of 0700000a, up and drawn.
fn open_0a() -> (nf_view::scene::Scene, usize) {
    let mut scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a");
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.keypad_page_drawn = true;
    let li = find_lock(sim, 2, 0x31);
    use_lock(sim, li);
    assert!(sim.drones.keypad.is_some(), "the use press arms the keypad page [M 0x1aed00's kind-0 state-3 arm]");
    (scene, li)
}

#[test]
fn a_page_nothing_draws_leaves_bond_free() {
    let mut scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a");
    let sim = scene.sim.as_mut().unwrap();
    let li = find_lock(sim, 2, 0x31);
    use_lock(sim, li);
    assert!(sim.drones.keypad.is_some(), "the press still arms the lock's state");
    let from = sim.bond.pos;
    let yaw = sim.bond.yaw;
    for _ in 0..30 { tick(sim, Input { forward: -1.0, ..Default::default() }); }
    let d = ((sim.bond.pos[0] - from[0]).powi(2) + (sim.bond.pos[2] - from[2]).powi(2)).sqrt();
    assert!(d > 0.5, "Bond walks away from the keypad (moved {d:.3})");
    tick(sim, Input { turn: 0.05, ..Default::default() });
    assert_ne!(sim.bond.yaw, yaw, "and turns");
}

#[test]
fn a_drawn_page_holds_bond_until_it_is_left() {
    let (mut scene, _) = open_0a();
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..5 { idle(sim); }
    let from = sim.bond.pos;
    for _ in 0..30 { tick(sim, Input { forward: -1.0, ..Default::default() }); }
    assert_eq!(sim.bond.pos, from, "the menu owns the pad [M 0x193600:80-87]");
    tick(sim, Input { keypad_back: true, ..Default::default() });
    idle(sim);
    assert!(sim.drones.keypad.is_none(), "△ closes the page");
    for _ in 0..30 { tick(sim, Input { forward: -1.0, ..Default::default() }); }
    assert_ne!(sim.bond.pos, from, "and Bond walks again");
}

#[test]
fn the_cursor_steps_between_the_digits_from_its_home() {
    // Up, down, left, right from each digit, by `Menu_FindControl`'s scoring [M 0x1f16d8]: the
    // nearest control that way, else the farthest the other way (the wrap), else stay. The 0 has
    // nothing beside it, and only the middle column reaches it.
    let table: [(u32, [u32; 4]); 10] = [
        (1, [7, 4, 3, 2]), (2, [0, 5, 1, 3]), (3, [9, 6, 2, 1]),
        (4, [1, 7, 6, 5]), (5, [2, 8, 4, 6]), (6, [3, 9, 5, 4]),
        (7, [4, 1, 9, 8]), (8, [5, 0, 7, 9]), (9, [6, 3, 8, 7]),
        (0, [8, 2, 0, 0]),
    ];
    for (from, to) in table {
        for (k, bit) in [1u8, 2, 4, 8].into_iter().enumerate() {
            let got = nf_game::lock::KEYPAD_DIGIT_VALUES[keypad_step(index_of(from), bit)];
            assert_eq!(got, to[k], "digit {from} direction {bit}");
        }
    }
    let (mut scene, _) = open_0a();
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.drones.keypad_focus_digit(), Some(1), "home is the first usable control [M 0x1f2708]");
    press_nav(sim, 8);
    assert_eq!(sim.drones.keypad_focus_digit(), Some(2));
    press_nav(sim, 2);
    assert_eq!(sim.drones.keypad_focus_digit(), Some(5));
    // A stick held at full deflection is one step, not a slide.
    tick(sim, Input { forward: 1.0, strafe: 1.0, ..Default::default() });
    assert_eq!(sim.drones.keypad_focus_digit(), Some(5), "the movement lanes do not move the cursor");
}

#[test]
fn the_page_sounds_are_the_game_s() {
    let (mut scene, _) = open_0a();
    let sim = scene.sim.as_mut().unwrap();
    let (a0, d0) = (voices(sim, 0x1da), voices(sim, 0x1d9));
    press_nav(sim, 8);
    assert_eq!(voices(sim, 0x1da), a0 + 1, "a step plays Menu_PlaySound(7) [M 0x21edc8:47]");
    press_confirm(sim);
    assert_eq!(voices(sim, 0x1da), a0 + 2, "a press plays Menu_PlaySound(0) [M 0x1ef948:106]");
    assert_eq!(sim.drones.keypad_display().as_deref(), Some("2"));
    // Walk to the 0, where left and right find nothing: no step, no sound.
    for bit in path(index_of(2), index_of(0)) { press_nav(sim, bit); }
    let a1 = voices(sim, 0x1da);
    press_nav(sim, 4);
    assert_eq!(sim.drones.keypad_focus_digit(), Some(0));
    assert_eq!(voices(sim, 0x1da), a1, "a step that finds no control is silent");
    tick(sim, Input { keypad_back: true, ..Default::default() });
    assert_eq!(voices(sim, 0x1d9), d0 + 1, "the close plays Menu_PlaySound(8) = 0x1d9 [M 0x21c3d8:101, 0x1f3278 case 8]");
    assert_eq!(voices(sim, 0x1da), a1, "and not the select sample");
}

#[test]
fn a_use_key_still_held_at_the_close_does_not_reopen_the_page() {
    let (mut scene, _) = open_0a();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Default::default(), false, false, true);
    sim.step_bond(Input { keypad_back: true, ..Default::default() }, false, false, true);
    for _ in 0..4 {
        sim.step_bond(Default::default(), false, false, true);
        assert!(sim.drones.keypad.is_none(), "the held key is no fresh press");
    }
    idle(sim);
    sim.step_bond(Default::default(), false, false, true);
    assert!(sim.drones.keypad.is_some(), "a new press re-arms [M 0x1ae3d8:292-295]");
}

#[test]
fn a_pad_can_work_the_page_and_leave_it() {
    let (mut scene, _) = open_0a();
    let sim = scene.sim.as_mut().unwrap();
    let pad = |buttons: u16, lx: u8, ly: u8| {
        let (keypad_nav, keypad_confirm, keypad_back) = pad_keypad_lanes(buttons, lx, ly);
        Input { keypad_nav, keypad_confirm, keypad_back, ..Default::default() }
    };
    let rest = pad(0, 0x7f, 0x7f);
    assert_eq!((rest.keypad_nav, rest.keypad_confirm, rest.keypad_back), (0, false, false));
    tick(sim, pad(0x2000, 0x7f, 0x7f)); // d-pad right
    tick(sim, rest);
    assert_eq!(sim.drones.keypad_focus_digit(), Some(2));
    tick(sim, pad(0, 0x7f, 0xff)); // left stick down
    tick(sim, rest);
    assert_eq!(sim.drones.keypad_focus_digit(), Some(5));
    tick(sim, pad(0, 0xa0, 0x7f)); // under half deflection
    tick(sim, rest);
    assert_eq!(sim.drones.keypad_focus_digit(), Some(5), "the stick needs more than half [M 0x10cd80]");
    tick(sim, pad(0x40, 0x7f, 0x7f)); // ✕
    tick(sim, rest);
    assert_eq!(sim.drones.keypad_display().as_deref(), Some("5"));
    tick(sim, pad(0x10, 0x7f, 0x7f)); // △
    tick(sim, rest);
    assert!(sim.drones.keypad.is_none(), "△ leaves the page");
}

#[test]
fn the_right_code_opens_0700000a_s_keypad_and_writes_its_channel() {
    let (mut scene, li) = open_0a();
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.drones.keypad_display().as_deref(), Some("****"),
        "the page's resting display is the four masks [M 0x216810:24-29]");
    let code = port_code(sim, li);
    type_code(sim, &code[..3]);
    assert_eq!(sim.drones.keypad_display().as_deref(), Some(std::str::from_utf8(&code[..3]).unwrap()),
        "the display reads the typed digits [M 0x216810:13-22]");
    assert!(!sim.drones.locks[li].unlocked, "an incomplete code never unlocks [M the count==cap gate]");
    steer_and_confirm(sim, (code[3] - b'0') as u32);
    for _ in 0..5 { idle(sim); }
    assert!(sim.drones.locks[li].unlocked, "the full matching code unlocks [M 0x1ae3d8:283-289]");
    assert!(sim.drones.keypad.is_none(), "the manager is deleted on the solve [M 0x1ae3d8:288]");
    assert!(sim.drones.channels.on(0x31), "the lock's out channel writes [M 0x1ae3d8:283-289, the case-1 hold]");
}

#[test]
fn a_solved_code_equips_its_key_and_the_sister_keypad_opens_from_the_ring() {
    let (mut scene, li) = open_0a();
    let sim = scene.sim.as_mut().unwrap();
    let sister = sim.drones.locks.iter().enumerate()
        .position(|(i, l)| i != li && l.kind == 0 && l.code == 2)
        .expect("0700000a carries a second keypad on code 2 [D the census]");
    assert!(!sim.drones.inventory.keys[2]);
    let code = port_code(sim, li);
    type_code(sim, &code);
    for _ in 0..5 { idle(sim); }
    assert!(sim.drones.inventory.keys[2], "`Lock_Unlock` equips the key [M 0x1b0650:15]");
    // `Lock_Update` case 0 reads the ring every frame: a held key on the lock's code fires
    // `Lock_Unlock` and state 1 with no press [M 0x1ae3d8:139-147], so the sister opens on its own.
    for _ in 0..5 { idle(sim); }
    assert!(sim.drones.locks[sister].unlocked, "a filled key slot opens the sister lock [M 0x1ae3d8:139-147]");
    assert!(sim.drones.keypad.is_none(), "with no page");
}

#[test]
fn the_wrap_and_the_cap_on_0700000a() {
    let mut scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a");
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.keypad_page_drawn = true;
    let li = find_lock(sim, 1, 0x30);
    use_lock(sim, li);
    assert!(sim.drones.keypad.is_some());
    // Five presses: the fifth wraps the count to 0 [M 0x216810:8-10].
    for d in [1u32, 2, 3, 4, 5] { steer_and_confirm(sim, d); }
    let disp = sim.drones.keypad_display().unwrap();
    assert_eq!(disp.len(), 1, "the 5th press wrapped the count ({disp})");
    for _ in 0..5 { idle(sim); }
    assert!(!sim.drones.locks[li].unlocked, "an incomplete code never unlocks [M 0x1ae3d8's count==cap gate]");
}

#[test]
fn a_wrong_code_never_opens_and_closing_leaves_07000011_s_lock_usable() {
    let mut scene = nf_view::load_with_carry_weapons_difficulty(&iso(), "07000011.bin", false, None, &[], 2)
        .expect("load 07000011 at difficulty 2");
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.keypad_page_drawn = true;
    // 07000011's one kind-0 lock (#0b1: code index 1, out ch 0x12) [D]. Its param-6 tag starts it
    // unlocked on Operative [M 0x1ae1f8:67-69], so the test plays at difficulty 2.
    let li = find_lock(sim, 1, 0x12);
    use_lock(sim, li);
    assert!(sim.drones.keypad.is_some(), "the page arms");
    // Every digit shifted +1 mod 9: differs from the code in every position (the table's digits
    // are '0'..'8' [M 0x1b0548: (Rand % 9) + '0']).
    let code = port_code(sim, li);
    let wrong: Vec<u8> = code.iter().map(|&d| b'0' + (d - b'0' + 1) % 9).collect();
    assert_ne!(wrong, code);
    type_code(sim, &wrong);
    for _ in 0..10 { idle(sim); }
    assert!(!sim.drones.locks[li].unlocked, "a wrong code never unlocks [M 0x1ae3d8's match gate]");
    assert!(sim.drones.keypad.is_some(), "the page stands on a wrong code");
    assert!(!sim.drones.channels.on(0x12), "the out channel stays off");
    tick(sim, Input { keypad_back: true, ..Default::default() });
    for _ in 0..5 { idle(sim); }
    assert!(sim.drones.keypad.is_none(), "△ closes the page [M 0x21d508:29-37]");
    use_lock(sim, li);
    assert!(sim.drones.keypad.is_some(), "the closed page leaves the lock re-armable [M 0x1ae3d8:292-295]");
    type_code(sim, &code);
    for _ in 0..5 { idle(sim); }
    assert!(sim.drones.locks[li].unlocked, "the right code unlocks after the re-arm");
    assert!(sim.drones.channels.on(0x12), "the out channel writes [M the case-1 hold]");
}
