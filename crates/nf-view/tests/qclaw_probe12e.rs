//! level-12-qclaw-ride probe 5: the 4.58 walkway's ramp — fine surface scan west/south of the
//! NE strip, then the walk up onto it.

#[path = "common/walker_b2.rs"]
mod walker;
use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[ignore = "level-12-qclaw-ride: the geometry probe — the findings live in its comments and the ride test"]
fn probe12e_the_walkway_ramp() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("== fine surface scan x 0..14 z 18..34 (first hit from y 10; '.' = none) ==");
        for zi in 0..16 {
            let z = 18.0 + zi as f32;
            let mut row = String::new();
            for xi in 0..29 {
                let x = xi as f32;
                let f = sim.world.segment([x, 10.0, z], [x, -3.0, z]).map(|t| 10.0 - t * 13.0);
                row.push_str(&match f { Some(y) => format!("{y:>5.2} "), None => "  ... ".to_string() });
            }
            eprintln!("z {z:>4.1}: {row}");
        }
    }
    // The walk: to the strip, then attempt the ramp/band legs.
    let legs: [(&str, Wp); 9] = [
        ("the shaft lobby", Wp::new(3.0, 1.0, 10.0).r(1.2)),
        ("lane north (6, 14)", Wp::new(6.0, 1.0, 14.0).r(1.2)),
        ("the lane gap (9.5, 20.5)", Wp::new(9.5, 1.0, 20.5).r(1.2)),
        ("the NE strip (10, 24)", Wp::new(10.0, 1.0, 24.0).r(1.2)),
        ("the ramp try (8, 26)", Wp::new(8.0, 1.0, 26.0).r(1.2)),
        ("the ramp try (7, 25)", Wp::new(7.0, 1.0, 25.0).r(1.2)),
        ("the ramp try (6, 24)", Wp::new(6.0, 1.0, 24.0).r(1.2)),
        ("the ramp try (5, 23)", Wp::new(5.0, 1.0, 23.0).r(1.2)),
        ("the walkway (10, 28)", Wp::new(10.0, 1.0, 28.0).r(1.2)),
    ];
    for (label, wp) in legs {
        let ok = w.try_walk_to(wp, label);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("{label}: ok {ok} at ({:.2}, {:.2}, {:.2}) stuck {}", p[0], p[1], p[2], w.report.stuck_events.len());
        if !ok {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [wp.at[0], wp.at[1] + 0.05, wp.at[2]];
            sim.bond.vel = [0.0; 3];
            w.wait(10);
            eprintln!("   (teleport stand-in applied)");
        }
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("final ({:.2}, {:.2}, {:.2})", p[0], p[1], p[2]);
}
