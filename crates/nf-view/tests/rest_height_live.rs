//! Bond's standing height on flat floor, against PCSX2 holds. Opt-in: NF_ISO.
//!
//! The game has no fixed rest height. `Player_Collision` 0x194df0 adds gravity unless the last
//! tick's feet flag (anim+0x60 bit 8) is set, and `Player_FeetOnPoint` 0x194188 sets that flag
//! when the feet ray's hit distance passes `dist - 0.1 <= cc` (disasm 0x19426c-0x194290, cc =
//! anim+0xCC) [M]. Bond's standing cc is 1.0347183 (`ROOT_HEIGHT` [L anim+0xCC]), so a falling Bond stops on the first tick
//! his root is within 1.1347 of the floor and then hovers there: the capsule bottom (root − cc)
//! only touches at 1.0347. The rest height is therefore set by the fall's own tick steps
//! (v += −9.8/60, y += v/60 from rest), not by a constant offset.
//!
//! The live runs: PCSX2 (GDB build), a fresh level load, god mode held, Bond placed with a raw
//! obj+0x30 write, his y read every frame over PINE. The port places Bond after the spawn settle
//! has run out (a placement during the settle hold integrates the held velocity once on release
//! and lands at a different height, a test artifact, not a game rule).

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Loads `level`, idles out the spawn settle, places Bond at `at`, idles `ticks`, returns y.
fn rest(level: &str, at: [f32; 3], ticks: u32) -> f32 {
    let mut w = Walker::new(nf_view::load(&iso(), level, false).unwrap());
    w.wait_enabled(3000, "load");
    for _ in 0..200 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        assert_eq!(sim.bond.settle_hold, 0, "the spawn settle has run out before the placement");
        let cel = sim.world.find_cel(at);
        sim.bond.pos = at;
        sim.bond.prev_pos = at;
        sim.bond.vel = [0.0; 3];
        sim.bond.cel = cel;
        assert_eq!(sim.bond.cc, nf_game::player::ROOT_HEIGHT, "Bond stands at the standing root height");
    }
    for _ in 0..ticks { w.tick(Input::default(), false); }
    w.scene.sim.as_ref().unwrap().bond.pos[1]
}

/// 07000006, the gallery shelf (floor y 45.0005 [D]): placed at y 46.2, the game's Bond falls
/// 7 ticks and rests at 46.1238 [L 2026-11-21, placed (30.0, 46.2, −63.5)].
#[test]
fn the_06_shelf_fall_rests_where_the_games_does() {
    let y = rest("07000006.bin", [30.0, 46.2, -63.5], 60);
    assert!((y - 46.1238).abs() < 1e-4, "the shelf rest is the game's 46.1238 [L], got {y:.5}");
}

/// 07000041, the open floor south of the spawn (floor y 0.0016 [D]): placed at y 1.2, the game
/// rests at 1.1238 [L 2026-11-21, placed (−3.0, 1.2, −43.5)].
#[test]
fn the_41_floor_fall_rests_where_the_games_does() {
    let y = rest("07000041.bin", [-3.0, 1.2, -43.5], 60);
    assert!((y - 1.1238).abs() < 1e-4, "the 41 rest is the game's 1.1238 [L], got {y:.5}");
}

/// The same shelf from y 46.3: the game falls 11 ticks to 46.1203 [L 2026-11-21, placed (30.0,
/// 46.3, −63.5), y read every frame: … 46.2020, 46.1775, 46.1203]. The fall stops one step past
/// the band edge, not at a fixed offset (1.1198 above this floor, against 1.1233 from 46.2).
#[test]
fn the_06_shelf_fall_from_higher_rests_one_step_past_the_band() {
    let y = rest("07000006.bin", [30.0, 46.3, -63.5], 60);
    assert!((y - 46.1203).abs() < 1e-4, "the shelf rest from 46.3 is the game's 46.1203 [L], got {y:.5}");
}
