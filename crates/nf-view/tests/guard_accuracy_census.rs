//! Guard accuracy census: the port's hit rate on an idle Bond against `DroneWeap_DoBulletAccuracy`'s
//! predicted rate for the same inputs. Opt-in via NF_ISO.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[derive(Default, Debug)]
struct Row { weapon: u32, acc: u8, shots: usize, rounds: usize, rolled: usize, hits: usize, dist_sum: f32, pred_sum: f32 }

/// Idle Bond at the level start, health held, `settle` ticks then `ticks` counted.
fn census(level: &str, diff: u32, crouch: bool, settle: u32, ticks: u32) -> Option<Vec<(usize, Row)>> {
    let iso = iso()?;
    let mut scene = nf_view::load_with_carry_difficulty(&iso, level, false, None, diff).ok()?;
    let lvl = scene.sim.as_ref().unwrap().level;
    let mut rows: std::collections::BTreeMap<usize, Row> = Default::default();
    let mut seen_shots: Vec<usize> = Vec::new();
    let trace: Option<usize> = std::env::var("NF_CEN_TRACE").ok().and_then(|v| v.parse().ok());
    let mut prev_b: Vec<([f32; 3], [f32; 3], f32)> = Vec::new();
    for t in 0..settle + ticks {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.player_hits.clear();
        sim.drones.health.health = 100.0;
        sim.drones.health.armour = 0.0;
        let inp = Input { crouch: crouch && t == 200, ..Default::default() };
        sim.step_bond(inp, false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if seen_shots.len() < sim.drones.list.len() { seen_shots.resize(sim.drones.list.len(), 0); }
        if t < settle { for (i, d) in sim.drones.list.iter().enumerate() { seen_shots[i] = d.shots.len(); } continue; }
        let d = sim.data();
        for (i, dr) in sim.drones.list.iter().enumerate() {
            let n = dr.shots.len();
            if n > seen_shots[i] {
                let r = rows.entry(i).or_default();
                r.weapon = dr.weapon; r.acc = dr.accuracy;
                let k = n - seen_shots[i];
                r.shots += k;
                let per = d.weapons.records.get(dr.weapon as usize).map(|w| w[0x18] as usize).unwrap_or(1);
                r.rounds += k * per;
                r.rolled += dr.shot_rolls.iter().rev().take(k).filter(|&&x| x).count();
                r.dist_sum += dr.dist * k as f32;
                let p = nf_game::drone::hit_chance(d.tuning, lvl, diff, dr.accuracy, dr.dist, dr.target_motion, dr.anim_set) / 100.0;
                r.pred_sum += p.clamp(0.0, 1.0) * k as f32;
                seen_shots[i] = n;
            }
        }
        if let Some(w) = trace {
            let now: Vec<([f32; 3], [f32; 3], f32)> = sim.drones.bullets.iter().filter(|b| b.owner == w).map(|b| (b.pos, b.dir, b.travelled)).collect();
            for pb in &prev_b { if !now.iter().any(|n| n.1 == pb.1) { eprintln!("[cen] t{t}   round gone, last {:?} dir {:?} trav {:.2}", pb.0, pb.1, pb.2); } }
            prev_b = now;
        }
        if let Ok(w) = std::env::var("NF_CEN_TRACE") { let w: usize = w.parse().unwrap();
            if let Some(dr) = sim.drones.list.get(w) { if let Some(a) = dr.shot_aims.last() { if a.0 + 1 >= sim.drones.frame {
                eprintln!("[cen] t{t} f{} shot muzzle {:?} aim {:?} bond {:?} roll {:?} dist {:.2} pos {:?} yaw {:.2}", a.0, a.1, a.2, sim.bond.pos, dr.shot_rolls.last(), dr.dist, dr.pos, dr.yaw); } } }
            for e in &sim.drones.impacts { eprintln!("[cen] t{t}   impact {e:?}"); }
            for e in &sim.drones.drone_round_hits { eprintln!("[cen] t{t}   drone-hit {e:?}"); }
            for e in &sim.drones.player_hits { eprintln!("[cen] t{t}   player-hit {e:?}"); }
        }
        for &(_, owner, dmg) in &sim.drones.player_hits { if dmg > 0.0 { if let Some(r) = rows.get_mut(&owner) { r.hits += 1; } } }
    }
    Some(rows.into_iter().collect())
}

fn print(label: &str, rows: &[(usize, Row)]) {
    let (mut s, mut ro, mut rl, mut h, mut pr) = (0, 0, 0, 0, 0.0);
    for (i, r) in rows {
        eprintln!("[cen] {label} drone {i} w {:#x} acc {} shots {} rounds {} rolled {} hits {} mean-dist {:.1} pred {:.1}",
            r.weapon, r.acc, r.shots, r.rounds, r.rolled, r.hits, r.dist_sum / r.shots.max(1) as f32, r.pred_sum);
        s += r.shots; ro += r.rounds; rl += r.rolled; h += r.hits; pr += r.pred_sum;
    }
    eprintln!("[cen] {label} TOTAL shots {s} rounds {ro} rolled {rl} hits {h} predicted {pr:.1}");
}

#[test]
#[ignore = "exploration printout"]
fn print_12_start() {
    if iso().is_none() { return; }
    for crouch in [false, true] {
        if let Some(r) = census("07000012.bin", 1, crouch, 240, 1200) { print(&format!("12 d1 crouch {crouch}"), &r); }
    }
}

#[test]
#[ignore = "exploration printout"]
fn probe_12_north_guard_line() {
    let Some(iso) = iso() else { return };
    let scene = nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 1).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    for (from, to) in [([14.4364f32, 1.41159, 18.81194], [22.48982f32, 1.33920, 19.15544]), ([14.30929, 0.83610, 18.87332], [22.48982, 1.33920, 19.15544])] {
        let h = sim.world.segment_surface_host(from, to, false);
        eprintln!("[cen] probe {from:?} -> {to:?}: {h:?}");
        if let Some((t, ..)) = h { let p: Vec<f32> = (0..3).map(|k| from[k] + (to[k] - from[k]) * t).collect(); eprintln!("[cen]   at {p:?}"); }
        eprintln!("[cen]   segment_bullet {:?} segment {:?}", sim.world.segment_bullet(from, to), sim.world.segment(from, to));
    }
    for (i, d) in sim.drones.list.iter().enumerate() { eprintln!("[cen] drone {i} pos {:?} weapon {:#x} skin {:#x} present {} enabled {}", d.pos, d.weapon, d.skin, d.present, d.enabled); }
}

#[test]
#[ignore = "exploration printout"]
fn probe_12_cover_object() {
    let Some(iso) = iso() else { return };
    let scene = nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 1).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let (from, to) = ([14.30929f32, 0.83610, 18.87332], [22.48982f32, 1.33920, 19.15544]);
    let hit = [17.893494f32, 1.0565274, 18.996927];
    for cell in 0..400usize {
        let Some(runs) = sim.world.scoped_run_bounds(cell, from, to) else { continue };
        for (r, owner, lo, hi) in runs {
            if (0..3).all(|k| hit[k] >= lo[k] - 0.05 && hit[k] <= hi[k] + 0.05) { eprintln!("[cen] cell {cell} run {r} owner {owner:#x} lo {lo:?} hi {hi:?}"); }
        }
    }
}

#[test]
#[ignore = "exploration printout"]
fn print_view_bones() {
    let Some(iso) = iso() else { return };
    for (lvl, crouch) in [("07000012.bin", false), ("07000012.bin", true), ("07000002.bin", false)] {
        let mut scene = nf_view::load_with_carry_difficulty(&iso, lvl, false, None, 1).expect("load");
        for t in 0..300 { scene.sim.as_mut().unwrap().step_bond(Input { crouch: crouch && t == 0, ..Default::default() }, false, false, false); }
        let sim = scene.sim.as_ref().unwrap();
        let b = sim.bond.pos;
        let rel: Vec<[f32; 3]> = sim.bond_view_bones().unwrap_or_default().iter().map(|v| [v[0] - b[0], v[1] - b[1], v[2] - b[2]]).collect();
        eprintln!("[cen] {lvl} crouch {crouch} (crouched {}) pos {b:?} yaw {:.3} target-rel {:?} bones-rel {rel:?}", sim.bond.crouched, sim.bond.yaw,
            sim.drones.list.iter().find(|d| d.present && d.team == 1).map(|d| [d.opp_target[0] - b[0], d.opp_target[1] - b[1], d.opp_target[2] - b[2]]));
    }
}

#[test]
#[ignore = "exploration printout: NF_CEN_LEVELS=07000001,07000002,..."]
fn print_level_starts() {
    if iso().is_none() { return; }
    let list = std::env::var("NF_CEN_LEVELS").unwrap_or_else(|_| "07000012".into());
    for l in list.split(',') {
        match census(&format!("{l}.bin"), 1, false, 240, 1200) { Some(r) => print(&format!("{l} d1"), &r), None => eprintln!("[cen] {l}: no load") }
    }
}

/// The aim point a guard's rounds fly at [M `NDrone2_SetOpponentAimPos` 0x141fe8:20-45, run every
/// tick from `NDrone2_FindOpponent` 0x1424e0:120 → `NDrone2_SetOpponent` 0x1422d0:30-33/:73;
/// `DroneWeap_FireWeapon` 0x178f68 aims at `+0x1f0 + +0x200`]: the player's bone 2
/// (`Drone_View_Bones[+0x1da]` while the last sight ray landed, `+0x228 & 2`), not a fixed point
/// under the eye. Crouched at the 07000012 start that is ~0.16 over obj+0x30; the old point (the
/// eye − 0.45) sat 0.2 UNDER it, so the start pair shot at Bond's knees. Sister: the old rule fails
/// both the bone match and the height band.
#[test]
fn guards_aim_at_bonds_view_bone() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 1).expect("load");
    for t in 0..400 { scene.sim.as_mut().unwrap().step_bond(Input { crouch: t == 200, ..Default::default() }, false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.bond.crouched, "Bond crouched");
    let bones = sim.bond_view_bones().expect("Bond's posed view bones");
    let b = sim.bond.pos;
    let mut n = 0;
    for (i, d) in sim.drones.list.iter().enumerate().filter(|(_, d)| d.present && d.enabled && d.team == 1 && !d.corpse) {
        let want = bones[if d.los { d.view_bone as usize } else { 0 }];
        let got = d.opp_target;
        assert!((0..3).all(|k| (got[k] - want[k]).abs() < 1e-4), "drone {i}: aim {got:?}, Bond's view bone {want:?}");
        if d.los && d.view_bone == 0 {
            let dy = got[1] - b[1];
            assert!(dy > 0.1 && dy < 0.25, "drone {i}: bone 2 sits {dy} over obj+0x30 crouched");
            n += 1;
        }
    }
    assert!(n >= 2, "the start pair has Bond in sight ({n})");
}
