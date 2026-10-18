//! level-14 probe 1: the vent store room jam — walkreal_07000014's walk north from the start
//! #666 (40.81, 2.40, 65.08) wedges at (40.73, 3.11, 59.41) crouched, ungrounded, 0.71 above the
//! floor (fail dump). Diagnostic: the state trace, the geometry along the duct (floor/ceiling
//! segments at each z), and a crouch-held walk attempt. Prints only.

use nf_game::player::Input;
use nf_view::scene::Sim;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn wrap_pi(a: f32) -> f32 {
    let mut a = a;
    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; }
    a
}

#[test]
fn probe_vent_store_room_jam() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    eprintln!("start pos {:?}", sim.bond.pos);
    eprintln!("start crouched {} grounded {} cc {:.3} crouch_drop {:.3} eye {:.3}",
        sim.bond.crouched, sim.bond.grounded, sim.bond.cc, sim.bond.crouch_drop, sim.bond.eye);
    eprintln!("start substate {}", sim.bond.substate);

    // Free-fall settle: what happens with no input at all?
    for k in 0..400 {
        sim.step_bond(Input::default(), false, false, false);
        if (10..26).contains(&k) || k % 60 == 0 {
            let p = sim.bond.pos;
            let ray = sim.bond.cc + 0.6;
            let scoped = sim.world.segment_hit_scoped(sim.bond.cel, p, [p[0], p[1] - ray, p[2]]);
            let flat = sim.world.segment_hit(p, [p[0], p[1] - ray, p[2]]);
            eprintln!("t{k}: y {:.6} crouched {} grounded {} cc {:.4} ray {:.3} dist {:.6} scoped {scoped:?} flat {flat:?}",
                p[1], sim.bond.crouched, sim.bond.grounded, sim.bond.cc, ray, p[1] - 2.4000325);
        }
    }

    // Geometry along the walk line x=40.7, from z=65 down to z=39: floor and ceiling.
    eprintln!("=== geometry along x=40.7 (floor: down-ray from y+3; ceiling: up-ray) ===");
    for z in [64.5f32, 62.0, 60.0, 59.4, 58.0, 56.0, 54.0, 52.0, 50.0, 48.0, 46.0, 44.0, 42.0, 40.5, 39.0] {
        let at = [40.7f32, sim.bond.pos[1] + 2.5, z];
        let floor = sim.world.segment(at, [at[0], at[1] - 12.0, at[2]]).map(|t| at[1] - 12.0 * t);
        let ceil = sim.world.segment(at, [at[0], at[1] + 8.0, at[2]]).map(|t| at[1] + 8.0 * t);
        eprintln!("z {z:6.1}: floor {floor:?} ceiling {ceil:?} gap {}",
            match (floor, ceil) { (Some(f), Some(c)) => format!("{:.2}", c - f), _ => "—".into() });
    }

    // Try the walk north with crouch HELD (the vent crawl).
    eprintln!("=== walk north holding crouch ===");
    for k in 0..600 {
        let p = sim.bond.pos;
        let desired = (40.7 - p[0]).atan2(39.0 - p[2]);
        let err = wrap_pi(desired - sim.bond.yaw);
        sim.step_bond(Input { forward: 1.0, turn: err.clamp(-0.08, 0.08), crouch: true, ..Default::default() }, false, false, false);
        if k % 60 == 0 {
            eprintln!("t{k}: pos ({:.2},{:.2},{:.2}) crouched {} grounded {} substate {}",
                sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.crouched, sim.bond.grounded, sim.bond.substate);
        }
        if sim.bond.pos[2] < 41.0 { eprintln!("REACHED the #301 band at t{k}"); break; }
    }
    eprintln!("final pos {:?}", sim.bond.pos);
}

fn floor_below(sim: &Sim, at: [f32; 3]) -> Option<f32> {
    sim.world.segment(at, [at[0], at[1] - 12.0, at[2]]).map(|t| at[1] - 12.0 * t)
}
