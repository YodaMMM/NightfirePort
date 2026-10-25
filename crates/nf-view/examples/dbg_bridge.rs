//! Scratch: debug the sky-bridge shot on 07000012.
use nf_game::player::Input;

fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000012.bin", false).expect("load");
    for _ in 0..90 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let pi = scene.sim.as_ref().unwrap().drones.scripts.players.iter()
        .position(|p| p.placement == 660).expect("bridge SP");
    {
        let sim = scene.sim.as_ref().unwrap();
        let p = &sim.drones.scripts.players[pi];
        println!("bridge SP: state {} hp {} mask {:04x} disabled {} | gun w {} sw_req {} sw_state {} clip {}",
            p.state, p.hit_points, p.mask, sim.disabled, sim.gun.weapon, sim.gun.switch_request, sim.gun.switch_state, sim.gun.clip);
        // every script object with src.0 == 0x06000530
        for (i, o) in sim.drones.scripts.objects.list.iter().enumerate() {
            if let Some(nf_game::nis::Obj::Entity { hash, src, matrix, .. }) = o {
                if src.0 == 0x0600_0530 {
                    println!("  obj {i}: hash {hash:08x} stream {} trans {:?}", src.1,
                        [matrix[3][0], matrix[3][1], matrix[3][2]]);
                }
            }
        }
    }
    let centre = [0.015154322f32, -0.41502616, -0.045462962];
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..900 { if !sim.disabled { break; } sim.step_bond(Default::default(), false, false, false); }
    println!("after disabled-wait: t~{} disabled {}", "n/a", sim.disabled);
    sim.bond.pos = [centre[0], centre[1], centre[2] + 1.2];
    let eye = [centre[0], centre[1] + 0.70, centre[2] + 1.2];
    let d = [centre[0] - eye[0], centre[1] - eye[1], centre[2] - eye[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let dir = [d[0] / l, d[1] / l, d[2] / l];
    sim.bond.yaw = dir[0].atan2(dir[2]);
    sim.bond.pitch = dir[1].clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI;
    println!("pre-fire: w {} sw {} clip {} spare {} latch {} timer {} view {} disabled {}",
        sim.gun.weapon, sim.gun.switch_request, sim.gun.clip, sim.gun.spare, sim.gun.latch as u8, sim.gun.timer, sim.view_script, sim.disabled);
    for t in 0..30u32 {
        let mut input = Input::default();
        input.fire = t < 3;
        scene.sim.as_mut().unwrap().step_bond(input, t < 3, false, false);
        let sim = scene.sim.as_ref().unwrap();
        println!("t{t}: bullets {} clip {} view {:08x} vt {:.1} state {}", sim.drones.player_bullets.len(), sim.gun.clip, sim.view_script, sim.view_time, sim.drones.scripts.players[pi].state);
    }
    for t in 30..42u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let p = &sim.drones.scripts.players[pi];
        println!("t{t}: bullets {} tick_hits {:?} hits {:?} state {} bond {:?}",
            sim.drones.player_bullets.len(), p.tick_bullet_hits, p.bullet_hits, p.state, sim.bond.pos);
    }
}
