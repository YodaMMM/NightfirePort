//! 07000014's released door guards must not go to combat unseen: after the fusebox release
//! (ch 0x87 + ch 0x88 -> 0x8C [D AND plc #717]) with Bond held out of sight in the generator
//! hall (-76.8, -1.1, 46.5), the game [L PCSX2 fresh boot, 3000 frames] walks two guards in
//! ENEMY_MISSION (0x1b, sub 0x06) to the west door and stops them at (-51.6, 1.0, 41.2) /
//! (-51.2, 1.0, 38.8), staying 0x1b throughout; their +0x5A6 saved state stays 0; no
//! OPEN_DOOR (0x81) ever. This build is the diagnostic run (full telemetry, pins pending).

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_released_guards_hold_enemy_mission_at_the_west_door_diagnostic() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    // The spawners start on 0x8C [D spawners #720/#1333 p_start 0x8C]: seed the sabotage AND's
    // two words through the REAL AND mux [D AND plc (0x87,0x88)→0x8C].
    scene.sim.as_mut().unwrap().drones.channels.set(0x87, 1, 1);
    scene.sim.as_mut().unwrap().drones.channels.set(0x88, 1, 1);
    scene.sim.as_mut().unwrap().bond.pos = [-76.8, -1.1, 46.5]; // the generator hall, out of sight
    let mut prev: std::collections::HashMap<usize, (u8, f32)> = std::collections::HashMap::new();
    for k in 0..3000u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.health.health = 100.0;
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let clones: Vec<usize> = (0..sim.drones.list.len())
            .filter(|&i| sim.drones.list[i].present && (sim.drones.list[i].group == 9 || sim.drones.list[i].group == 13))
            .collect();
        for &c in &clones {
            let g = &sim.drones.list[c];
            let cur = (g.state, g.alert);
            let changed = match prev.get(&c) { Some(p) => *p != cur, None => true };
            if changed && k > 0 {
                eprintln!("[diag] t{k}: guard {c} state {:#x}-> {:#x} alert {:.3} (los {} seen {} ever {} share {:.3} bits {:#x} pend {:#x}) pos ({:.1},{:.1},{:.1})",
                    prev.get(&c).map(|p| p.0).unwrap_or(0), g.state, g.alert, g.los, g.seen, g.ever_seen, g.sound_share, g.alert_bits, g.alert_pending, g.pos[0], g.pos[1], g.pos[2]);
            }
            prev.insert(c, cur);
        }
        if k % 250 == 0 {
            let sts: Vec<String> = clones.iter().map(|&c| {
                let x = &sim.drones.list[c];
                format!("#{c} st{:#x} a{:.2} ({:.1},{:.1},{:.1})", x.state, x.alert, x.pos[0], x.pos[1], x.pos[2])
            }).collect();
            let ch96 = sim.drones.channels.on(0x96);
            let ch8c = sim.drones.channels.on(0x8c);
            eprintln!("[diag] t{k}: 0x8C {ch8c} 0x96 {ch96} | {}", sts.join(" | "));
        }
    }
}
