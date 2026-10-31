//! Probe: from the pack cinema's hand-back at the 07000016 pit floor, steer Bond (Input only)
//! into a blast room and log position/health through the ch-0xf3 pulse. Probe-only setup: the
//! bridge slab is reached by a position write (the walk test walks it). Opt-in via NF_ISO.
//! Usage: probe16_shelter <x> <z> [jump]
use nf_game::player::Input;

fn carry(on: &[u32]) -> nf_game::player::RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    nf_game::player::RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let tx: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(-16.0);
    let tz: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(-170.0);
    let jump = args.get(3).map_or(false, |s| s == "jump");
    let wps: Vec<[f32; 2]> = args.iter().skip(4).filter_map(|s| { let v: Vec<f32> = s.split(',').filter_map(|t| t.parse().ok()).collect(); (v.len() == 2).then(|| [v[0], v[1]]) }).collect();
    let iso = std::env::var("NF_ISO").unwrap();
    let mut scene = nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).expect("load");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
        sim.bond.pos = [2.4, 1.2, -175.5];
        if std::env::var_os("PROBE_8D").is_some() { let f = sim.drones.frame; sim.drones.channels.set(0x8d, 1, f); }
    }
    let mut reenabled = None;
    let mut pulse = None;
    let mut wp = 0usize;
    let mut over = false;
    let mut last = vec![false; 256];
    let mut was_dis = false;
    let mut mstat = 0u32;
    let max_t: u32 = std::env::var("PROBE_T").ok().and_then(|v| v.parse().ok()).unwrap_or(4400);
    for t in 0..max_t {
        let mut inp = Input::default();
        let (p, yaw, dis) = { let s = scene.sim.as_ref().unwrap(); (s.bond.pos, s.bond.yaw, s.disabled) };
        if reenabled.is_none() && p[1] < -20.0 && !dis { reenabled = Some(t); eprintln!("re-enable t{t} at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]); }
        let on243 = scene.sim.as_ref().unwrap().drones.channels.on(243);
        if pulse.is_none() && on243 { pulse = Some(t); eprintln!("PULSE rise t{t} pos ({:.2},{:.2},{:.2}) hp {}", p[0], p[1], p[2], scene.sim.as_ref().unwrap().drones.health.health); }
        if t % 300 == 0 && t > 2200 { for (i, d) in scene.sim.as_ref().unwrap().drones.list.iter().enumerate() { if d.present && (matches!(d.death_channel, 23..=26) || d.skin & 0xffff == 0x1e) { eprintln!("  t{t} drone[{i}] dc {} pos ({:.2},{:.2},{:.2}) hp {:.0} corpse {} en {}", d.death_channel, d.pos[0], d.pos[1], d.pos[2], d.health, d.corpse, d.enabled); } } }
        let delay: u32 = std::env::var("PROBE_DELAY").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        if reenabled.map_or(false, |r| t >= r + delay) {
            let target = if wp < wps.len() { wps[wp] } else { [tx, tz] };
            let dx = target[0] - p[0]; let dz = target[1] - p[2];
            if wp < wps.len() && dx.hypot(dz) < 0.8 { wp += 1; }
            if dx.hypot(dz) > 0.4 {
                let want = dx.atan2(dz);
                let mut d = want - yaw;
                while d > std::f32::consts::PI { d -= 2.0 * std::f32::consts::PI; }
                while d < -std::f32::consts::PI { d += 2.0 * std::f32::consts::PI; }
                inp.turn = d.clamp(-0.08, 0.08);
                inp.forward = if d.abs() < 0.6 { 1.0 } else { 0.2 };
                if jump && t % 40 == 0 { inp.jump = true; }
            }
        }
        if std::env::var_os("PROBE_GOD").is_some() { scene.sim.as_mut().unwrap().drones.health.health = 100.0; }
        scene.sim.as_mut().unwrap().step_bond(inp, false, false, false);
        let s = scene.sim.as_ref().unwrap();
        if t % 500 == 0 || s.drones.health.health <= 0.0 {
            eprintln!("t{t}: pos({:.2},{:.2},{:.2}) hp {:.1} g {} 243 {} 240 {} 80 {} dis {}",
                s.bond.pos[0], s.bond.pos[1], s.bond.pos[2], s.drones.health.health, s.bond.grounded,
                s.drones.channels.on(243) as u8, s.drones.channels.on(240) as u8, s.drones.channels.on(80) as u8, s.disabled as u8);
        }
        if s.drones.health.health <= 0.0 { eprintln!("DEAD t{t}"); break; }
        if pulse.is_some() && !on243 && !over { over = true; eprintln!("PULSE over t{t}, hp {}", s.drones.health.health); }
        for c in [0x17u32, 0x18, 0x19, 0x1a, 187, 186, 176, 193, 194, 195, 196, 189, 0xbf, 0xc0, 0x82, 0x8d, 0x90, 100, 240, 80, 190, 192, 204, 205, 243, 244, 245, 0x62, 203] {
            let on = s.drones.channels.on(c);
            if on != last[c as usize] { last[c as usize] = on; eprintln!("t{t} ch {c:#x} -> {}  bond ({:.2},{:.2},{:.2}) dis {}", on as u8, s.bond.pos[0], s.bond.pos[1], s.bond.pos[2], s.disabled as u8); }
        }
        if s.disabled != was_dis { was_dis = s.disabled; eprintln!("t{t} disabled -> {} bond ({:.2},{:.2},{:.2})", s.disabled as u8, s.bond.pos[0], s.bond.pos[1], s.bond.pos[2]); }
        if s.drones.mission.status() != mstat { mstat = s.drones.mission.status(); eprintln!("t{t} mission status {mstat}"); }
    }
}
