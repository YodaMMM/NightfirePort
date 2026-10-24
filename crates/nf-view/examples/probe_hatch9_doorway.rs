//! Scratch (level-0a-office-hatch-9): the doorway base and the corner.
//!
//! 1. Dumps the world triangles whose centroid sits in the doorway-base box (x −8.8..−8.1,
//!    z 15.9..16.6, y −0.1..1.5) and the corner box (x −18.2..−17.2, z 4.2..5.1, y 0..1.5).
//! 2. Walks the glide test's doorway line (the lobby-south leg then z 16.29 west through the
//!    doorway) with the per-tick root trace, so the climb (game [L]: peak 1.18, settle 1.1147)
//!    can be compared tick by tick (run with NF_CAPS_LOG=1 for the per-contact pushes).
//! 3. Teleports to the wedge corner (−17.87, 1.13, 4.87) and holds forward toward (−17.5, 2.5)
//!    (the game's own hold-forward line [L fresh boot]: through in ~25 frames, y 1.127→1.120).
fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    let cold = nf_view::load_with_carry(&path, "0700000a.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut scene = nf_view::load_with_carry(&path, "0700000a.bin", false, Some(&save)).expect("warm");
    // ── 1. the triangle dumps ──────────────────────────────────────────────────────────────
    for (name, lo, hi) in [
        ("doorway", [-8.8f32, -0.1, 15.9], [-8.1, 1.5, 16.6]),
        ("corner", [-18.2, 0.0, 4.2], [-17.2, 1.5, 5.1]),
    ] {
        let tris = scene.sim.as_ref().unwrap().world.dump_tris_in(lo, hi);
        println!("== {name} box {lo:?}..{hi:?}: {} tris", tris.len());
        for t in &tris {
            let e1 = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
            let e2 = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
            let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
            println!("  tri ({:.3},{:.3},{:.3}) ({:.3},{:.3},{:.3}) ({:.3},{:.3},{:.3}) n=({:.3},{:.3},{:.3})",
                t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2], n[0] / l, n[1] / l, n[2] / l);
        }
    }
    // ── 2. the doorway walk (the glide test's line) ────────────────────────────────────────
    println!("== the doorway walk: lobby south then west along z 16.29");
    walk_line(&mut scene, [-3.9, 0.0, 15.5], 600, "lobby south");
    walk_line(&mut scene, [-8.45, 0.4, 16.29], 900, "the doorway base line");
    walk_line(&mut scene, [-10.5, 0.4, 16.29], 900, "the west hall");
    // ── 3. the corner hold-forward (teleported; the physics diagnosis, not the route) ──────
    println!("== the corner: teleport (−17.87, 1.13, 4.87) hold toward (−17.5, 2.5)");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [-17.87, 1.13, 4.87];
        sim.bond.vel = [0.0; 3];
        let d = [-17.5 - -17.87, 2.5 - 4.87];
        sim.bond.yaw = d[0].atan2(d[1]);
    }
    walk_line(&mut scene, [-17.5, 1.3, 2.5], 300, "the corner line");
}

/// Face the target and hold forward, printing the root trace every tick near the regions of
/// interest (x −9.2..−7.9 = the doorway band; the whole corner leg).
fn walk_line(scene: &mut Box<nf_view::Scene>, to: [f32; 3], max_ticks: u32, label: &str) {
    for t in 0..max_ticks {
        let (pos, yaw) = {
            let sim = scene.sim.as_ref().unwrap();
            (sim.bond.pos, sim.bond.yaw)
        };
        let dx = to[0] - pos[0];
        let dz = to[2] - pos[2];
        let dist = (dx * dx + dz * dz).sqrt();
        if dist < 0.35 { println!("[{label}] arrived t{t} at ({:.4},{:.4},{:.4})", pos[0], pos[1], pos[2]); return; }
        let desired = dx.atan2(dz);
        let err = wrap_pi(desired - yaw);
        let in_band = (-9.2..=-7.9).contains(&pos[0]) || label.contains("corner");
        let input = nf_game::player::Input { forward: 1.0, turn: err.clamp(-0.08, 0.08), ..Default::default() };
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
        if in_band {
            let sim = scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            println!("[{label}] t{t} pos=({:.4},{:.4},{:.4}) grounded={} floor={:?}",
                p[0], p[1], p[2], sim.bond.grounded, sim.world.floor_below(p, 3.0));
        }
    }
    let p = scene.sim.as_ref().unwrap().bond.pos;
    println!("[{label}] TIMED OUT at ({:.4},{:.4},{:.4})", p[0], p[1], p[2]);
}

fn wrap_pi(d: f32) -> f32 {
    let mut d = d % std::f32::consts::TAU;
    if d > std::f32::consts::PI { d -= std::f32::consts::TAU; }
    if d < -std::f32::consts::PI { d += std::f32::consts::TAU; }
    d
}
