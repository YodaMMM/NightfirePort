//! void-recovery: the game's out-of-cel recovery on the real 07000003 — the moat-bend void
//! strips the door-walk-03b round mapped (x ≈ 11.1-13.4 and 15.6-17.5+, z −30.4..−35.6) and the
//! live PCSX2 measurements this branch reproduces:
//!   | spot                | PCSX2                                     | old port                |
//!   | (14.5, 6.0, −32)    | stands, y 6.28                            | stands, y 6.28          |
//!   | (16.5, 6.3, −33)    | pushed to x 15.99, stands, y 6.23         | falls forever, y −160   |
//!   | (12.0, 6.3, −32)    | back at the level start inside 1 s        | falls forever, cel None |
//! The game's mechanism [M `control_handle_cel_change` 0x133288:186-231]: an object that left its
//! cel's box with no portal crossing and no `build_FindCel` 0x1c8cb0 hit resets `pos := obj+0x40`
//! (the move segment's from = `Bond::prev_pos`) clamped per-axis into the OLD cel's box
//! `cel+0x40..0x58` minus the world-sphere radius 1.0576 [`Bond::SPHERE_RADIUS`, L ee_start_05
//! M_CONTROL+0x8C] — `if v < min + r → min + r else min(v, max − r)` — and relinks into the old
//! cel. The (16.5) push is that clamp: cel 8's max x 17.04 − 1.0576 = 15.98 ≈ the measured 15.99,
//! with the ground disc (r 0.4) reaching back onto the floor edge at x 15.59. The (12.0) reset is
//! the same arm with `obj+0x40` still holding the pre-teleport position (the poke writes obj+0x30
//! only): the level start. An UNLINKED object gets no recovery [M 0x133288:63-73].

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const START: [f32; 3] = [-8.85, 6.41, -23.19];

/// The (12.0, 6.3, −32) poke — the west void strip (no room cel's bbox holds it, `find_cel`
/// None): the game returns Bond to the pre-teleport position inside 1 s [D live]. The poke
/// mirrors the live probe: a direct obj+0x30 write with obj+0x40 and obj+0x20 untouched,
/// then real ticks.
#[test]
fn poke_into_the_west_void_returns_to_the_poke_origin() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000003.bin", false).expect("load 07000003"));
    w.wait(60);
    let (old_cel,) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.bond.cel,)
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [12.0, 6.3, -32.0];
    }
    w.wait(90);
    let sim = w.scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    assert!((p[0] - START[0]).abs() < 0.15 && (p[1] - START[1]).abs() < 0.15 && (p[2] - START[2]).abs() < 0.15,
        "the game returns a void-poked Bond to the pre-teleport position [D live; M 0x133288:186-231], got {p:?}");
    assert_eq!(sim.bond.cel, old_cel, "the recovery relinks into the OLD cel [M 0x133288:233-261]");
}

/// The (16.5, 6.3, −33) poke — the east void strip inside cel 8's bbox (x 13.01-17.04): the game
/// pushes Bond west to x 15.99 = 17.04 − 1.0576 and leaves him standing at y 6.23 [D live]. The
/// port's clamp lands x 15.98; the residual against the game's stable stand is the ground-contact
/// model (the port grounds on a centre ray, the game on the capsule base) — the assert bounds the
/// recovered state: linked to cel 8, above the void, west of the poke.
#[test]
fn poke_into_the_east_void_recovers_into_cel_8() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000003.bin", false).expect("load 07000003"));
    w.wait(60);
    let cel8 = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.world.find_cel([16.5, 6.3, -33.0]).expect("cel 8's bbox holds the east strip's top")
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [16.5, 6.3, -33.0];
    }
    w.wait(360);
    let sim = w.scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    assert_eq!(sim.bond.cel, Some(cel8), "the recovery relinks into cel 8 [M 0x133288:233-261]");
    assert!(p[1] > 5.0, "the game never lets the poked Bond fall into the void [D live], got y {:.2}", p[1]);
    assert!(p[0] < 16.4 && p[0] > 15.4,
        "the clamp caps x at cel 8's max x − 1.0576 = 15.98 [M 0x133288:193-201], got x {:.2}", p[0]);
}

/// The (14.5, 6.0, −32) poke — the mid strip's real floor: stands in both [D live]. A guard
/// against the recovery firing where the game stands.
#[test]
fn poke_on_the_mid_strip_still_stands() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000003.bin", false).expect("load 07000003"));
    w.wait(60);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [14.5, 6.0, -32.0];
    }
    w.wait(120);
    let sim = w.scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    assert!(p[1] > 6.0 && p[1] < 6.6, "stands on the strip's floor y 5.25 + offset [D live], got y {:.2}", p[1]);
    assert!(sim.bond.cel.is_some(), "linked (cel 8's bbox holds the strip)");
}

/// The real-input walk to the moat edge (no teleport after the spawn): the west band's east end
/// (9.95, 5.84, −34.41) is the rerouted walk's own anchor (door-walk-03b). The honest real-input
/// statement: at FLOOR LEVEL the moat wall stops the capsule at the band's edge (x 10.36 + r 0.4
/// = the wall face 10.75) — a walking player cannot cross into the west strip here, in the port
/// or (same wall, same capsule) in the game; the strips are entered from the UPPER perches (the
/// building's upper-floor row to its garden end, the door top — door-walk-03b §3), where the fall
/// leaves every room cel's bbox (the probe: 0 bbox holders at (12.0, 6.3, −32)) and the game's
/// recovery WOULD run [M: `build_FindCel` 0x1c8cb0 miss → the 0x133288 default arm]. The poke
/// tests above prove that arm against the two measured spots.
#[test]
fn walk_off_the_west_band_edge_recovers() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000003.bin", false).expect("load 07000003"));
    w.wait(60);
    // The spawn → the west lawn → the west band (the rerouted walk's own anchors), all real nav.
    w.walk_nav_to([0.0, 5.9, -30.0], "the west lawn", 1.6);
    w.walk_nav_to([6.87, 5.84, -33.47], "the west band west anchor", 1.2);
    w.walk_nav_to([9.95, 5.84, -34.41], "the band's east end (the 03b anchor)", 0.5);
    let before = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.bond.pos, sim.bond.cel)
    };
    eprintln!("[void] at the band east end: pos {:?} cel {:?}", before.0.map(|v| (v * 100.0).round() / 100.0), before.1);
    assert!(before.1.is_some(), "the band's east end is inside cel 7's box");
    // Real forward input off the edge: east across x 10.75 (the band's floor edge) and 10.91
    // (cel 7's box edge) into the strip. The recovery must pull Bond back into cel 7's box
    // (x := 9.85, y := 6.40, then the ground snap onto the band) every cycle.
    for t in 0..240 {
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
        if t % 60 == 59 {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[void] east tick {}: pos {:?}", t + 1, sim.bond.pos.map(|v| (v * 100.0).round() / 100.0));
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    assert!(p[1] > 4.5, "never below the band [M 0x133288:186-231], got y {:.2} at {:?}", p[1], p);
    assert!(p[0] < 10.8, "the moat wall stops the capsule at the band's edge (face 10.75, r 0.4) [D raw scan], got x {:.2}", p[0]);
    assert_eq!(sim.bond.cel, before.1, "still linked to cel 7");
    assert!(w.report.falls.is_empty(), "no void fall: {:?}", w.report.falls);
}
