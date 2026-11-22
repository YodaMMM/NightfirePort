//! 07000016's ring, the north wall: what Bond stands on there. Opt-in: NF_ISO.
//!
//! There is no object on the ring's north wall. Along the wall the ring floor (y 5.708) meets a
//! kerb slope (record n (0, 0.7886, −0.6147), d −50.5765 [D]) that rises from z −74.944 to the
//! wall foot at z −74.103 (y 6.364), and both are the static `stealthship` room geometry (map 63
//! object 99, placement 99) [D]. A PCSX2 trace at the kerb foot (a breakpoint on
//! `Player_CollisionHandler` 0x1949a8 reading obj+0xD0's contact list, the feet hit copy at
//! anim+0x00..0x60 and HITTEST+0x750's push) found no object contacts near Bond and the feet hit
//! on the static cel, object pointer 0 [L 2026-11-21].
//!
//! The game's Bond, placed at the kerb foot and walked west, holds y 6.88037, z −74.96214 every
//! frame from x 13.2 to x 6.5, with a zero push (HITTEST+0x10) and the feet flag set although the
//! feet hit is 1.172 below the root (outside the 1.1347 band: the capsule's contact with the
//! up-facing kerb sets it) [L]. Whether Bond rides up onto the kerb depends on the z he reaches
//! the kerb foot at: from the wall rest the game's Bond arrived at z −74.9568 and climbed the kerb
//! at x 11.45 (the s/collide-push-2 run), while from z −74.96214 it never does [L].
//!
//! The rest sits on the knife edge |v| ≈ r of `Intersect_CylGeom` 0x109da0's push arm, and the
//! pick bit that sets the feet flag lives inside that arm [M :447-513]. With round-to-nearest
//! floats the port's sub-ulp pushes crept the root up one ulp a tick until |v| reached r, the
//! contact dropped, one gravity tick fell, and the slope's push walked Bond south and down
//! (6.8567, −74.9925 by x 6.5). The game's VU/FPU steps round toward zero [M ps2float]: the
//! tiny push no longer moves the root and the contact holds every tick.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_kerb_foot_walk_holds_the_games_line() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000016.bin", false).unwrap());
    w.wait_enabled(3000, "load");
    for _ in 0..200 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let at = [13.2, 6.95, -74.957];
        let cel = sim.world.find_cel(at);
        sim.bond.pos = at;
        sim.bond.prev_pos = at;
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
        sim.bond.cel = cel;
    }
    for _ in 0..40 { w.tick(Input::default(), false); }
    for _ in 0..70 { w.tick(Input { forward: 1.0, ..Default::default() }, false); }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    assert!(p[0] < 7.0, "Bond walked the kerb foot west, got {p:?}");
    assert!((p[1] - 6.88037).abs() < 5e-4 && (p[2] - (-74.96214)).abs() < 5e-4,
        "the game holds (_, 6.88037, −74.96214) along the kerb foot [L], got {p:?}");
}
