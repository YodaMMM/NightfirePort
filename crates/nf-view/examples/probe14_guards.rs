//! level-14-guards-at-locked-door probe v2: the REAL timing — guards spawn on 0x8C and arrive at
//! the locked Door_Slide_11A/B long before the E-use cascade. Watch the locked-door wait (states,
//! barks, positions), the unlock edge, and the crossing. Prints only.
use nf_game::player::Input;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    let (doors, clones) = {
        let sim = scene.sim.as_ref().unwrap();
        let doors: Vec<usize> = sim.drones.doors.list.iter().enumerate()
            .filter(|(_, d)| d.ch_lock == 0x85).map(|(i, _)| i).collect();
        let clones: Vec<usize> = sim.drones.list.iter().enumerate()
            .filter(|(_, x)| x.group == 9 || x.group == 13).map(|(i, _)| i).collect();
        (doors, clones)
    };
    println!("doors {doors:?} templates {clones:?}");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x87, 1, 1);
        sim.drones.channels.set(0x88, 1, 1);
        sim.bond.pos = [-87.5, -1.7, 33.4]; // the hall west end (the sabotage room)
    }
    // Phase 1: the locked-door wait. The real flow gives the guards ~1-2 min before the E press.
    let mut opendoor_ticks = 0u32;
    let mut barks: Vec<u32> = Vec::new();
    let mut prev_st: std::collections::HashMap<usize, u8> = std::collections::HashMap::new();
    for k in 0..2700u32 {
        scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        {
            let sim = scene.sim.as_ref().unwrap();
            for &c in &clones {
                let x = &sim.drones.list[c];
                let p = prev_st.insert(c, x.state);
                if p == Some(0x81) && x.state != 0x81 && x.state != 0x81 {
                    eprintln!("[EXIT k{k}] guard {c} 0x81 -> {:#04x} hit_door {:?} hit_door_open {} anim {}.{} call {:?} door_return {:#04x} alert {:.1}",
                        x.state, x.hit_door, x.hit_door_open, x.anim.class, x.anim.state, x.anim.call, x.door_return, x.alert);
                }
            }
        }
        if k >= 100 && k <= 260 {
            let sim = scene.sim.as_ref().unwrap();
            let sts: Vec<String> = clones.iter().map(|&c| {
                let x = &sim.drones.list[c];
                format!("#{c}:{}a{:.1}", x.state, x.alert)
            }).collect();
            eprintln!("@k{k} {}", sts.join(" "));
        }
        let sim = scene.sim.as_ref().unwrap();
        for &c in &clones {
            if sim.drones.list[c].state == 0x81 { opendoor_ticks += 1; }
        }
        for id in sim.drones.audio.active_ids() {
            if id >= 0x500 && !barks.contains(&id) { barks.push(id); }
        }
        if k % 300 == 0 { report(&sim, k, &doors, &clones, opendoor_ticks, &barks); }
    }
    report(scene.sim.as_ref().unwrap(), 2700, &doors, &clones, opendoor_ticks, &barks);

    // Phase 2: the REAL E press at the switch plate.
    let t_centre = {
        let sim = scene.sim.as_ref().unwrap();
        let i = sim.drones.triggers.iter()
            .position(|t| t.out == 0xa0 && t.mask & 4 != 0 && t.kind == nf_game::triggers::kind::TOUCH)
            .expect("the 0xA0 E-use touch exists");
        let t = &sim.drones.triggers[i];
        [(t.lo[0] + t.hi[0]) * 0.5, (t.lo[1] + t.hi[1]) * 0.5, (t.lo[2] + t.hi[2]) * 0.5]
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [-60.3, 0.9, 38.4];
        let head = sim.bond.head();
        let d = [t_centre[0] - head[0], t_centre[1] - head[1], t_centre[2] - head[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
        sim.bond.yaw = d[0].atan2(d[2]);
        sim.bond.pitch = (d[1] / l).asin() / std::f32::consts::FRAC_PI_2;
    }
    let mut fired = false;
    for _ in 0..12 {
        scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, true);
        if scene.sim.as_ref().unwrap().drones.channels.on(0xa0) { fired = true; break; }
    }
    println!("E fired: {fired}");
    // Phase 3: the cascade → 0x85 → the door opens; watch the crossing.
    let mut ch85 = false;
    let mut crossed_at: Vec<Option<u32>> = clones.iter().map(|_| None).collect();
    for k in 0..1500u32 {
        scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x85) { ch85 = true; }
        if (340..=440).contains(&k) {
            for g in &sim.drones.grenades {
                eprintln!("[G k{}] grenade at ({:.2},{:.2},{:.2}) vel ({:.2},{:.2},{:.2}) speed {:.2} state {} travelled {:.1}", k, g.pos[0], g.pos[1], g.pos[2], g.velocity[0], g.velocity[1], g.velocity[2], g.speed, g.state, g.travelled);
            }
            if k == 370 || k == 375 || k == 424 {
                let st: Vec<String> = clones.iter().map(|&c| {
                    let x = &sim.drones.list[c];
                    if x.present { format!("#{c} pos ({:.2},{:.2},{:.2}) opp ({:.2},{:.2},{:.2}) st{} gw {:#x} n{}", x.pos[0], x.pos[1], x.pos[2], x.opp_target[0], x.opp_target[1], x.opp_target[2], x.state, x.grenade_weapon, x.grenade_count) } else { format!("#{c} gone") }
                }).collect();
                eprintln!("[T k{}] {}", k, st.join(" | "));
            }
        }
        if k < 600 || sim.drones.channels.on(0x51) || sim.drones.channels.on(0x85) {
            if !sim.drones.explosions.is_empty() {
                for (pos, radius, damage, attacker, wt) in &sim.drones.explosions {
                    eprintln!("[E k{k}] blast at ({:.1},{:.1},{:.1}) r{radius:.1} dmg{damage:.0} att{attacker} wt{wt:#x}", pos[0], pos[1], pos[2]);
                }
            }
            if k % 10 == 0 {
                let hs: Vec<String> = clones.iter().map(|&c| {
                    let x = &sim.drones.list[c];
                    format!("#{c} h{:.0} st{}", x.health, x.state)
                }).collect();
                eprintln!("[H k{k}] 51{} 52{} 5b{} 85{} 12{} 13{} | {}", sim.drones.channels.on(0x51) as u8, sim.drones.channels.on(0x52) as u8, sim.drones.channels.on(0x5b) as u8, sim.drones.channels.on(0x85) as u8, sim.drones.channels.on(0x12) as u8, sim.drones.channels.on(0x13) as u8, hs.join(" "));
            }
        }
        for (j, &c) in clones.iter().enumerate() {
            if crossed_at[j].is_none() && sim.drones.list[c].present && sim.drones.list[c].pos[0] < -52.4 {
                crossed_at[j] = Some(k);
            }
        }
        for &c in &clones {
            if sim.drones.list[c].state == 0x81 { opendoor_ticks += 1; }
        }
        if k % 300 == 0 { report(&sim, 3000 + k, &doors, &clones, opendoor_ticks, &barks); }
    }
    let sim = scene.sim.as_ref().unwrap();
    report(&sim, 4500, &doors, &clones, opendoor_ticks, &barks);
    println!("0x85 rose: {ch85}");
    println!("crossed x<-52.4 (tick after E): {crossed_at:?}");
    for &d in &doors {
        let dr = &sim.drones.doors.list[d];
        println!("door {d}: state {:?} pos01 {:.3} ch_lock {:#x}", dr.state, dr.pos01, dr.ch_lock);
    }
    println!("bark sound ids seen: {barks:?}");
}

fn report(sim: &nf_view::scene::Sim, k: u32, doors: &[usize], clones: &[usize], od: u32, barks: &[u32]) {
    let mut parts: Vec<String> = Vec::new();
    for &c in clones {
        let x = &sim.drones.list[c];
        if x.present {
            parts.push(format!("#{c} ({:.1},{:.1},{:.1}) st{} al{:.0}", x.pos[0], x.pos[1], x.pos[2], x.state, x.alert));
        } else {
            parts.push(format!("#{c} held"));
        }
    }
    let mut dparts: Vec<String> = Vec::new();
    for &d in doors {
        let dr = &sim.drones.doors.list[d];
        dparts.push(format!("door{d} st{:?} p{:.2}", dr.state, dr.pos01));
    }
    let ch = &sim.drones.channels;
    eprintln!("[k{k}] 8c{} 51{} 85{} | {} | {} | od{od} barks{barks:?}",
        ch.on(0x8c) as u8, ch.on(0x51) as u8, ch.on(0x85) as u8, parts.join(" | "), dparts.join(" "));
}
