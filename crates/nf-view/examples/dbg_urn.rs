//! Scratch: debug the urn SP's primed streams and script objects on 07000007.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000007.bin", false).expect("load");
    for t in 0..6 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    for (pi, p) in sim.drones.scripts.players.iter().enumerate() {
        if p.placement != 312 { continue; }
        println!("player {pi} placement 312 state {} cur {} mask {:04x} scripts {:08x}/{:08x} deleted {}",
            p.state, p.cur, p.mask, p.scripts[0], p.scripts[1], p.deleted);
        match &p.info[p.cur] {
            Some(s) => println!("  info: nis {:08x} delay {}", s.nis.hashcode, s.delay),
            None => println!("  info: None"),
        }
    }
    println!("--- script objects with src.0 == 0x0600007a:");
    for (i, o) in sim.drones.scripts.objects.list.iter().enumerate() {
        match o {
            Some(nf_game::nis::Obj::Entity { hash, src, matrix, .. }) if src.0 == 0x0600_007a =>
                println!("  obj {i}: hash {hash:08x} src {src:?} trans {:?}", [matrix[3][0], matrix[3][1], matrix[3][2]]),
            Some(nf_game::nis::Obj::Anim { skin, src, .. }) if src.0 == 0x0600_007a =>
                println!("  obj {i}: ANIM skin {skin:08x} src {src:?}"),
            _ => {}
        }
    }
    for h in [0x0200_04e7u32, 0x0200_04ee, 0x0200_04e5, 0x0200_04ea] {
        println!("--- glist {:08x} in models: {:?}", h, sim.models.get(&h).map(|m| (m.centre, m.radius)));
    }
    // the intact urn's world sphere
    let centre: [f32; 3];
    {
    let (_, m) = sim.drones.scripts.objects.list.iter().enumerate()
        .find_map(|(i, o)| match o { Some(nf_game::nis::Obj::Entity { hash: 0x0200_04ee, src: (0x0600_007a, 9), matrix, .. }) => Some((i, *matrix)), _ => None }).unwrap();
    let md = sim.models.get(&0x0200_04ee).unwrap();
    centre = [m[0][0]*md.centre[0] + m[1][0]*md.centre[1] + m[2][0]*md.centre[2] + m[3][0],
              m[0][1]*md.centre[0] + m[1][1]*md.centre[1] + m[2][1]*md.centre[2] + m[3][1],
              m[0][2]*md.centre[0] + m[1][2]*md.centre[1] + m[2][2]*md.centre[2] + m[3][2]];
    println!("urn world centre {:?} r {}", centre, md.radius);
    }
    let mut sim = scene.sim.as_mut().unwrap();
    {
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..4000 { if !sim.disabled && sim.gun.switch_state == 0 && sim.drones.scripts.camera().is_none() { break; } sim.step_bond(Default::default(), false, false, false); }
    println!("pre-fire: w {} sw {} sw_state {} clip {}", sim.gun.weapon, sim.gun.switch_request, sim.gun.switch_state, sim.gun.clip);
    sim.bond.pos = [centre[0], 43.62, centre[2] + 1.5];
    let eye = [centre[0], 44.32, centre[2] + 1.5];
    let d = [centre[0]-eye[0], centre[1]-eye[1], centre[2]-eye[2]];
    let l = (d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt();
    let dir = [d[0]/l, d[1]/l, d[2]/l];
    sim.bond.yaw = dir[0].atan2(dir[2]);
    sim.bond.pitch = dir[1].clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI;
    sim.gun.weapon = 2;
    sim.gun.switch_request = 2;
    sim.gun.reload_state = 0;
    sim.gun.fire_state = false;
    sim.gun.clip = 12;
    sim.gun.pending = 0;
    }
    // inject a P2K round through the intact urn's sphere
    scene.sim.as_mut().unwrap().drones.player_bullets.push(nf_game::drone::PlayerBullet {
        pos: [centre[0], centre[1] - 0.35, centre[2] + 1.5],
        dir: [0.0, 0.222, -0.975], speed: 13.2, travelled: 0.0, range: 50.0, damage: 2.75, weapon: 2,
        first: true, dead: false, bounces: 0, homing: false, target: None,
        gravity: false, guided_cam: false, snd: 0, swoosh: None, born: 0,
    });
    println!("pushed: {:?}", scene.sim.as_ref().unwrap().drones.player_bullets.iter().map(|b| (b.pos, b.dir)).collect::<Vec<_>>());
    for t in 0..40u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let p = &sim.drones.scripts.players[0];
        if t % 3 == 0 || t < 6 {
            let objs: Vec<String> = sim.drones.scripts.objects.list.iter().enumerate()
                .filter_map(|(i, o)| match o {
                    Some(nf_game::nis::Obj::Entity { hash, src, .. }) if src.0 == 0x0600_007a =>
                        Some(format!("#{i} {hash:08x}@s{}", src.1)),
                    _ => None }).collect();
            println!("t{t}: state {} delay {} objs {:?}", p.state, p.info[p.cur].as_ref().map(|s| s.delay).unwrap_or(0), objs);
        }
    }
}
