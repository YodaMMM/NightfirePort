//! level-14 probe 3: what stands at the z≈59.41 wedge — every script object and every dynamic
//! part near the duct, plus the world triangles the capsule push actually reports (via a
//! brute-force sphere scan along the wedge plane). Prints only.

use nf_game::player::Input;
use nf_view::scene::Sim;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_wedge_objects() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..120 { sim.step_bond(Input::default(), false, false, false); }

    // Every script object position (the SPs' streams + the created entities) within the duct box
    // x 38..44, y 1..5, z 55..67.
    eprintln!("=== script players: every one, then the near-duct filter ===");
    let mut near = 0;
    for (i, p) in sim.drones.scripts.players.iter().enumerate() {
        let Some(pos) = nf_game::drone::Drones::script_obj_pos(&sim.drones.scripts, &sim.data(), i) else { continue };
        if pos[0] > 38.0 && pos[0] < 44.0 && pos[2] > 55.0 && pos[2] < 67.0 {
            eprintln!("sp{i}: placement {} scripts {:?} modes {:?} state {} pos {pos:?}", p.placement, p.scripts, p.modes, p.state);
            near += 1;
        }
    }
    eprintln!("near-duct script players: {near}");

    // The scripts' owned objects: scan their live positions (Entity matrices / Anim roots).
    eprintln!("=== scripts.objects near the duct ===");
    let mut shown = 0;
    for (i, o) in sim.drones.scripts.objects.list.iter().enumerate() {
        let Some(o) = o else { continue };
        let pos = match o {
            nf_game::nis::Obj::Entity { matrix, .. } => Some([matrix[3][0], matrix[3][1], matrix[3][2]]),
            nf_game::nis::Obj::Anim { skin, script, frame, .. } => {
                nf_game::nis::World::discrete0(&sim.data(), *skin, *script, *frame).map(|m| [m[3][0], m[3][1], m[3][2]])
            }
            _ => None,
        };
        if let Some(p) = pos {
            if p[0] > 38.0 && p[0] < 44.0 && p[2] > 55.0 && p[2] < 67.0 {
                eprintln!("obj{i}: {o:?} at {p:?}");
                shown += 1;
            }
        }
    }
    eprintln!("near-duct script objects: {shown}");

    // Sphere scan: drop a 0.55 sphere from BELOW the roof reach (y 3.25) at several x lanes;
    // print the first contact y and the push direction — reveals bar-like shapes the line rays miss.
    eprintln!("=== sphere (r 0.55) drop from y 3.25, per (x, z) ===");
    for x in [40.2f32, 40.5, 40.8, 41.1, 41.3] {
        for z in [60.5f32, 60.0, 59.6, 59.45, 59.4, 59.2, 58.8] {
            let mut contact = None::<(f32, [f32; 3])>;
            let mut y = 3.25f32;
            while y > 1.5 {
                let (push, _) = sim.world.capsule_push([x, y + 1e-3, z], [x, y - 1e-3, z], 0.55);
                if push[0].abs() > 1e-5 || push[1].abs() > 1e-5 || push[2].abs() > 1e-5 { contact = Some((y, push)); break; }
                y -= 0.02;
            }
            match contact {
                Some((y, p)) => eprintln!("x {x:5.2} z {z:6.2}: contact y {y:.2} push ({:+.3},{:+.3},{:+.3})", p[0], p[1], p[2]),
                None => eprintln!("x {x:5.2} z {z:6.2}: no contact to the floor"),
            }
        }
    }
    // Capsule push scan: the crouch capsule (a=pos+0.10, b=pos−0.10, r 0.55) at x 41.0, y 3.08,
    // stepping z from 59.45 down to 58.9 — the first contact and its push direction.
    eprintln!("=== capsule push scan at x 41.0 y 3.08, z 59.45 → 58.85 ===");
    for z in [59.45f32, 59.42, 59.40, 59.38, 59.36, 59.34, 59.30, 59.25, 59.20, 59.10, 59.00, 58.90] {
        let a = [41.0f32, 3.18, z];
        let b = [41.0f32, 2.98, z];
        let (push, walkable) = sim.world.capsule_push(a, b, 0.55);
        eprintln!("z {z:6.3}: push ({:+.4},{:+.4},{:+.4}) walkable {walkable}", push[0], push[1], push[2]);
    }
    eprintln!("=== same scan at x 40.8 (duct centre) ===");
    for z in [59.45f32, 59.40, 59.35, 59.30, 59.20, 59.10, 59.00, 58.90] {
        let a = [40.8f32, 3.18, z];
        let b = [40.8f32, 2.98, z];
        let (push, walkable) = sim.world.capsule_push(a, b, 0.55);
        eprintln!("z {z:6.3}: push ({:+.4},{:+.4},{:+.4}) walkable {walkable}", push[0], push[1], push[2]);
    }
}
