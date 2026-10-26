//! level-12-qclaw-ride probe 2: standability of the grapple-aim stances and the walk from the
//! spawn into the south-east area (the skybridge deck band).

#[path = "common/walker_b2.rs"]
mod walker;
use walker::{Walker, Wp, wrap_pi};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn floor_at(world: &nf_game::world::CollisionWorld, x: f32, z: f32, from_y: f32) -> Option<f32> {
    world.segment([x, from_y, z], [x, -3.0, z]).map(|t| from_y - t * (from_y + 3.0))
}

#[ignore = "level-12-qclaw-ride: the geometry probe — the findings live in its comments and the ride test"]
fn probe12b_stand_and_walk() {
    let iso = iso();
    // 1. Standability: teleport onto each candidate surface, tick, see what holds.
    eprintln!("== standability (place at surf+0.05, 45 ticks) ==");
    for (label, x, z) in [
        ("band (12, 29)", 12.0f32, 29.0), ("band (13, 30)", 13.0, 30.0),
        ("band (11, 28)", 11.0, 28.0), ("band (14, 31)", 14.0, 31.0),
        ("deck (11, 31)", 11.0, 31.0), ("deck (10, 31)", 10.0, 31.0),
        ("deck (8, 33)", 8.0, 33.0), ("deck (10, 30)", 10.0, 30.0),
        ("band far (7, 24)", 7.0, 24.0), ("office SE (16, 30)", 16.0, 30.0),
    ] {
        let mut scene = nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap();
        let surf = floor_at(&scene.sim.as_ref().unwrap().world, x, z, 10.0);
        let Some(surf) = surf else { eprintln!("{label}: no surface"); continue };
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.bond.pos = [x, surf + 0.05, z];
            sim.bond.vel = [0.0; 3];
            for _ in 0..45 { sim.step_bond(Input::default(), false, false, false); }
        }
        let sim = scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let grounded = sim.bond.grounded;
        let held = (p[0] - x).abs() < 0.6 && (p[2] - z).abs() < 0.6 && p[1] > surf - 1.2;
        eprintln!("{label} surf {surf:.2} -> pos ({:.2}, {:.2}, {:.2}) grounded {grounded} held {held}", p[0], p[1], p[2]);
    }

    // 2. The walk: spawn -> the SE area. Real input through the Walker.
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    let targets: [(&str, Wp); 6] = [
        ("SE corner (20, 26)", Wp::new(20.0, 1.0, 26.0).r(1.2)),
        ("SE mid (18, 30)", Wp::new(18.0, 1.0, 30.0).r(1.2)),
        ("SE east (17, 31)", Wp::new(17.0, 1.0, 31.0).r(1.2)),
        ("under-point rim (14, 30)", Wp::new(14.0, 1.0, 30.0).r(1.2)),
        ("SE deep (16, 28)", Wp::new(16.0, 1.0, 28.0).r(1.2)),
        ("deck edge (12, 30)", Wp::new(12.0, 1.0, 30.0).r(1.2)),
    ];
    for (label, wp) in targets {
        eprintln!("-- walk attempt: {label}");
        w.walk_to(wp, label);
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        eprintln!("   at ({:.2}, {:.2}, {:.2}) after {}; stuck {}", p[0], p[1], p[2], w.total_ticks, w.report.stuck_events.len());
        if (p[0] - wp.at[0]).powi(2) + (p[2] - wp.at[2]).powi(2) < 4.0 { break; }
    }
    let _ = (wrap_pi(0.0), gp_marker());
}

fn gp_marker() -> f32 { 0.0 }
