//! los_probe11 — stream-batch-5's blind-guard hunt on 07000011 (the d2 idle-Bond stall).
//!
//! Loads the level at d2, idles Bond at the spawn (the start_corridor d2 test's exact input),
//! and once the corridor guards go blind (`unseen_ticks > 15`) while Bond stands visible,
//! dumps each failing sight ray: from (the clamped bone/chest point), to (the guard's head),
//! the blocking triangle, its material byte and the hit fraction — via the world's
//! `los_debug_hit` (the same masked walk `line_of_sight_masked` runs).
//!
//! Run: NF_ISO=... cargo run --release -p nf-view --example los_probe11

use nf_view::scene::Scene;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "07000011.bin", false, None, 2)
        .expect("load 07000011 d2");
    let mask = nf_game::world::LOS_GUARD_VISION;

    for tick in 0..12_000u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if tick % 200 != 0 { continue; }
        let sim = scene.sim.as_ref().unwrap();
        let bond = &sim.bond;
        let mut line = format!("t{tick} bond ({:.2},{:.2},{:.2}) hp {:.1} |", bond.pos[0], bond.pos[1], bond.pos[2], sim.drones.health.health);
        for dr in sim.drones.list.iter().filter(|d| d.present && d.team == 1 && (d.placement == 126 || d.placement == 127)) {
            line.push_str(&format!(" p{} st{:#x} seen{} unt{} pos({:.2},{:.2},{:.2}) |",
                dr.placement, dr.state, dr.seen, dr.unseen_ticks, dr.pos[0], dr.pos[1], dr.pos[2]));
        }
        eprintln!("{line}");
        // The stall probe: any blind corridor guard gets its ray dumped.
        for dr in sim.drones.list.iter().filter(|d| d.present && d.team == 1 && (d.placement == 126 || d.placement == 127)) {
            if dr.unseen_ticks <= 15 { continue; }
            let eye = [dr.pos[0], dr.pos[1] + 0.6, dr.pos[2]];
            let bones = sim.drones.bond_view_bones;
            let from = match bones { Some(b) => nf_game::drone::clamp_to_cylinder(bond.pos, b[0]), None => bond.pos };
            let cel = bond.cel;
 if let Some(bs) = sim.drones.bond_view_bones {
                for (i, bp) in bs.iter().enumerate() {
                    let f = nf_game::drone::clamp_to_cylinder(bond.pos, *bp);
                    match sim.world.los_debug_hit(f, eye, mask) {
                        Some((t, tri, m)) => eprintln!("  [bone{}] from ({:.3},{:.3},{:.3}) t{:.3} mat {:#04x} tri ({:.2},{:.2},{:.2})({:.2},{:.2},{:.2})({:.2},{:.2},{:.2})",
                            i, f[0], f[1], f[2], t, m, tri[0][0], tri[0][1], tri[0][2], tri[1][0], tri[1][1], tri[1][2], tri[2][0], tri[2][1], tri[2][2]),
                        None => eprintln!("  [bone{}] from ({:.3},{:.3},{:.3}) CLEAR", i, f[0], f[1], f[2]),
                    }
                }
            }
            match sim.world.los_debug_hit_scoped(cel, from, eye, mask) {
                Some((t, tri, m)) => {
                    eprintln!("  BLOCK p{} ray ({:.3},{:.3},{:.3})->({:.3},{:.3},{:.3}) t{:.3} mat {:#04x} tri ({:.3},{:.3},{:.3}) ({:.3},{:.3},{:.3}) ({:.3},{:.3},{:.3})",
                        dr.placement, from[0], from[1], from[2], eye[0], eye[1], eye[2], t, m,
                        tri[0][0], tri[0][1], tri[0][2], tri[1][0], tri[1][1], tri[1][2], tri[2][0], tri[2][1], tri[2][2]);
                }
                None => eprintln!("  CLEAR? p{} ray t-none but unseen {}", dr.placement, dr.unseen_ticks),
            }
        }
    }
}
