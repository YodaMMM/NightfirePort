//! level-12-qclaw-ride probe 4: the walk to the 4.58 walkway through the z 18.5..21 open lane
//! (x 8..11), then the ride's start stance. try_walk_to everywhere (no panic on a stuck leg).

#[path = "common/walker_b2.rs"]
mod walker;
use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[ignore = "level-12-qclaw-ride: the geometry probe — the findings live in its comments and the ride test"]
fn probe12d_walk_to_the_walkway() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    // The tower2 route to the console first (the known-green legs), then north-east through
    // the open lane to the NE strip.
    let legs: [(&str, Wp); 10] = [
        ("the lobby south leg", Wp::new(14.0, 1.1, 6.0).r(1.2)),
        ("the floor south 2", Wp::new(10.0, 1.1, 2.0).r(1.2)),
        ("back to the lobby", Wp::new(12.0, 1.1, 6.0).r(1.2)),
        ("the shaft lobby", Wp::new(3.0, 1.0, 10.0).r(1.2)),
        ("lane north (6, 14)", Wp::new(6.0, 1.0, 14.0).r(1.2)),
        ("lane north (8, 18)", Wp::new(8.0, 1.0, 18.0).r(1.2)),
        ("the lane gap (9.5, 20.5)", Wp::new(9.5, 1.0, 20.5).r(1.2)),
        ("the NE strip (10, 24)", Wp::new(10.0, 1.0, 24.0).r(1.2)),
        ("the NE strip (11, 27)", Wp::new(11.0, 1.0, 27.0).r(1.2)),
        ("the walkway edge (11.5, 29)", Wp::new(11.5, 1.0, 29.0).r(1.2)),
    ];
    for (label, wp) in legs {
        let ok = w.try_walk_to(wp, label);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("{label}: ok {ok} at ({:.2}, {:.2}, {:.2}) stuck {}", p[0], p[1], p[2], w.report.stuck_events.len());
        if !ok {
            // one teleport stand-in per failed leg keeps the walk honest about the rest
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [wp.at[0], wp.at[1] + 0.05, wp.at[2]];
            sim.bond.vel = [0.0; 3];
            w.wait(10);
            eprintln!("   (teleport stand-in applied)");
        }
    }
    // Where does Bond end up relative to the walkway, and can he step onto the 4.58 band?
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("final ({:.2}, {:.2}, {:.2})", p[0], p[1], p[2]);
    // step-up attempt: the walkway band at (12, 30) is 4.58; a jump from the strip below?
    let sim = w.scene.sim.as_ref().unwrap();
    let up = sim.world.segment([12.0, 10.0, 30.0], [12.0, -3.0, 30.0]).map(|t| 10.0 - t * 13.0);
    eprintln!("surface at (12, 30) from y10: {up:?}");
}
