//! Probe: every placement of every 0700001b map with its kind, position and params (hex), one
//! line each — the channel map for the mission's writer hunt. Assertion-free; run with
//! --ignored --nocapture. [D via nf-formats::level.]

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_0700001b_every_placement() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let data = read_archive(&iso, "0700001b.bin");
    let lv = Level::parse(&data).expect("parse");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (i, p) in m.placements.iter().enumerate() {
            let mut ps = String::new();
            for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
            println!("map{mi} #{i} kind={:#04x} flags={:#x} hash={:#x} pos=({:.2},{:.2},{:.2}){ps}",
                p.flags & 0xff, p.flags, p.hashcode, p.translation[0], p.translation[1], p.translation[2]);
        }
    }
}

#[path = "common/walker.rs"]
#[allow(dead_code)]
mod walker;

/// The natural flow: load, no substate force, Bond idle in the start's float; log every channel
/// value change per tick for N ticks (NF_1B_TICKS, default 12000), plus Bond's substate/pos.
#[test]
#[ignore = "assertion-free trace; run with --ignored"]
fn trace_0700001b_idle_channels() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let n: u32 = std::env::var("NF_1B_TICKS").ok().and_then(|s| s.parse().ok()).unwrap_or(12000);
    let mut w = walker::Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    let mut prev = w.scene.sim.as_ref().unwrap().drones.channels.v.clone();
    for t in 0..n {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let v = &sim.drones.channels.v;
        for c in 0..256 {
            if v[c] != prev[c] {
                println!("t{t} f{} ch {c} ({c:#x}) {} -> {}  bond ss {} pos ({:.2},{:.2},{:.2}) dis {} mstat {}",
                    sim.drones.frame, prev[c], v[c], sim.bond.substate, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2],
                    sim.disabled, sim.drones.mission.status());
            }
        }
        prev = v.clone();
        if t % 1000 == 0 {
            println!("t{t} bond ss {} pos ({:.2},{:.2},{:.2}) disabled {} missiles {:?}", sim.bond.substate,
                sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.disabled,
                sim.drones.space_missiles.iter().map(|m| m.state).collect::<Vec<_>>());
            for (i, d) in sim.drones.list.iter().enumerate() {
                if !d.present { continue; }
                println!("   drone {i} skin {:#x} pos ({:.2},{:.2},{:.2}) en {} corpse {} state {:#x} hp {:.1}",
                    d.skin, d.pos[0], d.pos[1], d.pos[2], d.enabled, d.corpse, d.state, d.health);
            }
        }
    }
}

fn log_changes(w: &walker::Walker, prev: &mut Vec<u8>, t: u32) {
    let sim = w.scene.sim.as_ref().unwrap();
    let v = &sim.drones.channels.v;
    for c in 0..256 {
        if v[c] != prev[c] {
            println!("t{t} f{} ch {c} ({c:#x}) {} -> {}  bond ss {} pos ({:.2},{:.2},{:.2}) mstat {} hp {:.0}",
                sim.drones.frame, prev[c], v[c], sim.bond.substate, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2],
                sim.drones.mission.status(), sim.drones.health.health);
        }
    }
    *prev = v.clone();
}

/// The natural flow with the real fire: wait out the arrival, kill every spawned astronaut,
/// then idle and log.
#[test]
#[ignore = "assertion-free trace; run with --ignored"]
fn trace_0700001b_kill_wave() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = walker::Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    let mut prev = w.scene.sim.as_ref().unwrap().drones.channels.v.clone();
    let mut t = 0u32;
    while t < 1600 { w.tick(Default::default(), false); log_changes(&w, &mut prev, t); t += 1; }
    for round in 0..20 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().position(|d| d.present && !d.corpse && d.enabled && d.health > 0.0 && d.skin & 0xffff == 0x78)
        };
        let Some(di) = target else { println!("round {round}: no live astronaut"); break };
        let ok = w.shoot(di, "astronaut");
        log_changes(&w, &mut prev, t);
        let sim = w.scene.sim.as_ref().unwrap();
        println!("round {round}: shoot d{di} -> {ok}; bond ss {} pos {:?} hp {:.0} frame {}", sim.bond.substate, sim.bond.pos, sim.drones.health.health, sim.drones.frame);
    }
    let n: u32 = std::env::var("NF_1B_TICKS").ok().and_then(|s| s.parse().ok()).unwrap_or(6000);
    for _ in 0..n {
        w.tick(Default::default(), false); log_changes(&w, &mut prev, t); t += 1;
        if t % 1000 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            println!("t{t} missiles {:?} deploy {:?}", sim.drones.space_missiles.iter().map(|m| m.state).collect::<Vec<_>>(), sim.drones.missile_deploy);
        }
    }
}

#[test]
#[ignore = "assertion-free trace; run with --ignored"]
fn zerog_axis_signs() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = walker::Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "arrival");
    w.tick(Default::default(), false);
    let r0 = w.scene.sim.as_ref().unwrap().bond.zero_g_rot;
    println!("rot0 {:?} active {}", r0, w.scene.sim.as_ref().unwrap().bond.zero_g_active);
    w.tick(nf_game::player::Input { turn: 0.1, ..Default::default() }, false);
    let r1 = w.scene.sim.as_ref().unwrap().bond.zero_g_rot;
    println!("after turn +0.1: fwd {:?} (was {:?}), right {:?}", r1[2], r0[2], r0[0]);
    w.tick(nf_game::player::Input { look: 0.1, ..Default::default() }, false);
    let r2 = w.scene.sim.as_ref().unwrap().bond.zero_g_rot;
    println!("after look +0.1: fwd {:?} (was {:?}), up {:?}", r2[2], r1[2], r1[1]);
    let sim = w.scene.sim.as_ref().unwrap();
    println!("look() {:?} eye {} crouch {}", sim.bond.look(), sim.bond.eye, sim.bond.crouch_drop);
}

fn data_of(sim: &nf_view::scene::Sim) -> nf_game::drone::Data<'_> {
    nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
        ai_points: &sim.ai_points,
    }
}

/// Thrust toward `at` on the body axes (a P controller on the thruster targets — no turning).
fn thrust_to(sim: &nf_view::scene::Sim, at: [f32; 3], k: f32) -> nf_game::player::Input {
    let p = sim.bond.pos;
    let d = [at[0] - p[0], at[1] - p[1], at[2] - p[2]];
    let m = sim.bond.zero_g_rot;
    let dot = |a: [f32; 3]| a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
    nf_game::player::Input {
        strafe: (dot(m[0]) * k).clamp(-1.0, 1.0),
        lift: (dot(m[1]) * k).clamp(-1.0, 1.0),
        forward: (dot(m[2]) * k).clamp(-1.0, 1.0),
        ..Default::default()
    }
}

#[test]
#[ignore = "assertion-free trace; run with --ignored"]
fn trace_0700001b_consoles() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = walker::Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    let mut prev = w.scene.sim.as_ref().unwrap().drones.channels.v.clone();
    let mut t = 0u32;
    while t < 1600 { w.tick(Default::default(), false); log_changes(&w, &mut prev, t); t += 1; }
    while !w.scene.sim.as_ref().unwrap().drones.channels.on(19) && t < 6000 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().position(|d| d.present && !d.corpse && d.enabled && d.health > 0.0 && d.skin & 0xffff == 0x78)
        };
        if let Some(di) = target { let _ = w.shoot(di, "astronaut"); } else { w.tick(Default::default(), false); t += 1; }
        log_changes(&w, &mut prev, t);
    }
    println!("ch19 at frame {}", w.scene.sim.as_ref().unwrap().drones.frame);
    for k in [3u32, 4, 5, 6, 8, 1, 2, 7] {
        let gate = 30 + k;
        let pi = w.scene.sim.as_ref().unwrap().drones.scripts.players.iter().position(|p| p.gate as u32 == gate && p.hit_points > 0.0).unwrap();
        let mut done = false;
        for tt in 0..3000u32 {
            let (input, fire) = {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.drones.channels.on(k) { done = true; break; }
                let tg = sim.drones.scripts.held_hit_targets(&data_of(sim));
                let tg: Vec<_> = tg.into_iter().filter(|x| x.0 == pi).collect();
                let m = sim.drones.scripts.players[pi].live_matrix;
                let centre = tg.first().map(|x| x.1).unwrap_or([m[3][0], m[3][1] + 5.0, m[3][2]]);
                if tt % 200 == 0 {
                    println!("   errs {:?} look {:?}", walker::zero_g_errs(sim, centre), sim.bond.look());
                    println!("console {k} sp {pi} tt {tt} held targets {:?} sp origin ({:.2},{:.2},{:.2}) bond ({:.2},{:.2},{:.2}) frame {}",
                        tg, m[3][0], m[3][1], m[3][2], sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.drones.frame);
                }
                // Stand 4 u off the screen toward the shaft's mid plane (z 0).
                let stand = [centre[0], centre[1], centre[2] - centre[2].signum() * 4.0];
                let mut inp = thrust_to(sim, stand, 0.15);
                let (e, pe) = walker::zero_g_errs(sim, centre).unwrap_or((0.0, 0.0));
                inp.turn = e.clamp(-0.08, 0.08);
                inp.look = pe.clamp(-0.08, 0.08);
                let on = !tg.is_empty() && e.abs() < 0.03 && pe.abs() < 0.02 && tt % 16 == 0;
                (inp, on)
            };
            if fire { w.tick_fire(input); } else { w.tick(input, false); }
            t += 1;
            log_changes(&w, &mut prev, t);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        println!("console {k}: done {done} frame {} bond {:?} clip {} spare {}", sim.drones.frame, sim.bond.pos, sim.gun.clip, sim.gun.spare);
    }
    for _ in 0..6000 { w.tick(Default::default(), false); t += 1; log_changes(&w, &mut prev, t); }
}

/// Console phase v2: hold a float point and shoot the held screens in deadline order; when the
/// bullet line is blocked, drift to the screen's stand point.
#[test]
#[ignore = "assertion-free trace; run with --ignored"]
fn trace_0700001b_consoles2() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let order: Vec<u32> = std::env::var("NF_1B_ORDER").ok().map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect())
        .unwrap_or(vec![3, 4, 5, 6, 8, 7, 2, 1]);
    let hold: [f32; 3] = std::env::var("NF_1B_HOLD").ok().map(|s| { let v: Vec<f32> = s.split(',').filter_map(|x| x.parse().ok()).collect(); [v[0], v[1], v[2]] })
        .unwrap_or([30.0, -3.0, 0.0]);
    let mut w = walker::Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    let mut prev = w.scene.sim.as_ref().unwrap().drones.channels.v.clone();
    let mut t = 0u32;
    while t < 1600 { w.tick(Default::default(), false); log_changes(&w, &mut prev, t); t += 1; }
    while !w.scene.sim.as_ref().unwrap().drones.channels.on(19) && t < 6000 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().position(|d| d.present && !d.corpse && d.enabled && d.health > 0.0 && d.skin & 0xffff == 0x78)
        };
        if let Some(di) = target { let _ = w.shoot(di, "astronaut"); } else {
            let input = thrust_to(w.scene.sim.as_ref().unwrap(), hold, 0.15);
            w.tick(input, false); t += 1;
        }
        log_changes(&w, &mut prev, t);
    }
    println!("ch19 at frame {} clip {}", w.scene.sim.as_ref().unwrap().drones.frame, w.scene.sim.as_ref().unwrap().gun.clip);
    for k in order {
        let gate = 30 + k;
        let pi = w.scene.sim.as_ref().unwrap().drones.scripts.players.iter().position(|p| p.gate as u32 == gate && p.hit_points > 0.0).unwrap();
        let mut done = false;
        let mut shots = 0u32;
        let mut moving = false;
        let f0 = w.scene.sim.as_ref().unwrap().drones.frame;
        let c0 = w.scene.sim.as_ref().unwrap().gun.clip;
        for tt in 0..3000u32 {
            let (input, fire) = {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.drones.channels.on(k) { done = true; break; }
                let tg = sim.drones.scripts.held_hit_targets(&data_of(sim));
                let tg: Vec<_> = tg.into_iter().filter(|x| x.0 == pi).collect();
                let m = sim.drones.scripts.players[pi].live_matrix;
                let centre = tg.iter().max_by(|a, b| a.2.partial_cmp(&b.2).unwrap()).map(|x| x.1).unwrap_or([m[3][0], m[3][1] + 5.0, m[3][2]]);
                let eye = sim.bond.head();
                let clear = sim.world.segment_bullet(eye, centre).is_none();
                if !clear { moving = true; }
                let stand = [centre[0], centre[1], centre[2] - centre[2].signum() * 4.0];
                let mut inp = if moving { thrust_to(sim, stand, 0.15) } else { thrust_to(sim, sim.bond.pos, 0.0) };
                let (e, pe) = walker::zero_g_errs(sim, centre).unwrap_or((0.0, 0.0));
                inp.turn = e.clamp(-0.08, 0.08);
                inp.look = pe.clamp(-0.08, 0.08);
                let on = !tg.is_empty() && clear && e.abs() < 0.01 && pe.abs() < 0.008 && tt % 12 == 0;
                (inp, on)
            };
            if fire { shots += 1; w.tick_fire(input); } else { w.tick(input, false); }
            t += 1;
            log_changes(&w, &mut prev, t);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        println!("console {k}: done {done} frame {} (took {}) moved {moving} taps {shots} rounds {} bond ({:.1},{:.1},{:.1}) clip {}",
            sim.drones.frame, sim.drones.frame - f0, c0 as i64 - sim.gun.clip as i64, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.gun.clip);
    }
    for _ in 0..3000 { w.tick(Default::default(), false); t += 1; log_changes(&w, &mut prev, t); }
    let sim = w.scene.sim.as_ref().unwrap();
    println!("end: missiles {:?} deploy {:?} mstat {}", sim.drones.space_missiles.iter().map(|m| m.state).collect::<Vec<_>>(), sim.drones.missile_deploy, sim.drones.mission.status());
}

/// Console phase v3: the mid-plane stations — each missile pair shares an x; Bond floats to
/// (x, y_screen, 0) and shoots both screens from there, pairs in deadline order.
#[test]
#[ignore = "assertion-free trace; run with --ignored"]
fn trace_0700001b_consoles3() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let order: Vec<u32> = std::env::var("NF_1B_ORDER").ok().map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect())
        .unwrap_or(vec![3, 4, 5, 6, 8, 7, 2, 1]);
    let tol: f32 = std::env::var("NF_1B_TOL").ok().and_then(|s| s.parse().ok()).unwrap_or(0.015);
    let zoff: f32 = std::env::var("NF_1B_Z").ok().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let range: f32 = std::env::var("NF_1B_R").ok().and_then(|s| s.parse().ok()).unwrap_or(1.5);
    let mut w = walker::Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    let mut prev = w.scene.sim.as_ref().unwrap().drones.channels.v.clone();
    let mut t = 0u32;
    while t < 1600 { w.tick(Default::default(), false); log_changes(&w, &mut prev, t); t += 1; }
    while !w.scene.sim.as_ref().unwrap().drones.channels.on(19) && t < 6000 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().position(|d| d.present && !d.corpse && d.enabled && d.health > 0.0 && d.skin & 0xffff == 0x78)
        };
        if let Some(di) = target { let _ = w.shoot(di, "astronaut"); } else { w.tick(Default::default(), false); t += 1; }
        log_changes(&w, &mut prev, t);
    }
    println!("ch19 at frame {} clip {}", w.scene.sim.as_ref().unwrap().drones.frame, w.scene.sim.as_ref().unwrap().gun.clip);
    for k in order {
        let gate = 30 + k;
        let pi = w.scene.sim.as_ref().unwrap().drones.scripts.players.iter().position(|p| p.gate as u32 == gate && p.hit_points > 0.0).unwrap();
        let mut done = false;
        let mut shots = 0u32;
        let f0 = w.scene.sim.as_ref().unwrap().drones.frame;
        let c0 = w.scene.sim.as_ref().unwrap().gun.clip;
        for tt in 0..3000u32 {
            let (input, fire) = {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.drones.channels.on(k) { done = true; break; }
                let tg = sim.drones.scripts.held_hit_targets(&data_of(sim));
                let tg: Vec<_> = tg.into_iter().filter(|x| x.0 == pi).collect();
                let m = sim.drones.scripts.players[pi].live_matrix;
                let centre = tg.iter().max_by(|a, b| a.2.partial_cmp(&b.2).unwrap()).map(|x| x.1)
                    .unwrap_or([m[3][0], m[3][1] + 5.33, m[3][2] - m[3][2].signum() * 1.75]);
                let stand = [centre[0], centre[1], centre[2].signum() * zoff];
                let near = walker::dxz(sim.bond.pos, stand) < range;
                let mut inp = thrust_to(sim, stand, 0.15);
                let (e, pe) = walker::zero_g_errs(sim, centre).unwrap_or((0.0, 0.0));
                inp.turn = e.clamp(-0.08, 0.08);
                inp.look = pe.clamp(-0.08, 0.08);
                let on = !tg.is_empty() && near && e.abs() < tol && pe.abs() < tol && tt % 12 == 0;
                (inp, on)
            };
            if fire { shots += 1; w.tick_fire(input); } else { w.tick(input, false); }
            t += 1;
            log_changes(&w, &mut prev, t);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        println!("console {k}: done {done} frame {} (took {}) taps {shots} rounds {} bond ({:.1},{:.1},{:.1}) clip {}",
            sim.drones.frame, sim.drones.frame - f0, c0 as i64 - sim.gun.clip as i64, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.gun.clip);
    }
    for _ in 0..3000 { w.tick(Default::default(), false); t += 1; log_changes(&w, &mut prev, t); }
    let sim = w.scene.sim.as_ref().unwrap();
    println!("end: missiles {:?} deploy {:?} mstat {}", sim.drones.space_missiles.iter().map(|m| m.state).collect::<Vec<_>>(), sim.drones.missile_deploy, sim.drones.mission.status());
}

#[test]
#[ignore = "assertion-free trace; run with --ignored"]
fn gun_1b() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = walker::Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "arrival");
    w.wait(200);
    let sim = w.scene.sim.as_ref().unwrap();
    println!("weapon {:#x} clip {} spare {}", sim.gun.weapon, sim.gun.clip, sim.gun.spare);
    let c0 = sim.gun.clip;
    for t in 0..120u32 {
        let fire = t == 10;
        if fire { w.tick_fire(Default::default()); } else { w.tick(Default::default(), false); }
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.gun.clip != c0 { println!("t{t} clip {}", sim.gun.clip); }
    }
    for t in 0..600u32 {
        w.tick(Default::default(), false);
        if t % 100 == 0 { println!("recharge t{t} clip {}", w.scene.sim.as_ref().unwrap().gun.clip); }
    }
}

/// LOS scan at the consoles' hold: which float points see which screens (bullet segment clear
/// up to the sphere's surface).
#[test]
#[ignore = "assertion-free trace; run with --ignored"]
fn los_1b_screens() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = walker::Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    // drive the natural chain to the hold (the probe's only use of a channel write: LOS geometry only)
    w.wait_enabled(3000, "arrival");
    let f = w.scene.sim.as_ref().unwrap().drones.frame;
    w.scene.sim.as_mut().unwrap().drones.channels.set(19, 1, f);
    w.wait(1100);
    let sim = w.scene.sim.as_ref().unwrap();
    let tg = sim.drones.scripts.held_hit_targets(&data_of(sim));
    let mut screens: Vec<(u32, [f32; 3], f32)> = Vec::new();
    for k in 1..=8u32 {
        let Some(pi) = sim.drones.scripts.players.iter().position(|p| p.gate as u32 == 30 + k && p.hit_points > 0.0) else { continue };
        let mut best: Option<([f32; 3], f32)> = None;
        for x in tg.iter().filter(|x| x.0 == pi) { if best.map_or(true, |b| x.2 > b.1) { best = Some((x.1, x.2)); } }
        if let Some((c, r)) = best { screens.push((k, c, r)); }
    }
    println!("screens {:?}", screens);
    let mut rows = Vec::new();
    let mut x = 10.0f32;
    while x <= 54.0 {
        let mut y = -9.0f32;
        while y <= 3.0 {
            let mut z = -6.0f32;
            while z <= 6.01 {
                let e = [x, y + 0.7, z];
                let mut seen = Vec::new();
                for (k, c, r) in &screens {
                    let d = ((c[0]-e[0]).powi(2) + (c[1]-e[1]).powi(2) + (c[2]-e[2]).powi(2)).sqrt();
                    let ok = match sim.world.segment_bullet(e, *c) { None => true, Some(t) => (1.0 - t) * d < *r + 0.05 };
                    if ok { seen.push(*k); }
                }
                rows.push((seen.len(), [x, y, z], seen));
                z += 3.0;
            }
            y += 2.0;
        }
        x += 2.0;
    }
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    for r in rows.iter().take(25) { println!("{:?}", r); }
    for x in [12.7f32, 25.7, 38.7, 51.8] {
        for r in rows.iter().filter(|r| (r.1[0] - x).abs() < 1.1 && r.1[2] == 0.0) { println!("station x{x}: {:?}", r); }
    }
}

/// Console phase v4: one float station that sees every screen; deadline order; burst fire.
#[test]
#[ignore = "assertion-free trace; run with --ignored"]
fn trace_0700001b_consoles4() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let order: Vec<u32> = std::env::var("NF_1B_ORDER").ok().map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect())
        .unwrap_or(vec![3, 4, 5, 6, 8, 1, 2, 7]);
    let hold: [f32; 3] = std::env::var("NF_1B_HOLD").ok().map(|s| { let v: Vec<f32> = s.split(',').filter_map(|x| x.parse().ok()).collect(); [v[0], v[1], v[2]] })
        .unwrap_or([32.0, -1.7, 0.0]);
    let burst: u32 = std::env::var("NF_1B_BURST").ok().and_then(|s| s.parse().ok()).unwrap_or(96);
    let tol: f32 = std::env::var("NF_1B_TOL").ok().and_then(|s| s.parse().ok()).unwrap_or(0.006);
    let mut w = walker::Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    if std::env::var_os("NF_1B_NOTOPUP").is_some() { w.top_up_health = false; }
    let mut prev = w.scene.sim.as_ref().unwrap().drones.channels.v.clone();
    let mut t = 0u32;
    while t < 1600 { w.tick(Default::default(), false); log_changes(&w, &mut prev, t); t += 1; }
    while !w.scene.sim.as_ref().unwrap().drones.channels.on(19) && t < 6000 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().position(|d| d.present && !d.corpse && d.enabled && d.health > 0.0 && d.skin & 0xffff == 0x78)
        };
        if let Some(di) = target { let _ = w.shoot(di, "astronaut"); } else {
            let input = thrust_to(w.scene.sim.as_ref().unwrap(), hold, 0.15);
            w.tick(input, false); t += 1;
        }
        log_changes(&w, &mut prev, t);
    }
    println!("ch19 at frame {} clip {}", w.scene.sim.as_ref().unwrap().drones.frame, w.scene.sim.as_ref().unwrap().gun.clip);
    let mut last_shot = 0u32;
    let mut in_burst = false;
    let mut shots = 0u32;
    for tt in 0..6000u32 {
        let (input, fire, k) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let Some(&k) = order.iter().find(|&&k| !sim.drones.channels.on(k)) else { break };
            let pi = sim.drones.scripts.players.iter().position(|p| p.gate as u32 == 30 + k && p.hit_points > 0.0).unwrap();
            let tg = sim.drones.scripts.held_hit_targets(&data_of(sim));
            let best = tg.iter().filter(|x| x.0 == pi).max_by(|a, b| a.2.partial_cmp(&b.2).unwrap()).map(|x| x.1);
            let station = match k { 7 | 8 => [46.0, -7.0, 0.0], 1 | 2 => [16.0, -3.0, 0.0], _ => hold };
            let mut inp = thrust_to(sim, station, 0.15);
            let near = walker::dxz(sim.bond.pos, station) < 2.5;
            let mut fire = false;
            if let Some(c) = best {
                let (e, pe) = walker::zero_g_errs(sim, c).unwrap_or((0.0, 0.0));
                inp.turn = e.clamp(-0.08, 0.08);
                inp.look = pe.clamp(-0.08, 0.08);
                let clip = sim.gun.clip;
                if clip >= burst { in_burst = true; }
                if clip < 12 { in_burst = false; }
                fire = near && in_burst && e.abs() < tol && pe.abs() < tol && tt.saturating_sub(last_shot) >= 12 && tt % 6 == 0;
            }
            if tt % 300 == 0 { println!("   tt{tt} k{k} best {:?} errs {:?} clip {} burst {in_burst} shots {shots}", best, best.and_then(|c| walker::zero_g_errs(sim, c)), sim.gun.clip); }
            (inp, fire, k)
        };
        if fire {
            let c0 = w.scene.sim.as_ref().unwrap().gun.clip;
            w.tick_fire(input);
            if w.scene.sim.as_ref().unwrap().gun.clip < c0 { shots += 1; last_shot = tt; }
        } else { w.tick(input, false); }
        t += 1;
        let before: Vec<bool> = (1..=8).map(|c| prev[c] != 0).collect();
        log_changes(&w, &mut prev, t);
        let sim = w.scene.sim.as_ref().unwrap();
        for c in 1..=8usize { if !before[c - 1] && prev[c] != 0 { println!("CONSOLE {c} latched frame {} shots so far {shots} clip {} bond ({:.1},{:.1},{:.1})", sim.drones.frame, sim.gun.clip, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]); } }
        let _ = k;
    }
    if std::env::var_os("NF_1B_DRAKE").is_some() {
        let mut drake = None;
        for _ in 0..6000u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) { drake = Some(i); break; }
            let input = thrust_to(sim, [32.0, -1.7, 0.0], 0.15);
            w.tick(input, false); t += 1; log_changes(&w, &mut prev, t);
        }
        println!("drake {:?} frame {}", drake, w.scene.sim.as_ref().unwrap().drones.frame);
        if let Some(di) = drake {
            w.skip_cutscenes = false;
            let ok = w.shoot(di, "Drake");
            log_changes(&w, &mut prev, t);
            let sim = w.scene.sim.as_ref().unwrap();
            println!("drake shot {ok} frame {} ch0e {} hp {:.1}", sim.drones.frame, sim.drones.channels.on(0x0e), sim.drones.list[di].health);
        }
    }
    let n: u32 = std::env::var("NF_1B_TICKS").ok().and_then(|s| s.parse().ok()).unwrap_or(4000);
    for _ in 0..n { w.tick(Default::default(), false); t += 1; log_changes(&w, &mut prev, t); }
    let sim = w.scene.sim.as_ref().unwrap();
    println!("health {:.1} dead? mstat {}", w.scene.sim.as_ref().unwrap().drones.health.health, w.scene.sim.as_ref().unwrap().drones.mission.status());
    println!("end: missiles {:?} deploy {:?} mstat {} frame {}", sim.drones.space_missiles.iter().map(|m| m.state).collect::<Vec<_>>(), sim.drones.missile_deploy, sim.drones.mission.status(), sim.drones.frame);
    for (i, d) in sim.drones.list.iter().enumerate() {
        if d.present { println!("   drone {i} skin {:#x} pos ({:.2},{:.2},{:.2}) en {} corpse {} state {:#x}", d.skin, d.pos[0], d.pos[1], d.pos[2], d.enabled, d.corpse, d.state); }
    }
}
