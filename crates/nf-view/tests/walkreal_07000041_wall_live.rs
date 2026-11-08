//! submap-placements pin: 07000041's class-0xd5 "1-Sided Thing" (plc 86 on the kind-1 main map,
//! object hash ffffffff "1-Sided Thing") blocks Bond from BOTH sides through real input.
//!
//! The game's record: `parsemap_create_dynamic_objects` 0x1d04f0 case 0xd5 → `OneSided_Create`
//! 0x1e7ce8 — a control object (obj+0xD8 = the placement's celglist, obj+0xff = 0x4c) reached only
//! through the flag-1 pass `ResetMap_Load` 0x1d6850 over the last block-0x1a (the main map) [M].
//! Its glist collision rides the pick, so Bond meets it from both sides; the one-sidedness is the
//! DRAW (`OneSided_Update` 0x1e7d58 toggles obj+0xF0 bit 0x10 by viewer side — the port's
//! per-triangle backface cull is the same job, scene.rs). The placement carries no 0x8000, so the
//! static-cel pass (parsemap_block_map_data_static 0x1d1998:92) never builds it — before the
//! submap-placements fold the port's world had a hole there (a probe's 4/4 rays through the wall
//! plane returned None). The wall: x −5.07..−0.95, y 0.51..4.60, plane z −44.90 [D placement +
//! the object's 2-tri collision block]; flat floor y 0 all around [D the disc tri dump].
//!
//! Real input from the level start (no teleports): the airlock start #0 (27.22, 1.10, −30.15
//! [D #0]) → the station floor → the north face (approach −z, stall at contact) → around the west
//! edge (x −7, clear of the wall's −5.07 edge) → the south face (approach +z, stall at contact).
//! Opt-in: NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Walk to the wall's north face and stall on it; the capsule rests at contact distance and never
/// crosses the plane.
#[test]
fn the_one_sided_wall_blocks_from_the_north() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000041.bin", false).unwrap());
    w.wait_enabled(600, "the wrapper's load");
    assert!(
        (w.scene.sim.as_ref().unwrap().bond.pos[0] - 27.22).abs() < 0.5,
        "the wrapper starts Bond at the airlock #0 [D #0]"
    );
    w.walk_route(&[
        ("off the airlock west", Wp::new(24.0, 1.1, -31.0).r(1.4)),
        ("the station floor", Wp::new(19.0, 1.1, -32.5).r(1.4)),
        ("south-west lane", Wp::new(2.0, 1.1, -40.0).r(1.4)),
        ("north of the wall", Wp::new(-3.0, 1.1, -43.2).r(1.0)),
        // The micro-leg lines Bond's facing up with −z exactly on the wall's centre x (−3.01),
        // 2.06 units from either edge — the push cannot slip around.
        ("align on the wall's centre", Wp::new(-3.0, 1.1, -43.5).r(0.3)),
    ]);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - (-3.0)).abs() < 0.6 && p[2] > -44.4 && p[2] < -43.0,
            "Bond stands centred north of the wall, at {:?}", p);
    }
    // Push into the wall: the walker steers at the point THROUGH the wall (keeps Bond on the
    // wall's centre x, no edge to slip around), the cap ends the push. The capsule (r ≈ 0.35)
    // must rest north of the plane −44.90 + contact, never cross.
    let z_before = w.scene.sim.as_ref().unwrap().bond.pos[2];
    let arrived = w.walk_slope([-3.0, 1.1, -46.0], "push into the wall (expect blocked)", 0.9, 400);
    assert!(!arrived, "the push cannot arrive through the wall");
    let sim = w.scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    assert!((p[0] - (-3.0)).abs() < 0.8, "no sideways drift around an edge, at x {:.3}", p[0]);
    assert!(p[2] > -45.45, "Bond stalls north of the wall plane (contact −45.25), at z {:.3}", p[2]);
    assert!(p[2] < z_before - 0.3, "Bond walked INTO the wall (z {:.3} → {:.3})", z_before, p[2]);
    assert!(p[2] > -44.95, "Bond never crossed the plane, at z {:.3}", p[2]);
}

/// The same wall from the south face: around the west edge, then push +z into it.
#[test]
fn the_one_sided_wall_blocks_from_the_south() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000041.bin", false).unwrap());
    w.wait_enabled(600, "the wrapper's load");
    w.walk_route(&[
        ("off the airlock west", Wp::new(24.0, 1.1, -31.0).r(1.4)),
        ("the station floor", Wp::new(19.0, 1.1, -32.5).r(1.4)),
        ("south-west lane", Wp::new(2.0, 1.1, -40.0).r(1.4)),
        ("north of the wall", Wp::new(-3.0, 1.1, -43.2).r(1.0)),
        ("west, clear of the wall's edge", Wp::new(-7.0, 1.1, -43.4).r(1.0)),
        ("south past the plane", Wp::new(-7.0, 1.1, -46.4).r(1.0)),
        ("east on the south face", Wp::new(-3.0, 1.1, -46.4).r(1.0)),
        // The micro-leg lines Bond's facing up with +z exactly on the wall's centre x (−3.01).
        ("align on the wall's centre", Wp::new(-3.0, 1.1, -46.1).r(0.3)),
    ]);
    // Push +z (north) into the wall's south face: the same steered push, the cap ends it.
    let z_before = w.scene.sim.as_ref().unwrap().bond.pos[2];
    let arrived = w.walk_slope([-3.0, 1.1, -43.4], "push into the wall's south face (expect blocked)", 0.9, 400);
    assert!(!arrived, "the push cannot arrive through the wall");
    let sim = w.scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    assert!((p[0] - (-3.0)).abs() < 0.8, "no sideways drift around an edge, at x {:.3}", p[0]);
    assert!(p[2] < -44.35, "Bond stalls south of the wall plane (contact −44.55), at z {:.3}", p[2]);
    assert!(p[2] > z_before + 0.3, "Bond walked INTO the wall from the south (z {:.3} → {:.3})", z_before, p[2]);
    assert!(p[2] < -44.85, "Bond never crossed the plane northward, at z {:.3}", p[2]);
}
