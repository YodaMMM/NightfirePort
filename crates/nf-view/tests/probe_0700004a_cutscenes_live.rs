//! level-4a-cutscene probe: 0700004a fresh load (no --bond, no pos writes), idle input — logs
//! every scripted cut-scene the level plays: which scripts run (the SP start + its pack hashes
//! [D the level's SPs #42/#273/#276/#289]), the script camera per tick (position, fov — the
//! reported symptom pair: a near-zero fov key draws a flat void; a camera inside geometry draws
//! nothing), the hand-back (camera off, the disable transitions), and the actors on screen.
//! Probe (assertion-free except the load) — the pins land in the fix tests. Opt-in via NF_ISO.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn probe_0700004a_cutscenes_fresh_load() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "0700004a.bin", false).expect("load 0700004a");
    let mut cam_was = false;
    let mut dis_was = false;
    let mut started: Vec<u32> = Vec::new();
    for t in 0..7000u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        // script starts: any player entering state 2 with a new pack hash
        for p in &sim.drones.scripts.players {
            if p.state == 2 {
                for h in p.scripts.iter() {
                    if *h != 0 && !started.contains(h) {
                        started.push(*h);
                        eprintln!("[4a-nis] t{t} SP start hash {h:08x} (placement {}, mode {:?}, out {:04x})",
                            p.placement, p.modes, p.out);
                    }
                }
            }
        }        // the camera per tick
        let cam = sim.drones.scripts.camera();
        match cam {
            Some((m, fov)) => {
                if !cam_was { eprintln!("[4a-cam] t{t} camera ON"); cam_was = true; }
                if t % 10 == 0 || fov < 0.1 {
                    eprintln!("[4a-cam] t{t} pos ({:.4},{:.4},{:.4}) fov {:.6} yaw {:.4} pitch {:.4}",
                        m[3][0], m[3][1], m[3][2], fov,
                        m[2][0].atan2(m[2][2]), (m[2][1] / (m[2][0]*m[2][0] + m[2][1]*m[2][1] + m[2][2]*m[2][2]).sqrt().max(1e-6)).clamp(-1.0, 1.0).asin());
                }
            }
            None => {
                if cam_was { eprintln!("[4a-cam] t{t} camera OFF (hand-back)"); cam_was = false; }
            }
        }
        // the disable transitions (the cut-scene's PlayerDisable / hand-back)
        let dis = sim.disabled;
        if dis != dis_was {
            eprintln!("[4a-dis] t{t} disabled -> {dis}");
            dis_was = dis;
        }
        // actors: present + drawn counts every 100 ticks while a camera runs
        if t % 100 == 0 && cam.is_some() {
            let n = sim.drones.list.iter().filter(|d| d.present).count();
            let drawn = sim.drones.list.iter().filter(|d| d.present && d.drawn).count();
            let y = sim.bond.pos[1];
            eprintln!("[4a-act] t{t} drones present {n} drawn {drawn} bond y {y:.2}");
        }
        if sim.drones.level_exit.is_some() { eprintln!("[4a-nis] t{t} level exit"); break; }
    }
}
