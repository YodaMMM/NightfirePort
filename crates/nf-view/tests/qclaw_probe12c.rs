//! level-12-qclaw-ride probe 3: how the playable floor reaches the skybridge deck band —
//! the z ~19.5 wall's gaps, the doors, and the walk lanes north.

#[path = "common/walker_b2.rs"]
mod walker;
use walker::{Walker, Wp};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[ignore = "level-12-qclaw-ride: the geometry probe — the findings live in its comments and the ride test"]
fn probe12c_reach_the_deck() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("== doors: {}", sim.drones.doors.list.iter()
            .map(|d| format!("#{}@({:.1},{:.1},{:.1})", d.placement, d.pos[0], d.pos[1], d.pos[2]))
            .collect::<Vec<_>>().join(" "));
        // the z-band 18.5..21 wall scan at head height and knee height
        for y in [0.4f32, 1.0, 2.0] {
            let mut row = String::new();
            for xi in 0..25 {
                let x = 4.0 + xi as f32 * 1.0;
                let blocked = sim.world.segment([x, y, 18.5], [x, y, 21.0]).is_some();
                row.push(if blocked { '#' } else { '.' });
            }
            eprintln!("wall y {y}: {row}   (x 4..28, '.' = open)");
        }
        // the deep scan: which xz of the NE strip is floor at all (from y 3 down)
        eprintln!("== NE strip floor map x 6..22 z 20..34 (first hit from y 3; '....' = none) ==");
        for zi in 0..8 {
            let z = 20.0 + zi as f32 * 2.0;
            let mut row = String::new();
            for xi in 0..17 {
                let x = 6.0 + xi as f32 * 1.0;
                let f = sim.world.segment([x, 3.0, z], [x, -3.0, z]).map(|t| 3.0 - t * 6.0);
                row.push_str(&match f { Some(y) => format!("{y:>5.2} "), None => " .... ".to_string() });
            }
            eprintln!("z {z:>4.1}: {row}");
        }
    }

    // walk attempts: the open lane(s) from the spawn side, then north to the deck band.
    let legs: [(&str, Wp); 8] = [
        ("the lobby south leg (green on the route)", Wp::new(14.0, 1.1, 6.0).r(1.2)),
        ("lane x12 z12", Wp::new(12.0, 1.0, 12.0).r(1.2)),
        ("lane x12 z16", Wp::new(12.0, 1.0, 16.0).r(1.2)),
        ("lane x12 z20", Wp::new(12.0, 1.0, 20.0).r(1.2)),
        ("lane x12 z24", Wp::new(12.0, 1.0, 24.0).r(1.2)),
        ("lane x12 z28", Wp::new(12.0, 1.0, 28.0).r(1.2)),
        ("the deck rim (12, 30)", Wp::new(12.0, 1.0, 30.0).r(1.2)),
        ("the office SE (16, 28)", Wp::new(16.0, 1.0, 28.0).r(1.2)),
    ];
    for (label, wp) in legs {
        eprintln!("-- {label}");
        w.walk_to(wp, label);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("   at ({:.2}, {:.2}, {:.2}); stuck {}", p[0], p[1], p[2], w.report.stuck_events.len());
    }
    let _ = Input::default();
}
