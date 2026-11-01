//! level-03-fall-through-floor — walk the slot: real input east across the courtyard's west void.
//!
//! The flat feet-ray map found the through-world void x 8.0..8.9, z −19.5..−16.0: a ~1 u slot
//! between the west ground (floor 4.4 at x ≤ 8.0) and the courtyard plateau (floor 5.34 at
//! x ≥ 8.94) with NO floor face at any depth [D 07000003.bin block 0x2E — the probe dump shows no
//! tri in y 3..8 there]. The player capsule (r 0.55 [M Player_Collision 0x194df0]) spans 1.1 u —
//! wider than the slot — so the game bridges it by the rims' edge contacts (Dist2Tri flags 1..7
//! [M 0x10a330 site 0x10a324/0x10a528]). This walk crosses the slot eastbound on real input and
//! samples per tick; the fall (the −5577 symptom) localizes to the slot if it opens.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free probe walk; run with --ignored"]
fn walk_east_across_the_west_slot() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    w.wait(60);
    // To the west ground north of the moat lane (floor 3.3 per the ray map at x ≤ 6.5, z −18).
    for (name, at) in [
        ("courtyard east", [0.0f32, 5.9, -20.0]),
        ("west ground", [5.0f32, 3.5, -18.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }
    eprintln!("start: bond at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    // The wall at (6.35, −15.7..−17.7) blocks the direct east walk ("parks Bond west of a wall"
    // [00310262]); the nav "climbs to y 7.3, airborne" [c118ed46]. Climb it, then cross east.
    w.climb_stairs([7.2, 7.5, -17.7], "up the courtyard wall", 1.2, 900);
    eprintln!("climbed: bond at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    // East across the slot, real input, per-tick samples.
    for (label, target) in [
        ("east to the slot rim (7.6,-18.0)", [7.6f32, 4.5, -18.0]),
        ("across the slot to (9.6,-18.0)", [9.6, 5.5, -18.0]),
    ] {
        let start = w.total_ticks;
        let mut airborne = 0u32;
        loop {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let d = ((target[0] - p[0]).powi(2) + (target[2] - p[2]).powi(2)).sqrt();
            if d < 0.9 || w.total_ticks - start > 700 { break; }
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            let err = desired - sim.bond.yaw;
            let err = err - (err / std::f32::consts::PI).round() * std::f32::consts::PI;
            let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() };
            w.tick(input, false);
            let sim = w.scene.sim.as_ref().unwrap();
            let p2 = sim.bond.pos;
            if !sim.bond.grounded { airborne += 1; } else { airborne = 0; }
            if !sim.bond.grounded || p2[1] < 2.0 {
                eprintln!("  {label} t{} bond ({:.2},{:.2},{:.2}) grounded {} air {airborne} ny {:.2}",
                    w.total_ticks, p2[0], p2[1], p2[2], sim.bond.grounded, sim.bond.floor_ny);
            }
            if p2[1] < -100.0 {
                eprintln!("  THROUGH-WORLD FALL at t{} pos {:?}", w.total_ticks, p2);
                break;
            }
        }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("{label}: ended at {:?} falls {:?}", sim.bond.pos, w.report.falls);
    }
}
