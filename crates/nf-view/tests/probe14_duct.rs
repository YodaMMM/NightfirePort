//! level-14 probe 2: the duct blocker at z≈59.41 — the walkreal wedge point. Scans the duct
//! cross-section at capsule height, lists the script entities (the Vent_StoreRoom_Graphic SPs)
//! near the path, and tries crawling past with a smaller y (the game's duct clearance). Prints only.

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
fn probe_duct_blocker() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..120 { sim.step_bond(Input::default(), false, false, false); }

    // Horizontal rays inside the duct at crouch-capsule height (y 2.95 centre, ±0.55 sphere).
    eprintln!("=== horizontal rays at y 2.95 / 3.3 / 3.5, x 40.7, z 60.5 → 55.0 ===");
    for y in [2.95f32, 3.3, 3.5] {
        for z in [60.5f32, 60.0, 59.8, 59.6, 59.5, 59.42, 59.3, 59.0, 58.5, 57.0, 55.0] {
            let from = [40.7f32, y, z];
            let hits = |dir: [f32; 3]| sim.world.segment(from, [from[0] + dir[0], from[1] + dir[1], from[2] + dir[2]])
                .map(|t| format!("{:.2}", t * 3.0));
            eprintln!("y {y} z {z:6.2}: +z {} −z {} +x {} −x {}",
                hits([0.0, 0.0, 3.0]).unwrap_or("—".into()),
                hits([0.0, 0.0, -3.0]).unwrap_or("—".into()),
                hits([3.0, 0.0, 0.0]).unwrap_or("—".into()),
                hits([-3.0, 0.0, 0.0]).unwrap_or("—".into()));
        }
        break; // one height row is enough for the blocker map; keep the log tight
    }

    // The duct floor and ceiling INSIDE (down-ray from y 3.5, up-ray from y 2.9).
    eprintln!("=== interior floor (down from 3.5) / ceiling (up from 2.9) ===");
    for z in [61.0f32, 60.0, 59.6, 59.42, 59.2, 58.8, 58.0, 56.0, 54.5, 53.5, 52.5, 51.0] {
        let floor = sim.world.segment([40.7f32, 3.5, z], [40.7, 3.5 - 8.0, z]).map(|t| 3.5 - 8.0 * t);
        let ceil = sim.world.segment([40.7f32, 2.9, z], [40.7, 2.9 + 6.0, z]).map(|t| 2.9 + 6.0 * t);
        eprintln!("z {z:6.1}: floor {floor:?} ceiling {ceil:?}");
    }

    // Script entities near the duct (the Vent_StoreRoom_Graphic SPs, mode 64).
    eprintln!("=== script players 0..40 (placement, scripts, modes, state) ===");
    for (i, p) in sim.drones.scripts.players.iter().take(40).enumerate() {
        let pos = nf_game::drone::Drones::script_obj_pos(&sim.drones.scripts, &sim.data(), i);
        eprintln!("sp{i}: placement {} scripts {:?} modes {:?} state {} activated {} hidden {} pos {pos:?}",
            p.placement, p.scripts, p.modes, p.state, p.activated, p.hidden);
    }

    // Crawl attempt with forward only (no turn): does Bond pass z 59.41 alone?
    eprintln!("=== crawl forward from the settle, no steering ===");
    for k in 0..300 {
        sim.step_bond(Input { forward: 1.0, crouch: true, ..Default::default() }, false, false, false);
        if k % 30 == 0 {
            eprintln!("t{k}: ({:.2},{:.2},{:.2}) grounded {} substate {}",
                sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded, sim.bond.substate);
        }
        if (80..140).contains(&k) && k % 10 == 0 {
            eprintln!("t{k}: ({:.3},{:.3},{:.3}) vel ({:.3},{:.3},{:.3}) grounded {} floor_ny {:.2} cel {:?}",
                sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2],
                sim.bond.vel[0], sim.bond.vel[1], sim.bond.vel[2],
                sim.bond.grounded, sim.bond.floor_ny, sim.bond.cel);
        }
        if sim.bond.pos[2] < 57.0 { eprintln!("PASSED the wedge at t{k}"); break; }
    }
    let _ = wrap_pi(0.0);
}
